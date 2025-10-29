use std::fs;
use std::io::BufReader;
use std::path::Path;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

const CERT_FILE: &str = "server-cert.pem";
const KEY_FILE: &str = "server-key.pem";

/// Generate a self-signed certificate and private key
pub fn generate_self_signed_cert() -> Result<(), Box<dyn std::error::Error>> {
    let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];

    let cert = rcgen::generate_simple_self_signed(subject_alt_names)?;
    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();

    fs::write(CERT_FILE, cert_pem)?;
    fs::write(KEY_FILE, key_pem)?;

    println!("✓ Generated self-signed certificate");

    Ok(())
}

/// Load certificates from file, or generate if they don't exist
pub fn load_certs() -> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error>> {
    if !Path::new(CERT_FILE).exists() {
        generate_self_signed_cert()?;
    }

    let cert_file = fs::File::open(CERT_FILE)?;
    let mut reader = BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(certs)
}

/// Load private key from file
pub fn load_private_key() -> Result<PrivateKeyDer<'static>, Box<dyn std::error::Error>> {
    let key_file = fs::File::open(KEY_FILE)?;
    let mut reader = BufReader::new(key_file);

    let key = rustls_pemfile::private_key(&mut reader)?
        .ok_or("No private key found")?;

    Ok(key)
}
