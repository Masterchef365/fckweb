use futures::Future;

pub use futures;
pub use serde;
pub use tarpc;

use bytes::Bytes;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::convert::Infallible;
use std::{marker::PhantomData, sync::Arc, task::Poll};
use tarpc::{transport::channel::UnboundedChannel, Transport};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream, ReadHalf, SimplexStream, WriteHalf};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};

use futures::{AsyncRead, Sink, SinkExt, Stream, StreamExt};
use web_transport::{RecvStream, SendStream, Session};

pub mod io;

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

// NOTE: These have different trait requirements. This is an unfortunate consequence of
// incompatability of many libraries we are using between WASM and native, papered over by these functions.

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future);
}
