use anyhow::Result;
use common::MyService;
use framework::{futures::StreamExt, tarpc::server::{BaseChannel, Channel}};
use quic_session::web_transport::Session;

#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = quic_session::server_endpoint("0.0.0.0:9090".parse().unwrap()).await?;

    while let Some(inc) = endpoint.accept().await {
        println!("new connection");
        tokio::spawn(async move {
            let sess = quic_session::server_connect(inc).await?;
            handler(sess).await?;
            println!("connection ended");
            Ok::<_, anyhow::Error>(())
        });
    }

    Ok(())
}

async fn handler(mut sess: Session) -> Result<()> {
    loop {
        let socks = sess.accept_bi().await?;

        tokio::spawn(async move {
            let transport = framework::io::webtransport_protocol(socks);
            let transport = BaseChannel::with_defaults(transport);

            let server = MyServiceServer;
            let executor = transport.execute(server.serve());

            tokio::spawn(executor.for_each(|response| async move {
                tokio::spawn(response);
            }));
        });
    }
}

#[derive(Clone)]
struct MyServiceServer;

impl MyService for MyServiceServer {
    async fn add(self, _context: framework::tarpc::context::Context, a: u32, b: u32) -> u32 {
        a + b
    }

    async fn get_sub(self,context: framework::tarpc::context::Context,) -> framework::Subservice<common::MyOtherServiceClient> {
        todo!()
    }
}
