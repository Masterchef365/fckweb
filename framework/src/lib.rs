pub use serde;
pub use tarpc;
pub use futures;


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
