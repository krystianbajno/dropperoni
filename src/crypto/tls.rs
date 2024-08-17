use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio_rustls::rustls::{self, Certificate, ClientConfig, Error as RustlsError, PrivateKey, RootCertStore, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::client::TlsStream;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use std::error::Error;
use tokio_rustls::rustls::client::{ServerCertVerifier, ServerCertVerified};
use std::io::BufReader;

use crate::crypto::certs::generate_self_signed_certificate;

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

pub fn prepare_tls_cert(
    ssl_issuer: &str,
    private_key_path: Option<&Path>,
    cert_path: Option<&Path>,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    match (private_key_path, cert_path) {
        (Some(key_path), Some(cert_path)) => {
            let key_file = File::open(key_path)?;
            let cert_file = File::open(cert_path)?;
            let private_key = pkcs8_private_keys(&mut BufReader::new(key_file))?.remove(0);
            let cert = certs(&mut BufReader::new(cert_file))?.remove(0);
            Ok((cert, private_key))
        }
        _ => generate_self_signed_certificate(ssl_issuer),
    }
}

pub fn generate_tls_acceptor(cert: Vec<u8>, private_key: Vec<u8>) -> Result<TlsAcceptor, Box<dyn Error>> {
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
