use anyhow::Result;
use quic_session::web_transport::Session;

#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = quic_session::server_endpoint("0.0.0.0:9090".parse().unwrap()).await?;

    while let Some(inc) = endpoint.accept().await {
        tokio::spawn(async move {
            let sess = quic_session::server_connect(inc).await?;
            handler(sess).await?;
            Ok::<_, anyhow::Error>(())
        });
    }

    Ok(())
}

async fn handler(mut sess: Session) -> Result<()> {
    loop {
        let (mut sink, mut stream) = sess.accept_bi().await?;

        tokio::spawn(async move {
            loop {
                if let Some(chunk) = stream.read(512).await? {
                    let s = String::from_utf8(chunk.to_vec())?;
                    println!("{}", s);

                    sink.write(&"Hi back".to_string().into_bytes()).await?;
                }
            }

            #[allow(unreachable_code)]
            Ok::<_, anyhow::Error>(())
        });
    }
}
