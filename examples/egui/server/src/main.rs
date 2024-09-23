use anyhow::Result;
use common::{MyOtherService, MyService};
use framework::{
    futures::StreamExt,
    tarpc::server::{BaseChannel, Channel}, ServerFramework,
};

#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = quic_session::server_endpoint("0.0.0.0:9090".parse().unwrap()).await?;

    while let Some(inc) = endpoint.accept().await {
        println!("new connection");
        tokio::spawn(async move {
            let sess = quic_session::server_connect(inc).await?;

            // Spawn the root service
            let (frame, channel) = ServerFramework::new(sess).await?;
            let transport = BaseChannel::with_defaults(channel);

            let server = MyServiceServer;
            let executor = transport.execute(MyService::serve(server));

            tokio::spawn(executor.for_each(|response| async move {
                tokio::spawn(response);
            }));

            println!("connection ended");
            Ok::<_, anyhow::Error>(())
        });
    }

    Ok(())
}

#[derive(Clone)]
struct MyServiceServer;

impl MyService for MyServiceServer {
    async fn add(self, _context: framework::tarpc::context::Context, a: u32, b: u32) -> u32 {
        a + b
    }

    async fn get_sub(
        self,
        context: framework::tarpc::context::Context,
    ) -> framework::Subservice<common::MyOtherServiceClient> {
        todo!()
    }
}

#[derive(Clone)]
struct MyOtherServiceServer;

impl MyOtherService for MyServiceServer {
    async fn subtract(self, _context: framework::tarpc::context::Context, a: u32, b: u32) -> u32 {
        a - b
    }
}


