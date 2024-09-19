use web_transport::Session;

pub struct Framework {
    pub sess: Session,
}

/// Don't worry about it
#[cfg(target_arch = "wasm32")]
unsafe impl Send for Framework {}

impl Framework {
    pub fn new(sess: Session) -> Self {
        Self {
            sess
        }
    }
}

