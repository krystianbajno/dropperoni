use std::error::Error;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
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
    let mut client_stream = acceptor.accept(stream).await?;

    let mut server_stream = TcpStream::connect(target_address).await?;

    let mut client_to_server_buffer = vec![0u8; 4096];
    let mut server_to_client_buffer = vec![0u8; 4096];

    loop {
        tokio::select! {
            client_read = client_stream.read(&mut client_to_server_buffer) => {
                let n = client_read?;
                if n == 0 {
                    break;
                }
                server_stream.write_all(&client_to_server_buffer[..n]).await?;
            }

            server_read = server_stream.read(&mut server_to_client_buffer) => {
                let n = server_read?;
                if n == 0 {
                    break;
                }
                client_stream.write_all(&server_to_client_buffer[..n]).await?;
            }
        }
    }

    Ok(())
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
