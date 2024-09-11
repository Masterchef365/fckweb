use url::Url;
use std::sync::Arc;
use anyhow::{Context, Result};

#[cfg(target_arch = "wasm32")]
pub async fn session(url: &Url) -> Result<web_transport::Session> {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn session(url: &Url) -> Result<web_transport::Session> {
    // Read the PEM certificate chain

    use rustls::pki_types::CertificateDer;
    let chain = std::fs::File::open("certificate.pem").context("failed to open cert file")?;
    let mut chain = std::io::BufReader::new(chain);

    let chain: Vec<CertificateDer> = rustls_pemfile::certs(&mut chain)
        .collect::<Result<_, _>>()
        .context("failed to load certs")?;

    anyhow::ensure!(!chain.is_empty(), "could not find certificate");

    let mut roots = rustls::RootCertStore::empty();
    roots.add_parsable_certificates(chain);

    // Standard quinn setup, accepting only the given certificate.
    // You should use system roots in production.
    let mut config = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])?
    .with_root_certificates(roots)
    .with_no_client_auth();
    config.alpn_protocols = vec![web_transport_quinn::ALPN.to_vec()]; // this one is important

    let config: quinn::crypto::rustls::QuicClientConfig = config.try_into()?;
    let config = quinn::ClientConfig::new(Arc::new(config));

    let mut client = quinn::Endpoint::client("[::]:0".parse()?)?;
    client.set_default_client_config(config);

    // Connect to the given URL.
    let session = web_transport_quinn::connect(&client, &url).await?;

    Ok(session.into())
}
