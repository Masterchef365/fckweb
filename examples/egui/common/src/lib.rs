#[tarpc::service]
pub trait MyService {
    /// Returns a greeting for name.
    async fn add(a: u32, b: u32) -> u32;
}
