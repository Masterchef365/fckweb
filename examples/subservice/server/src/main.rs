use anyhow::Result;
use framework::{
    futures::StreamExt,
    tarpc::{
        self,
        server::{BaseChannel, Channel},
    },
    ServerFramework,
};
use subservice_common::{MyOtherService, MyService};

#[tokio::main]
async fn main() -> Result<()> {
    let mut endpoint = quic_session::server_endpoint(
        "0.0.0.0:9090".parse().unwrap(),
        subservice_common::CERTIFICATE.to_vec(),
        include_bytes!("localhost.key").to_vec(),
    )
    .await?;

    while let Some(inc) = endpoint.accept().await {
        println!("new connection");
        tokio::spawn(async move {
            let sess = quic_session::server_connect(inc).await?;

            // Spawn the root service
            let (framework, channel) = ServerFramework::new(sess).await?;
            let transport = BaseChannel::with_defaults(channel);

            let server = MyServiceServer { framework };
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
struct MyServiceServer {
    framework: ServerFramework,
}

impl MyService for MyServiceServer {
    async fn add(self, _context: tarpc::context::Context, a: u32, b: u32) -> u32 {
        a + b
    }

    async fn get_sub(
        self,
        _context: tarpc::context::Context,
    ) -> framework::Subservice<subservice_common::MyOtherServiceClient> {
        println!("Getting sub, accepting");
        let (token, channelfuture) = self.framework.accept_subservice();
        println!("Accepted");

        tokio::spawn(async move {
            let transport = BaseChannel::with_defaults(channelfuture.await?);

            let server = MyOtherServiceServer;
            let executor = transport.execute(MyOtherService::serve(server));

            tokio::spawn(executor.for_each(|response| async move {
                tokio::spawn(response);
            }));

            Ok::<_, anyhow::Error>(())
        });

        token
    }
}

#[derive(Clone)]
struct MyOtherServiceServer;

impl MyOtherService for MyOtherServiceServer {
    async fn subtract(self, _context: tarpc::context::Context, a: u32, b: u32) -> u32 {
        a.saturating_sub(b)
    }
}
