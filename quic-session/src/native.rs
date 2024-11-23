use anyhow::{Context, Result};
use quinn::{IdleTimeout, TransportConfig, VarInt};
use std::net::SocketAddr;
use std::sync::Arc;
use url::Url;

pub async fn client_session(
    url: &Url,
    certificate: Vec<u8>,
    _certificate_hashes: Vec<u8>,
) -> Result<web_transport::Session> {
    // Read the PEM certificate chain

    use rustls::pki_types::CertificateDer;
    //let chain = std::fs::File::open(CERTIFICATE).context("failed to open cert file")?;
    let mut chain = std::io::Cursor::new(certificate);

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
    let mut config = quinn::ClientConfig::new(Arc::new(config));

    config.transport_config(transport_config().into()); // Don't disconnect so soon h

    let mut client = quinn::Endpoint::client("[::]:0".parse()?)?;
    client.set_default_client_config(config);

    // Connect to the given URL.
    let session = web_transport_quinn::connect(&client, &url).await?;

    Ok(session.into())
}

pub async fn server_endpoint(
    bind: SocketAddr,
    certificate: Vec<u8>,
    key: Vec<u8>,
) -> Result<quinn::Endpoint> {
    // Read the PEM certificate chain

    use rustls::pki_types::CertificateDer;
    //let chain = std::fs::File::open(CERTIFICATE).context("failed to open cert file")?;
    let mut chain = std::io::Cursor::new(certificate);

    let chain: Vec<CertificateDer> = rustls_pemfile::certs(&mut chain)
        .collect::<Result<_, _>>()
        .context("failed to load certs")?;

    anyhow::ensure!(!chain.is_empty(), "could not find certificate");

    // Read the PEM private key
    //let mut keys = std::fs::File::open(PRIVATE_KEY).context("failed to open key file")?;

    // Read the keys into a Vec so we can parse it twice.
    //let mut buf = Vec::new();
    //keys.read_to_end(&mut buf)?;
    let buf = key;

    // Try to parse a PKCS#8 key
    // -----BEGIN PRIVATE KEY-----
    let key = rustls_pemfile::private_key(&mut std::io::Cursor::new(&buf))
        .context("failed to load private key")?
        .context("missing private key")?;

    // Standard Quinn setup
    let mut config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])?
    .with_no_client_auth()
    .with_single_cert(chain, key)?;

    config.max_early_data_size = u32::MAX;
    config.alpn_protocols = vec![web_transport_quinn::ALPN.to_vec()]; // this one is important

    let config: quinn::crypto::rustls::QuicServerConfig = config.try_into()?;
    let mut config = quinn::ServerConfig::with_crypto(Arc::new(config));

    config.transport_config(transport_config().into()); // Don't disconnect so soon h

    let endpoint = quinn::Endpoint::server(config, bind)?;

    Ok(endpoint)
}

pub async fn server_connect(inc: quinn::Incoming) -> Result<web_transport::Session> {
    let conn = inc.await.context("failed to accept connection")?;

    let request = web_transport_quinn::accept(conn).await?;
    let session = request.ok().await.context("failed to accept session")?;

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
