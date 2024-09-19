use std::marker::PhantomData;

use web_transport::Session;

pub struct Framework {
    pub sess: Session,
}

/// Don't worry about it
#[cfg(target_arch = "wasm32")]
unsafe impl Send for Framework {}

impl Framework {
    pub fn new(sess: Session) -> Self {
        Self { sess }
    }
}

/// Which session, client or server did accept_bi()?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
enum Accepter {
    Client,
    Server,
}

/// Uniquely identifies a stream, and carries type information about its contents.
/// This is the type used to transmit information between client and server about the identity of a
/// connected stream/sink combo. 
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BiStream<ClientToServer, ServerToClient> {
    id: usize,
    end: Acceptor,
    _phantom: PhantomData<(ClientToServer, ServerToClient)>,
}

impl<CTS, STC> BiStream<CTS, STC> {
    pub fn accept(&self, fr: &mut Framework) -> Box<dyn Transport> {
    }
}
