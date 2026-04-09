# 📦 @noxy-network/rust-sdk

SDK for **AI agent runtimes** integrating with the [Noxy](https://noxy.network) **Decision Layer**: send encrypted, **actionable** decision payloads (tool proposals, approvals, next-step hints) to registered agent devices over gRPC.

**Before you integrate:** Create your app at [noxy.network](https://noxy.network). When the app is created, you receive an **app id** and an **app token** (auth token). This Rust SDK authenticates with the relay using the **app token** (`auth_token` in `NoxyConfig`). The **app id** is used by client SDKs (browser, iOS, Android, Telegram bot), not as the bearer token here.

## Overview

Use this SDK to:

- **Route decisions** to devices bound to a Web3 identity (`0x…` address) — structured JSON you define (e.g. proposed tool calls, parameters, user-visible summaries).
- **Receive delivery outcomes** from the relay (`DELIVERED`, `QUEUED`, `NO_DEVICES`, etc.) plus a **`decision_id`** when the relay accepts the route.
- **Wait for human-in-the-loop resolution** — the wallet user **approves**, **rejects**, or the decision **expires**. The usual path is **`send_decision_and_wait_for_outcome`** (route + poll in one step). Use `get_decision_outcome` / `wait_for_decision_outcome` alone for finer control.
- **Query quota** for your agent application on the relay.
- **Resolve identity devices** so each device receives its own encrypted copy of the decision.

The wire API uses **`agent.proto`** (`noxy.agent.AgentService`): `RouteDecision`, `GetDecisionOutcome`, `GetQuota`, `GetIdentityDevices`.

Communication is **gRPC over TLS** with Bearer authentication. Payloads are **encrypted end-to-end** (Kyber + AES-256-GCM) per device before leaving your process; the relay sees ciphertext only.

## Architecture

The **encrypted path** covers **SDK → relay** and **relay → device**: decision content is ciphertext on both hops; the relay forwards without decrypting.

```
                      Ciphertext only (E2E)                  Ciphertext only (E2E)
┌──────────────────┐     gRPC (TLS)      ┌─────────────────┐     gRPC (TLS)       ┌──────────────────┐
│  AI agent /      │ ◄─────────────────► │  Noxy relay     │ ◄──────────────────► │  Agent device    │
│  orchestrator    │   RouteDecision     │  (Decision      │                      │  (human approves │
│  (this SDK)      │   GetDecisionOutcome│   Layer)        │                      │   or rejects)    │
│                  │   GetQuota          │   forwards only │                      │   decrypts       │
│                  │   GetIdentityDevices│                 │                      │                  │
└──────────────────┘                     └─────────────────┘                      └──────────────────┘
```

## Requirements

- Rust **>= 1.70**
- Tokio async runtime

## Installation

```toml
[dependencies]
noxy-sdk = "2.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
```

## Quick start

Route a decision and wait until the user **approves**, **rejects**, or the decision **expires** (one call):

```rust
use noxy_sdk::{init_noxy_agent_client, NoxyConfig, NoxyHumanDecisionOutcome};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = init_noxy_agent_client(NoxyConfig {
        endpoint: "https://relay.noxy.network".into(),
        auth_token: "your-api-token".into(),
        decision_ttl_seconds: 3600,
    })
    .await?;

    let identity = "0x...".to_string();
    let resolution = client
        .send_decision_and_wait_for_outcome(
            identity,
            &serde_json::json!({
                "kind": "propose_tool_call",
                "tool": "transfer_funds",
                "args": { "to": "0x000000000000000000000000000000000000dEaD", "amountWei": "1" },
                "title": "Transfer 1 wei to the burn address",
                "summary": "The agent is requesting approval to send 1 wei to the burn address.",
            }),
            None,
        )
        .await?;

    if resolution.outcome == NoxyHumanDecisionOutcome::Approved {
        // run the proposed action
    }
    Ok(())
}
```

`resolution.outcome` is a **`NoxyHumanDecisionOutcome`**. **`Approved`** → continue; anything else → stop or fallback. Use **`is_terminal_human_outcome(outcome)`** for “finalized vs still pending”; for one-off **`get_decision_outcome`** polls, also read **`pending`**.

## Configuration

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `endpoint` | `String` | Yes | Relay gRPC endpoint (e.g. `https://relay.noxy.network`). `https://` is stripped; TLS is used. |
| `auth_token` | `String` | Yes | Bearer token for relay auth (`Authorization` header). |
| `decision_ttl_seconds` | `u32` | Yes | TTL for routed decisions (seconds). |

## SendDecisionAndWaitOptions

Optional third argument to **`send_decision_and_wait_for_outcome`**.

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `initial_poll_interval_ms` | `Option<u64>` | No | Delay after the first poll before the next attempt (ms). Default `400`. |
| `max_poll_interval_ms` | `Option<u64>` | No | Maximum delay between polls (ms). Default `30000`. |
| `max_wait_ms` | `Option<u64>` | No | Total time budget for polling (ms). Default `900000` (15 minutes). Exceeded → `WaitForDecisionOutcomeTimeoutError`. |
| `backoff_multiplier` | `Option<f64>` | No | Multiplier applied to the poll interval after each attempt. Default `1.6`. |

## API

### `init_noxy_agent_client(config) -> Result<NoxyAgentClient, Error>`

Async init (establishes gRPC + Kyber for post-quantum encapsulation).

### `NoxyAgentClient`

#### `send_decision(identity_address, actionable_decision)`

Routes an encrypted decision to every device registered for the identity.

- **Returns** per device: relay **`status`**, **`request_id`**, and **`decision_id`** when applicable.

#### `get_decision_outcome(decision_id, identity_id)`

Single poll for human-in-the-loop state (`pending` + `outcome`).

#### `send_decision_and_wait_for_outcome(identity_address, actionable_decision, options?)`

Runs `send_decision`, then `wait_for_decision_outcome` using the **first** delivery with a non-empty `decision_id`. Polling uses `identity_address` as `identity_id`.

- **Returns** `NoxyGetDecisionOutcomeResponse`. Errors with `SendDecisionAndWaitNoDecisionIdError` if no `decision_id` was returned (boxed `Error`).

#### `wait_for_decision_outcome(options)`

Polls with exponential backoff until terminal outcome, `pending == false`, or **`max_wait_ms`** → `WaitForDecisionOutcomeTimeoutError`.

#### `get_quota()`

Quota usage for the application.

### Helpers (re-exported)

- `is_terminal_human_outcome`
- `WaitForDecisionOutcomeTimeoutError`, `SendDecisionAndWaitNoDecisionIdError`

### Types

- **`NoxyDeliveryStatus`**: `Delivered` | `Queued` | `NoDevices` | `Rejected` | `Error`
- **`NoxyHumanDecisionOutcome`**: `Pending` | `Approved` | `Rejected` | `Expired`
- **`NoxyQuotaStatus`**: `QuotaActive` | `QuotaSuspended` | `QuotaDeleted`

## Encryption (summary)

1. Kyber768 encapsulation per device `pq_public_key`.
2. HKDF-SHA256 → AES-256-GCM key; random 12-byte nonce.
3. JSON payload encrypted; only `kyber_ct`, `nonce`, `ciphertext` cross the relay.

## License

MIT
