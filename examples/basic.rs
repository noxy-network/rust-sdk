//! Route a decision and print quota.
//!
//! Run:
//!   NOXY_APP_TOKEN=<token> NOXY_IDENTITY_ID=<logical user id> cargo run --example basic

use noxy_sdk::{init_noxy_agent_client, NoxyConfig, NoxyHumanDecisionOutcome};
const NOXY_ENDPOINT: &str = "https://relay.noxy.network";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let auth_token = std::env::var("NOXY_APP_TOKEN").expect("NOXY_APP_TOKEN must be set");
    let identity_id = std::env::var("NOXY_IDENTITY_ID").expect("NOXY_IDENTITY_ID must be set");

    let config = NoxyConfig {
        endpoint: NOXY_ENDPOINT.to_string(),
        auth_token,
        decision_ttl_seconds: 3600,
    };

    let client = init_noxy_agent_client(config).await?;

    let quota = client.get_quota().await?;
    println!(
        "Quota: {} / {} remaining (status: {:?})",
        quota.quota_remaining, quota.quota_total, quota.status
    );

    let actionable = serde_json::json!({
        "kind": "propose_tool_call",
        "tool": "transfer_funds",
        "args": { "to": "0x000000000000000000000000000000000000dEaD", "amountWei": "1" },
        "title": "[RUST] Transfer 1 wei to the burn address",
        "summary": "[RUST] The agent is requesting approval to send 1 wei to the burn address.",
    });

    println!("Routing decision to {}...", identity_id);
    let resolution = client
        .send_decision_and_wait_for_outcome(identity_id, &actionable, None)
        .await?;

    println!(
        "Outcome: {:?}, pending: {}",
        resolution.outcome, resolution.pending
    );
    if resolution.outcome == NoxyHumanDecisionOutcome::Approved {
        println!("User approved — continue agent loop.");
    }

    Ok(())
}
