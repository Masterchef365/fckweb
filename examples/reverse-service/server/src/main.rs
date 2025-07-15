use anyhow::Result;
use framework::{
    futures::StreamExt,
    tarpc::{
        self,
        server::{BaseChannel, Channel},
    },
    ServerFramework,
};
use reverse_common::{MyOtherService, MyOtherServiceClient, MyService};

#[tokio::main]
async fn main() -> Result<()> {
    let mut endpoint = quic_session::server_endpoint(
        "0.0.0.0:9090".parse().unwrap(),
        reverse_common::CERTIFICATE.to_vec(),
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
    async fn offer(
        self,
        context: tarpc::context::Context,
        token: framework::OfferedService<MyOtherServiceClient>,
    ) {
        let transport = self.framework.connect_reverse_service(token).await.unwrap();
        tokio::spawn(async move {
            let newclient = MyOtherServiceClient::new(Default::default(), transport);
            tokio::task::spawn(newclient.dispatch);

            let client = newclient.client;
            let _ = dbg!(client.subtract(context, 10, 7).await);
        });
    }
}
