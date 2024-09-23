use std::{marker::PhantomData, sync::Arc};

pub use futures;
use futures::{Sink, Stream};
use io::FrameworkError;
pub use serde;
use serde::{de::DeserializeOwned, Serialize};
pub use tarpc;

use tarpc::{
    client::{NewClient, RequestDispatch},
    Transport,
};
use web_transport::Session;

pub mod io;

#[derive(Clone)]
pub struct Framework {
    // Ensures each open() occurs in sequence with each accept(). We don't open() until the last
    // one was either completed or failed!
    pub seq: Arc<futures::lock::Mutex<Session>>,
}

/// Don't worry about it
#[cfg(target_arch = "wasm32")]
unsafe impl Send for Framework {}

impl Framework {
    pub fn new(sess: Session) -> Self {
        Self {
            seq: Arc::new(futures::lock::Mutex::new(sess)),
        }
    }

    /*
    pub fn get_next<Request, Response>(&mut self) -> Transporter<Request, Response> {
        todo!()
    }
    */

    // TODO: Typecheck that Client's types match Rx/Tx!!
    pub async fn connect_subservice<Rx: DeserializeOwned, Tx: Serialize, Client>(
        &self,
        _token: Subservice<Client>,
    ) -> Result<impl Transport<Tx, Rx, Error = FrameworkError>, FrameworkError> {
        // Holds the lock only while we are opening the stream
        let socks = {
            let mut sess = self.seq.lock().await;
            sess.open_bi().await?
        };

        Ok(crate::io::webtransport_protocol(socks))
    }
}

// NOTE: Doesn't implement Clone, since we want to this to be consumed
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Subservice<Client> {
    _phantom: PhantomData<Client>,
}

/*
trait MyStuff<Request, Response>: Sink<Request, Error = RpcError> + Stream<Item = Response> {}

type Transporter<Request, Response> = Box<dyn MyStuff<Request, Response>>;

impl<Client> Subservice<Client> {
    /// Here 'F' is the Client::new function, which (because tarpc is dumb) isn't part of a trait.
    pub fn connect<F, Request, Response>(self, frame: &mut Framework, f: F) -> Client
    where
        Request: Send,
        Response: Send,
        F: FnOnce(
            tarpc::client::Config,
            Transporter<Request, Response>,
        ) -> NewClient<
            Self,
            RequestDispatch<Request, Response, Transporter<Request, Response>>,
        >
    {
        // sue me for using the default here
        let nc = f(tarpc::client::Config::default(), frame.get_next());
        tokio::spawn(nc.dispatch);
        nc.client
    }
}
*/
