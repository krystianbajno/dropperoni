use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::convert::TryFrom;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tokio_rustls::rustls::ServerName;
use colored::*;

use crate::crypto::tls::{generate_tls_acceptor, generate_tls_connector, prepare_tls_cert, MaybeTlsStream};
use crate::mitm::mitm_handler::MitmHandler;

pub async fn start_ssl_proxy(
    server_address: &str,
    target_address: &str,
    ssl_issuer: &str,
    private_key_path: Option<&Path>,
    cert_path: Option<&Path>,
) -> Result<(), Box<dyn Error>> {
    let (cert, private_key) = prepare_tls_cert(ssl_issuer, private_key_path, cert_path)?;

    let acceptor = generate_tls_acceptor(cert, private_key)?;

    let listener = TcpListener::bind(server_address).await?;

    while let Ok((stream, _)) = listener.accept().await {
        let ip = stream.peer_addr().unwrap().ip();
        let port = stream.peer_addr().unwrap().port();
        println!("{}", format!("{ip}:{port} - Accepted a new TLS connection").green());

        let acceptor = acceptor.clone();
        let target_address = target_address.to_string();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(acceptor, stream, target_address).await {
                eprintln!("{}", format!("Error handling connection: {:?}", e).red());
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    acceptor: TlsAcceptor,
    stream: TcpStream,
    target_address: String,
) -> Result<(), Box<dyn Error>> {
    println!("{}", "Accepted connection from client.".cyan());

    let mitm_handler = MitmHandler::new();

    let mut client_stream = acceptor.accept(stream).await?;
    println!("{}", "TLS handshake with client successful.".cyan());

    let (trim_target_address, domain, is_target_https) = {
        let trim_target_address = target_address.trim_start_matches("https://").trim_start_matches("http://");
        let is_target_https = target_address.starts_with("https://");
        let domain = trim_target_address.split(':').next().ok_or("Invalid target address")?.to_string();

        (trim_target_address, domain.to_string(), is_target_https)
    };

    let mut server_stream = if is_target_https {
        println!("{}", format!("Connecting to target (TLS): {}", trim_target_address).green());

        let connector = generate_tls_connector()?;
        let server_name = ServerName::try_from(domain.as_str()).map_err(|_| "Invalid domain for ServerName")?;

        let stream = TcpStream::connect(trim_target_address).await?;
        let server_stream = TlsConnector::from(Arc::new(connector)).connect(server_name, stream).await?;
        println!("{}", "TLS handshake with server successful.".green());

        MaybeTlsStream::Tls(server_stream)
    } else {
        println!("{}", format!("Connecting to target (plain): {}", trim_target_address).green());

        let server_stream = TcpStream::connect(trim_target_address).await?;
        println!("{}", "Connected to target (plain).".green());

        MaybeTlsStream::Plain(server_stream)
    };

    let mut client_to_server_buffer = vec![0u8; 4096];
    let mut server_to_client_buffer = vec![0u8; 4096];
    let mut response_buffer = Vec::new();
    let mut headers_parsed = false;

    loop {
        tokio::select! {
            client_read = client_stream.read(&mut client_to_server_buffer) => {
                let n = client_read?;
                if n == 0 {
                    println!("{}", "Client closed the connection.".cyan());
                    break;
                }
                println!("{}", format!("Read {} bytes from client", n).cyan());
                println!("{}", format!("Client data:\n{}", String::from_utf8_lossy(&client_to_server_buffer[..n])).cyan());

                let modified_request = mitm_handler.process_request(&client_to_server_buffer[..n], &domain)?;

                server_stream.write_all(&modified_request).await?;

                println!("{}", format!("Forwarded {} bytes to server.", n).cyan());
            }

            server_read = server_stream.read(&mut server_to_client_buffer) => {
                match server_read {
                    Ok(n) => {
                        if n == 0 {
                            println!("{}", "Server closed the connection.".green());
                            break;
                        }
                        println!("{}", format!("Read {} bytes from server", n).green());
                        println!("{}", format!("Server data:\n{}", String::from_utf8_lossy(&server_to_client_buffer[..n])).green());

                        response_buffer.extend_from_slice(&server_to_client_buffer[..n]);

                        if !headers_parsed && is_end_of_headers(&response_buffer) {
                            headers_parsed = true;

                            let modified_response = mitm_handler.process_response(&response_buffer, &domain)?;

                            client_stream.write_all(&modified_response).await?;

                            println!("{}", format!("Forwarded {} bytes to client.", modified_response.len()).green());

                            response_buffer.clear();
                        } else if headers_parsed {
                            client_stream.write_all(&server_to_client_buffer[..n]).await?;
                            println!("{}", format!("Forwarded {} bytes to client.", &server_to_client_buffer[..n].len()).green());
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", format!("Failed to read from server: {}", e).red());
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn is_end_of_headers(buffer: &[u8]) -> bool {
    buffer.windows(4).any(|window| window == b"\r\n\r\n")
}
