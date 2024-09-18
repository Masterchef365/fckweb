use anyhow::Result;
use quic_session::web_transport;
use web_transport::Session;

#[tokio::main]
async fn main() -> Result<()> {
    let url = url::Url::parse("https://127.0.0.1:9090/")?;
    let sess = quic_session::client_session(&url).await?;
    handler(sess).await?;

    Ok(())
}

async fn handler(mut sess: Session) -> Result<()> {
    let mut n = 0_u64;
    let (mut sink, mut stream) = sess.open_bi().await?;
    loop {
        let payload = format!("{n}").into_bytes();
        sink.write(&payload).await?;

        if let Some(chunk) = stream.read(512).await? {
            let s = String::from_utf8(chunk.to_vec())?;
            println!("{}", s);
        }

        (n, _) = n.overflowing_add(1);
    }
}

