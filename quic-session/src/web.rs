use anyhow::{Context, Result};
use quinn::{IdleTimeout, TransportConfig, VarInt};
use std::net::SocketAddr;
use std::sync::Arc;
use url::Url;
use std::future::Future;

pub async fn client_session(url: &Url) -> Result<web_transport::Session> {
    let hexes = include_bytes!("certs/localhost.hex");
    let hexes: Vec<u8> = hexes
        .chunks_exact(2)
        .map(|chunk| u8::from_str_radix(&String::from_utf8(chunk.to_vec()).unwrap(), 16).unwrap())
        .collect();

    Ok(web_transport_wasm::SessionBuilder::new(url.clone())
        .server_certificate_hashes(vec![hexes])
        .connect()
        .await
        .map_err(|e| anyhow::format_err!("{e}"))?
        .into())
}
