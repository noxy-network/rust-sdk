/// Configuration for the Noxy SDK client.
#[derive(Clone, Debug)]
pub struct NoxyConfig {
    /// Noxy relay gRPC endpoint (e.g. `https://relay.noxy.network:443`).
    pub endpoint: String,
    /// Bearer token for relay authentication.
    pub auth_token: String,
    /// Time-to-live for notifications in seconds.
    pub notification_ttl_seconds: u32,
}
