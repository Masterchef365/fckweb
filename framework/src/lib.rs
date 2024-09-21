pub use tarpc;
pub use serde;
use tarpc::{transport::channel::UnboundedChannel, Transport};
use tokio::io::{AsyncWriteExt, DuplexStream, SimplexStream, ReadHalf, WriteHalf};
use std::{marker::PhantomData, sync::Arc, task::Poll};
use serde::{Serialize, Deserialize, de::DeserializeOwned};

use futures::{AsyncRead, Sink, SinkExt, Stream, StreamExt};
use web_transport::{RecvStream, SendStream, Session};

pub struct Framework {
    pub sess: Session,
    pub next_id: usize,
}

/// Don't worry about it
#[cfg(target_arch = "wasm32")]
unsafe impl Send for Framework {}

impl Framework {
    pub fn new(sess: Session) -> Self {
        Self { sess, next_id: 0 }
    }

    fn get_next_id(&mut self) -> usize {
        let next = self.next_id + 1;
        std::mem::replace(&mut self.next_id, next)
    }
}

/// Internal type representing the identity of a connection between client and server
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct BiStream(usize);

/*
/// Uniquely identifies a stream, and carries type information about its contents.
/// This is the type used to transmit information between client and server about the identity of a
/// connected stream/sink combo.
///
/// This is a type you should return from your API, in order to get a bidirectional stream on the other end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TypedBiStream<ClientToServer, ServerToClient> {
    id: BiStream,
    _phantom: PhantomData<(ClientToServer, ServerToClient)>,
}

impl<CTS, STC> TypedBiStream<CTS, STC> {
    pub async fn accept(&self, fr: &mut Framework) -> Box<dyn Stream<CTS> + Sink<STC>> {
        todo!()
    }
}
*/

const BUFFER_SIZE: usize = 4096; // Chosen arbitrarily!
const MAX_READ_BYTES: usize = 4096; // Chosen arbitrarily!

/// This is the type used to provide connectivity to an alternate tarpc connection
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TarpcBiStream(BiStream);

pub fn webtransport_rx_stream(mut rx: web_transport::RecvStream) -> ReadHalf<tokio::io::SimplexStream> {
    let (ret, mut proxy) = tokio::io::simplex(BUFFER_SIZE);

    tokio::spawn(async move {
        loop {
            if let Some(bytes) = rx.read(MAX_READ_BYTES).await? {
                proxy.write(bytes.as_ref()).await?;
            }
        }

        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });

    ret
}

/*
pub fn webtransport_futures_bridge<Rx, Tx>((mut rx, mut tx): (RecvStream, SendStream)) -> DuplexStream {
    let (duplex, returned) = tokio::io::duplex(BUFFER_SIZE);

    let duplex = Arc::new(tokio::sync::Mutex::new(duplex));

    let dup1 = duplex.clone();

    tokio::spawn(async move {
        loop {
            if let Some(bytes) = rx.read(MAX_READ_BYTES).await? {
                dup1.lock().await.write(bytes.as_ref()).await?;
            }
        }

        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });

    tokio::spawn(async move {
        loop {
            if let Some(bytes) = rx.read(MAX_READ_BYTES).await? {
                dup1.lock().await.write(bytes.as_ref()).await?;
            }
        }

        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });


    returned
}
*/

/// The encoding function for all data. Mostly for internal use, exposed here for debugging
/// potential
pub fn encode<T: Serialize>(value: &T) -> bincode::Result<Vec<u8>> {
    bincode::serialize(value)
}

/// The dencoding function for all data. Mostly for internal use, exposed here for debugging
/// potential
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> bincode::Result<T> {
    bincode::deserialize(bytes)
}
