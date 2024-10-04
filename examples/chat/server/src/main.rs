use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chat_common::*;
use framework::futures::{Sink, SinkExt, Stream, TryFutureExt};
use framework::io::FrameworkError;
use framework::tarpc::context::Context as TarpcContext;
use framework::{
    futures::StreamExt,
    tarpc::server::{BaseChannel, Channel},
    ServerFramework,
};
use tokio::sync::mpsc::Sender as TokioSender;
use tokio::sync::Mutex as TokioMutex;

#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = quic_session::server_endpoint(
        "0.0.0.0:9090".parse().unwrap(),
        include_bytes!("localhost.crt").to_vec(),
        include_bytes!("localhost.key").to_vec(),
    )
    .await?;

    let shared = Arc::new(TokioMutex::new(SharedData::default()));

    while let Some(inc) = endpoint.accept().await {
        println!("new connection");
        tokio::spawn(async move {
            let sess = quic_session::server_connect(inc).await?;

            // Spawn the root service
            let (framework, channel) = ServerFramework::new(sess).await?;
            let transport = BaseChannel::with_defaults(channel);

            let server = ChatServer::new(framework);
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
struct ChatServer {
    framework: ServerFramework,
    shared: Arc<TokioMutex<SharedData>>,
}

#[derive(Default)]
struct SharedData {
    rooms: HashMap<String, Arc<TokioMutex<Room>>>,
}

type MessageSink = Box<dyn Sink<MessageMetaData, Error = FrameworkError> + Send + Sync + Unpin + 'static>;

struct Room {
    desc: RoomDescription,
    messages: Vec<MessageMetaData>,
    connected: Vec<MessageSink>,
    tx: TokioSender<MessageMetaData>,
}

impl ChatServer {
    pub fn new(framework: ServerFramework) -> Self {
        Self {
            framework,
            shared: Default::default(),
        }
    }
}

impl ChatService for ChatServer {
    async fn create_room(self, context: TarpcContext, desc: RoomDescription) -> bool {
        todo!()
    }

    async fn get_rooms(self, context: TarpcContext) -> HashMap<String, RoomDescription> {
        todo!()
    }

    async fn chat(
        self,
        context: TarpcContext,
        room_name: String,
        username: String,
        user_color: [u8; 3],
    ) -> Result<framework::BiStream<MessageMetaData, MessageMetaData>, ChatError> {
        let (handle, streamfut) = self.framework.accept_bistream();

        let shared = self.shared.clone();
        tokio::spawn(async move {
            let streams = streamfut.await?;
            let (sink, stream) = streams.split();

            let shared = shared.lock().await;
            let room = shared.get_room(&room_name).await?;
            drop(shared);
            let mut room = room.lock().await;
            room.connected.push(Box::new(sink));

            Ok::<_, anyhow::Error>(())
        });

        Ok(handle)
    }
}

impl SharedData {
    async fn get_room(&self, room_name: &str) -> Result<Arc<TokioMutex<Room>>, ChatError> {
        self.rooms
            .get(room_name)
            .ok_or_else(|| ChatError::RoomDoesNotExist(room_name.to_string()))
            .cloned()
    }

    async fn create_room(&mut self, desc: RoomDescription) -> bool {
        if self.rooms.contains_key(&desc.name) {
            false
        } else {
            self.rooms.insert(desc.name.clone(), Room::new(desc).await);
            true
        }
    }
}

impl Room {
    async fn new(desc: RoomDescription) -> Arc<TokioMutex<Self>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let inst = Self {
            tx,
            desc,
            connected: vec![],
            messages: Default::default(),
        };

        let inst = Arc::new(TokioMutex::new(inst));

        let room = inst.clone();

        tokio::spawn(async move {
            // TODO: This is straightforward but slow!
            while let Some(msg) = rx.recv().await {
                let mut lck = room.lock().await;
                for conn in &mut lck.connected {
                    conn.send(msg.clone()).await?;
                }
            }

            Ok::<_, anyhow::Error>(())
        });

        inst
    }
}
