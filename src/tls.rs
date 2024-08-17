use tokio_rustls::rustls::{Certificate, ClientConfig, PrivateKey, RootCertStore, ServerConfig, Error as RustlsError};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::client::TlsStream;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use std::error::Error;
use tokio_rustls::rustls::client::{ServerCertVerifier, ServerCertVerified};

use crate::certs::generate_self_signed_certificate;

struct NoCertVerification;

impl ServerCertVerifier for NoCertVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &tokio_rustls::rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }
}

pub enum MaybeTlsStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl MaybeTlsStream {
    pub async fn read(&mut self, buf: &mut [u8]) -> tokio::io::Result<usize> {
        match self {
            MaybeTlsStream::Plain(stream) => stream.read(buf).await,
            MaybeTlsStream::Tls(stream) => stream.read(buf).await,
        }
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> tokio::io::Result<()> {
        match self {
            MaybeTlsStream::Plain(stream) => stream.write_all(buf).await,
            MaybeTlsStream::Tls(stream) => stream.write_all(buf).await,
        }
    }
}

pub fn generate_tls_acceptor(ssl_issuer: &str) -> Result<TlsAcceptor, Box<dyn Error>> {
    let (cert, private_key) = generate_self_signed_certificate(ssl_issuer)?;
    let rustls_cert = Certificate(cert);
    let rustls_private_key = PrivateKey(private_key);

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![rustls_cert], rustls_private_key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

pub fn generate_tls_connector() -> Result<ClientConfig, Box<dyn Error>> {
    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(RootCertStore::empty())
        .with_no_client_auth();

    config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoCertVerification));


    Ok(config)
}