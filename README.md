# 📦 @noxy-network/rust-sdk

Backend SDK for Rust servers to integrate with the [Noxy](https://noxy.network) push notification network. Send encrypted push notifications to Web3 wallet addresses via the Noxy relay infrastructure.

## Overview

This SDK enables server-side applications to:

- **Send push notifications** to users by their Web3 wallet address (EVM `0x` format)
- **Query quota usage** for your application's relay allocation
- **Resolve identity devices** to deliver notifications to all registered devices

Communication with the Noxy relay is performed over **gRPC** using Protocol Buffers. All notifications are **encrypted end-to-end** on the backend before transmission; decryption occurs only on the recipient's Noxy device. The SDK uses **post-quantum encryption** (Kyber) to protect payloads against future quantum attacks.

## Architecture

```
┌─────────────────┐     gRPC (TLS)      ┌─────────────────┐     E2E Encrypted     ┌─────────────────┐
│  Your Backend   │ ◄─────────────────► │  Noxy Relay     │ ◄──────────────────► │  Noxy Device    │
│  (this SDK)     │   PushNotification  │                 │   Ciphertext only    │  (decrypts)      │
│                 │   GetQuota          │                 │                      │                 │
│                 │   GetIdentityDevices│                 │                      │                 │
└─────────────────┘                     └─────────────────┘                      └─────────────────┘
```

- **Encryption**: Kyber (post-quantum KEM) + AES-256-GCM. Each notification is encrypted per-device using the device's post-quantum public key.
- **Transport**: gRPC over TLS with Bearer token authentication.
- **Relay**: The Noxy relay forwards ciphertext only; it cannot decrypt notification payloads.

## Requirements

- Rust **>= 1.70**
- Tokio async runtime

## 🚀 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
noxy-sdk = "1.0"
```

## 🛠 Quick Start

```rust
use noxy_sdk::{init_noxy_client, NoxyConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = NoxyConfig {
        endpoint: "https://relay.noxy.network".to_string(),
        auth_token: "your-api-token".to_string(),
        notification_ttl_seconds: 3600,
    };

    let client = init_noxy_client(config).await?;

    // Send a push notification to a wallet address
    let results = client
        .send_push(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".into(),
            &serde_json::json!({
                "title": "New message",
                "body": "You have a new notification",
                "data": { "action": "open_chat", "id": "123" }
            }),
        )
        .await?;

    // Check quota usage
    let quota = client.get_quota().await?;
    println!("{} remaining", quota.quota_remaining);

    Ok(())
}
```

## Configuration

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `endpoint` | `String` | Yes | Noxy relay gRPC endpoint (e.g. `https://relay.noxy.network`). Scheme is stripped; TLS is used by default. |
| `auth_token` | `String` | Yes | Bearer token for relay authentication. Sent in the `Authorization` header on every request. |
| `notification_ttl_seconds` | `u32` | Yes | Time-to-live for notifications in seconds. |

## API Reference

### `init_noxy_client(config: NoxyConfig) -> Result<NoxyPushClient, Error>`

Initializes the SDK client. This is asynchronous because it establishes the gRPC connection.

### `NoxyPushClient`

#### `send_push(identity_address, push_notification) -> Result<Vec<NoxyPushResponse>, Error>`

Sends a push notification to all devices registered for the given Web3 identity address.

- **`identity_address`**: EVM address in `0x` format (e.g. `0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1`)
- **`push_notification`**: Any type implementing `serde::Serialize` (e.g. `serde_json::Value`). Encrypted before transmission.
- **Returns**: `Vec<NoxyPushResponse>` per device, with `status` and `request_id`.

#### `get_quota() -> Result<NoxyGetQuotaResponse, Status>`

Returns quota usage for your application.

- **Returns**: `NoxyGetQuotaResponse` with `request_id`, `app_name`, `quota_total`, `quota_remaining`, `status`.

### Types

- **`NoxyPushDeliveryStatus`**: `Delivered` | `Queued` | `NoDevices` | `Rejected` | `Error`
- **`NoxyQuotaStatus`**: `QuotaActive` | `QuotaSuspended` | `QuotaDeleted`

## Encryption Details

1. **Key encapsulation**: For each device, the SDK encapsulates a shared secret using the device's Kyber post-quantum public key (`pq_public_key`).
2. **Key derivation**: The shared secret is expanded via HKDF-SHA256 to a 256-bit AES key.
3. **Payload encryption**: The notification payload (JSON) is encrypted with AES-256-GCM. The ciphertext includes the GCM auth tag appended for integrity verification.
4. **Transmission**: Only `kyber_ct`, `nonce`, and `ciphertext` are sent to the relay. The relay cannot decrypt; only the target device (with its secret key) can decrypt.

## 📄 License

MIT
