mod spawner;
pub use poll_promise::Promise;
pub use spawner::{SimpleSpawner, SpawnerState};
pub use std::future::Future;

#[cfg(target_arch = "wasm32")]
pub fn spawn_promise<F>(fut: F) -> Promise<F::Output>
where
    F: Future + 'static,
    F::Output: Send,
{
    Promise::spawn_local(fut)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_promise<F>(fut: F) -> Promise<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    Promise::spawn_async(fut)
}
