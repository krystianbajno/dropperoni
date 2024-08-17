use std::error::Error;
use std::sync::Arc;
use std::convert::TryFrom;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tokio_rustls::rustls::ServerName;

use crate::tls::{generate_tls_acceptor, generate_tls_connector, MaybeTlsStream};
use crate::mitm::{MitmBuilder, RequestModifier, ResponseModifier, DefaultRequestModifier, DefaultResponseModifier};

struct CustomRequestModifier;

impl RequestModifier for CustomRequestModifier {
    fn modify(&self, request: &str, needle: &str, payload: &str) -> String {
        // Modifying the HOST header is important for proxy to work correctly.
        let payload = format!("Host: {}", payload);
        DefaultRequestModifier.modify(request, "Host:", &payload)
    }
}

struct CustomResponseModifier;

impl ResponseModifier for CustomResponseModifier {
    fn modify(&self, response: &str, _needle: &str, _payload: &str) -> String {
        // Custom logic for modifying the response
        //DefaultResponseModifier.modify(response, "", "")
        response.to_string()
    }
}

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

    // Build the MITM proxy with custom modifiers
    let mitm = MitmBuilder::new()
        .with_request_modifier(Box::new(CustomRequestModifier)) 
        .with_response_modifier(Box::new(CustomResponseModifier))
        .build();

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

                // Use MITM to modify the HTTP request header
                if let Ok(decoded) = std::str::from_utf8(&client_to_server_buffer[..n]) {
                    println!("\n\nReceived from client:\n{}\n\n", decoded);
                    let modified_request = mitm.modify_request(decoded, "", &domain);
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

                        // Use MITM to modify the HTTP response
                        if let Ok(decoded) = std::str::from_utf8(&server_to_client_buffer[..n]) {
                            println!("\n\nReceived from server:\n{}\n\n", decoded);
                            let modified_response = mitm.modify_response(decoded, "", "");
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
