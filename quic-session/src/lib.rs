#[cfg(target_arch = "wasm32")]
pub async fn session() -> web_transport::Session {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn session() -> web_transport::Session {
    todo!()
}
