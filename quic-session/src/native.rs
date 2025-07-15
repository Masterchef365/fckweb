use anyhow::{Context, Result};
use quinn::{IdleTimeout, TransportConfig, VarInt};
use rustls::pki_types::CertificateDer;
use std::net::SocketAddr;
use std::sync::Arc;
use url::Url;

pub async fn client_session_selfsigned(
    url: &Url,
    certificate: Vec<u8>,
    _certificate_hashes: Vec<u8>,
) -> Result<web_transport::Session> {
    client_session(url, certificate).await
}

pub async fn client_session(url: &Url, certificate: Vec<u8>) -> Result<web_transport::Session> {
    // Read the PEM certificate chain
    let mut chain = std::io::Cursor::new(certificate);

    let chain: Vec<CertificateDer> = rustls_pemfile::certs(&mut chain)
        .collect::<Result<_, _>>()
        .context("failed to load certs")?;

    anyhow::ensure!(!chain.is_empty(), "could not find certificate");

    let client = web_transport_quinn::ClientBuilder::new().with_server_certificates(chain)?;

    // Connect to the given URL.
    let session = client.connect(url.clone()).await?; // Connect to the given URL.
                                                      //let session = web_transport_quinn::connect(&client, &url).await?;

    Ok(session.into())
}

pub async fn server_endpoint(
    bind: SocketAddr,
    certificate: Vec<u8>,
    key: Vec<u8>,
) -> Result<web_transport_quinn::Server> {
    let mut chain = std::io::Cursor::new(certificate);

    let chain: Vec<CertificateDer> = rustls_pemfile::certs(&mut chain)
        .collect::<Result<_, _>>()
        .context("failed to load certs")?;

    anyhow::ensure!(!chain.is_empty(), "could not find certificate");

    // Read the PEM private key
    let mut keys = std::io::Cursor::new(key);

    // Try to parse a PKCS#8 key
    // -----BEGIN PRIVATE KEY-----
    let key = rustls_pemfile::private_key(&mut keys)
        .context("failed to load private key")?
        .context("missing private key")?;

    let server = web_transport_quinn::ServerBuilder::new()
        .with_addr(bind)
        .with_certificate(chain, key)?;

    log::info!("listening on {}", bind);

    Ok(server)
}

pub async fn server_connect(inc: web_transport_quinn::Request) -> Result<web_transport::Session> {
    let session = inc.ok().await.context("failed to accept connection")?;

    //let request = web_transport_quinn::accept(conn).await?;
    //let session = request.ok().await.context("failed to accept session")?;

    Ok(session.into())
}

fn transport_config() -> TransportConfig {
    let mut transport_config = TransportConfig::default();

    // Timeout set for 10 days, the default was 30 seconds lol
    transport_config.max_idle_timeout(Some(IdleTimeout::from(VarInt::from_u32(
        10 * 24 * 60 * 60 * 1000,
    ))));

    transport_config
}
