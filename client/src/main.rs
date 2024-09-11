use anyhow::Result;
use quic_session::web_transport::Session;

#[tokio::main]
async fn main() -> Result<()> {
    let url = url::Url::parse("https://127.0.0.1:9090/")?;
    let sess = quic_session::client_session(&url).await?;
    handler(sess).await?;

    Ok(())
}

async fn handler(mut sess: Session) -> Result<()> {
    let mut n = 0_u64;
    loop {
        let payload = format!("{n}").into_bytes();

        sess.send_datagram(payload.into()).await?;

        (n, _) = n.overflowing_add(1);
    }
}

