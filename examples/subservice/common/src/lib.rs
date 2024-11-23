use framework::Subservice;

/// TLS certificate (self-signed for debug purposes)
pub const CERTIFICATE: &[u8] = include_bytes!("localhost.crt");
pub const CERTIFICATE_HASHES: &[u8] = include_bytes!("localhost.hex");

#[tarpc::service]
pub trait MyService {
    /// Adds numbers
    async fn add(a: u32, b: u32) -> u32;

    /// Returns a sub-service
    async fn get_sub() -> Subservice<MyOtherServiceClient>;
}

#[tarpc::service]
pub trait MyOtherService {
    /// Subtracts numbers
    async fn subtract(a: u32, b: u32) -> u32;
}
