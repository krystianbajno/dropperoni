use std::error::Error;
use rand::rngs::OsRng;
use rcgen::Certificate;
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::RsaPrivateKey;
use rcgen::{CertificateParams, KeyPair};
use rsa::pkcs8::EncodePrivateKey;

pub fn generate_self_signed_certificate(issuer: &str) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048)?;

    let private_key_pem = private_key.to_pkcs8_pem(rsa::pkcs1::LineEnding::LF)?;
    println!("{:?}", private_key.to_public_key().to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?);

    let mut params = CertificateParams::default();
    params.distinguished_name.push(rcgen::DnType::CommonName, issuer.to_string());

    params.alg = &rcgen::PKCS_RSA_SHA256; 

    let key_pair = KeyPair::from_pem(&private_key_pem)?;
    params.key_pair = Some(key_pair);

    let cert = Certificate::from_params(params)
        .map_err(|e| format!("Failed to generate certificate: {}", e))?;
    let cert_der = cert.serialize_der()
        .map_err(|e| format!("Failed to serialize certificate to DER: {}", e))?;
    let private_key_der = private_key.to_pkcs8_der()
        .map_err(|e| format!("Failed to convert private key to DER: {}", e))?;

    Ok((cert_der, private_key_der.to_bytes().to_vec()))
}

