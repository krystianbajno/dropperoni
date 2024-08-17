use std::error::Error;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use reqwest::Client;
use tokio_rustls::rustls::pki_types::PrivateKeyDer;
use tokio_rustls::rustls::pki_types::CertificateDer;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

use std::sync::Arc;
use crate::ssl::generate_self_signed_certificate;

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
            if let Err(e) = handle_connection(acceptor, stream, &target_address).await {
                eprintln!("Error handling connection: {:?}", e);
            } else {
            }
        });
    }

    Ok(())
}

async fn handle_connection(acceptor: TlsAcceptor, stream: TcpStream, target_address: &str) -> Result<(), Box<dyn Error>> {
    let mut tls_stream = acceptor.accept(stream).await?;

    let mut buffer = [0; 1024];
    let mut request_data = Vec::new();
    
    loop {
        let n = tls_stream.read(&mut buffer).await?;
        if n == 0 {
            println!("Received no data from the client.");
            return Ok(());
        }
        request_data.extend_from_slice(&buffer[..n]);

        if request_data.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    let request = String::from_utf8(request_data)?;
    println!("Received request:\n{}", request);

    let (method, uri) = match parse_request(&request) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Failed to parse request: {}", e);
            return Ok(());
        }
    };

    let url = format!("http://{}{}", target_address, uri);
    println!("Forwarding request to: {}", url);

    let client = Client::new();
    let response = match method.as_str() {
        "GET" => client.get(&url).send().await?,
        "POST" => {
            let body = extract_body(&request);
            client.post(&url).body(body).send().await?
        },
        "PUT" => {
            let body = extract_body(&request);
            client.put(&url).body(body).send().await?
        },
        "DELETE" => client.delete(&url).send().await?,
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url).send().await?,
        _ => {
            eprintln!("Unsupported HTTP method: {}", method);
            return Ok(());
        }
    };

    let status_line = format!("HTTP/1.1 {} {}\r\n", response.status().as_u16(), response.status().canonical_reason().unwrap_or("Unknown"));
    let headers = response.headers().iter()
        .map(|(k, v)| format!("{}: {}\r\n", k.as_str(), v.to_str().unwrap_or("Invalid header value")))
        .collect::<String>();
    let body = response.bytes().await?;

    let full_response = format!("{}{}\r\n", status_line, headers);
    tls_stream.write_all(full_response.as_bytes()).await?;
    tls_stream.write_all(&body).await?;
    tls_stream.flush().await?;

    println!("Response sent back to the client.");

    Ok(())
}

fn parse_request(request: &str) -> Result<(String, String), Box<dyn Error>> {
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return Err("Empty request".into());
    }
    
    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err("Invalid HTTP request line".into());
    }

    let method = parts[0].to_string();
    let uri = parts[1].to_string();
    
    Ok((method, uri))
}

fn extract_body(request: &str) -> String {
    let parts: Vec<&str> = request.split("\r\n\r\n").collect();
    if parts.len() > 1 {
        parts[1].to_string()
    } else {
        String::new()
    }
}

fn generate_tls_acceptor(ssl_issuer: &str) -> Result<TlsAcceptor, Box<dyn Error>> {
    let (cert, private_key) = generate_self_signed_certificate(ssl_issuer)?;
    let rustls_cert = CertificateDer::try_from(cert)?;
    let rustls_private_key = PrivateKeyDer::try_from(private_key)?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![rustls_cert], rustls_private_key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
