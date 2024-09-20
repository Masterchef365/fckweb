pub use tarpc;
pub use serde;
use std::marker::PhantomData;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

use futures::{Sink, Stream};
use web_transport::Session;

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

/// This is the type used to provide connectivity to an alternate tarpc connection
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TarpcBiStream(BiStream);


pub fn encode<T: Serialize>(value: &T) -> bincode::Result<Vec<u8>> {
    bincode::serialize(value)
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> bincode::Result<T> {
    bincode::deserialize(bytes)
}
