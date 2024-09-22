use futures::Future;

pub use futures;
pub use serde;
pub use tarpc;

use web_transport::Session;

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
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async {
        future.await;
    });
}

/// Consumes the given future returning an anyhow::Result<()> and transforms it into one which logs
/// the error.
pub fn log_error<F>(f: F) -> impl Future<Output = ()>
where
    F: Future<Output = anyhow::Result<()>>,
{
    async {
        if let Err(e) = f.await {
            log::error!("{:#}", e);
        }
    }
}
