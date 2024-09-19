use std::marker::PhantomData;

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

/// Uniquely identifies a stream, and carries type information about its contents.
/// This is the type used to transmit information between client and server about the identity of a
/// connected stream/sink combo.
///
/// This is a type you should return from your API, in order to get a bidirectional stream on the other end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BiStream<ClientToServer, ServerToClient> {
    id: usize,
    _phantom: PhantomData<(ClientToServer, ServerToClient)>,
}

impl<CTS, STC> BiStream<CTS, STC> {
    pub async fn accept(&self, fr: &mut Framework) -> Box<dyn Stream<CTS> + Sink<STC>> {
        todo!()
    }
}
