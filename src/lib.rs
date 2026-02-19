//! Noxy SDK - Backend SDK for Rust servers to integrate with the Noxy push notification network.
//!
//! Send encrypted push notifications to Web3 wallet addresses via the Noxy relay.

pub mod config;
pub mod kyber_provider;
pub mod retries;
pub mod transport;
pub mod types;

mod client;
mod crypto;
mod services;

pub use client::NoxyPushClient;
pub use config::NoxyConfig;
pub use types::{
    NoxyGetQuotaResponse, NoxyIdentityDevice, NoxyPushDeliveryStatus, NoxyPushResponse,
    NoxyQuotaStatus,
};


/// Initialize the Noxy client. This is async because it establishes the gRPC connection.
pub async fn init_noxy_client(config: NoxyConfig) -> Result<NoxyPushClient, Box<dyn std::error::Error + Send + Sync>> {
    let endpoint = normalize_endpoint(&config.endpoint);
    let channel = tonic::transport::Endpoint::from_shared(format!("https://{}", endpoint))?
        .tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())?
        .connect()
        .await?;
    let kyber_provider = kyber_provider::KyberProvider::new();
    let client = NoxyPushClient::new(config, channel, kyber_provider);
    Ok(client)
}

fn normalize_endpoint(endpoint: &str) -> String {
    endpoint
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}
