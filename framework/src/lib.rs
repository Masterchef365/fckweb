use std::{marker::PhantomData, sync::Arc};

pub use futures;
use io::FrameworkError;
pub use serde;
use serde::{de::DeserializeOwned, Serialize};
pub use tarpc;

use tarpc::{client::stub::Stub, Transport};
use web_transport::Session;

pub mod io;

// NOTE: Doesn't implement Clone, since we want to this to be consumed
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Subservice<Client> {
    _phantom: PhantomData<Client>,
}

#[derive(Clone)]
pub struct ClientFramework {
    // Ensures each open() occurs in sequence with each accept(). We don't open() until the last
    // one was either completed or failed!
    pub seq: Arc<futures::lock::Mutex<Session>>,
}

/// Don't worry about it
#[cfg(target_arch = "wasm32")]
unsafe impl Send for ClientFramework {}

impl ClientFramework {
    /// Creates a new framework, and offers a root transport
    pub async fn new<Rx: DeserializeOwned, Tx: Serialize>(
        mut sess: Session,
    ) -> Result<(Self, impl Transport<Tx, Rx, Error = FrameworkError>), FrameworkError>
    {
        let socks = sess.open_bi().await?;
        let channel = crate::io::webtransport_protocol(socks);
        let inst = Self::new_internal(sess);
        Ok((inst, channel))
    }

    fn new_internal(sess: Session) -> Self {
        Self {
            seq: Arc::new(futures::lock::Mutex::new(sess)),
        }
    }

    // TODO: Typecheck that Client's types match Rx/Tx!!
    pub async fn connect_subservice<Rx: DeserializeOwned, Tx: Serialize, Client>(
        &self,
        _token: Subservice<Client>,
    ) -> Result<impl Transport<Tx, Rx, Error = FrameworkError>, FrameworkError>
    where
        //Client: Stub<Req = Tx, Resp = Rx>,
    {
        // Holds the lock only while we are opening the stream
        let socks = {
            let mut sess = self.seq.lock().await;
            sess.open_bi().await?
        };

        Ok(crate::io::webtransport_protocol(socks))
    }
}


#[derive(Clone)]
pub struct ServerFramework {
    // Ensures each open() occurs in sequence with each accept(). We don't open() until the last
    // one was either completed or failed!
    pub seq: Arc<futures::lock::Mutex<Session>>,
}

/// Don't worry about it
#[cfg(target_arch = "wasm32")]
unsafe impl Send for ClientFramework {}

impl ServerFramework {
    /// Creates a new framework, and offers a root transport
    pub async fn new<Rx: DeserializeOwned, Tx: Serialize>(
        mut sess: Session,
    ) -> Result<(Self, impl Transport<Tx, Rx, Error = FrameworkError>), FrameworkError>
    {
        let socks = sess.accept_bi().await?;
        let channel = crate::io::webtransport_protocol(socks);
        let inst = Self::new_internal(sess);
        Ok((inst, channel))
    }

    fn new_internal(sess: Session) -> Self {
        Self {
            seq: Arc::new(futures::lock::Mutex::new(sess)),
        }
    }

    // TODO: Typecheck that Client's types match Rx/Tx!!
    pub async fn accept_subservice<Rx: DeserializeOwned, Tx: Serialize, Client>(
        &self,
    ) -> Result<(Subservice<Client>, impl Transport<Tx, Rx, Error = FrameworkError>), FrameworkError>
    where
        //Client: Stub<Req = Tx, Resp = Rx>,
    {
        // Holds the lock only while we are opening the stream
        let socks = {
            let mut sess = self.seq.lock().await;
            sess.accept_bi().await?
        };

        let channel = crate::io::webtransport_protocol(socks);

        let sub = Subservice { _phantom: PhantomData };

        Ok((sub, channel))
    }
}
