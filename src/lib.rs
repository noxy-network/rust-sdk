//! SDK for **AI agent runtimes** integrating with the [Noxy](https://noxy.network) **Decision Layer**:
//! send encrypted, **actionable** decision payloads (tool proposals, approvals, next-step hints) to
//! registered agent devices over gRPC.
//!
//! The wire API is **`agent.proto`** (`noxy.agent.AgentService`): `RouteDecision`, `GetDecisionOutcome`,
//! `GetQuota`, `GetIdentityDevices`.

pub mod config;
pub mod decision_outcome;
pub mod kyber_provider;
pub mod retries;
pub mod transport;
pub mod types;

mod client;
mod crypto;
mod services;

pub use client::NoxyAgentClient;
pub use config::NoxyConfig;
pub use decision_outcome::{
    is_terminal_human_outcome, SendDecisionAndWaitNoDecisionIdError, SendDecisionAndWaitOptions,
    WaitForDecisionOutcomeOptions, WaitForDecisionOutcomeTimeoutError,
};
pub use types::{
    NoxyDeliveryOutcome, NoxyDeliveryStatus, NoxyGetDecisionOutcomeResponse, NoxyGetQuotaResponse,
    NoxyHumanDecisionOutcome, NoxyIdentityDevice, NoxyQuotaStatus,
};

/// Initialize the Noxy Decision Layer client (async: establishes the gRPC connection).
pub async fn init_noxy_agent_client(
    config: NoxyConfig,
) -> Result<NoxyAgentClient, Box<dyn std::error::Error + Send + Sync>> {
    let endpoint = normalize_endpoint(&config.endpoint);
    let channel = tonic::transport::Endpoint::from_shared(format!("https://{}", endpoint))?
        .tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())?
        .connect()
        .await?;
    let kyber_provider = kyber_provider::KyberProvider::new();
    let client = NoxyAgentClient::new(config, channel, kyber_provider);
    Ok(client)
}

fn normalize_endpoint(endpoint: &str) -> String {
    endpoint
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}
