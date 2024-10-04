use anyhow::Result;
use chat_common::ChatService;
use framework::{
    futures::StreamExt,
    tarpc::server::{BaseChannel, Channel},
    ServerFramework,
};

#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = quic_session::server_endpoint(
        "0.0.0.0:9090".parse().unwrap(),
        include_bytes!("localhost.crt").to_vec(),
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
            let executor = transport.execute(ChatService::serve(server));

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

impl ChatService for MyServiceServer {
    async fn create_room(self,context: framework::tarpc::context::Context,name:String) -> bool {
        todo!()
    }

    async fn get_rooms(self,context: framework::tarpc::context::Context,) -> std::collections::HashMap<String,chat_common::RoomDescription> {
        todo!()
    }

    async fn chat(self,context: framework::tarpc::context::Context,room_name:String,username:String,user_color:[u8;
    3]) -> framework::BiStream<chat_common::MessageMetaData,chat_common::ChatMessage> {
        todo!()
    }
}
