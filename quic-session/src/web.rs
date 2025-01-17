use anyhow::{Context, Result};
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use url::Url;

pub async fn client_session(
    url: &Url,
    certificate: Vec<u8>,
    certificate_hashes: Vec<u8>,
) -> Result<web_transport::Session> {
    let hexes: Vec<u8> = certificate_hashes
        .chunks_exact(2)
        .map(|chunk| u8::from_str_radix(&String::from_utf8(chunk.to_vec()).unwrap(), 16).unwrap())
        .collect();

    Ok(web_transport_wasm::Client::new()
        .server_certificate_hashes(vec![hexes])
        .connect(url)
        .await
        .map_err(|e| anyhow::format_err!("{e}"))?
        .into())
}
