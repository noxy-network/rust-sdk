//! Type definitions matching `agent.proto` and the SDK API.

/// Relay `identity_id`: logical user identity (wallet `0x…`, email, phone, app `user_id`, …)—must match device registration.
pub type NoxyIdentityId = String;

/// Relay-side delivery status after `RouteDecision` (matches proto `DeliveryStatus`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum NoxyDeliveryStatus {
    Delivered = 0,
    Queued = 1,
    NoDevices = 2,
    Rejected = 3,
    Error = 4,
}

impl From<i32> for NoxyDeliveryStatus {
    fn from(v: i32) -> Self {
        match v {
            0 => Self::Delivered,
            1 => Self::Queued,
            2 => Self::NoDevices,
            3 => Self::Rejected,
            4 => Self::Error,
            _ => Self::Error,
        }
    }
}

/// Response for a single `RouteDecision` delivery to one device.
#[derive(Debug, Clone)]
pub struct NoxyDeliveryOutcome {
    pub status: NoxyDeliveryStatus,
    pub request_id: String,
    /// Present when the relay tracks human resolution; use with `get_decision_outcome` / `wait_for_decision_outcome`.
    pub decision_id: String,
}

/// Human-in-the-loop resolution (matches proto `DecisionOutcome`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum NoxyHumanDecisionOutcome {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Expired = 3,
}

impl From<i32> for NoxyHumanDecisionOutcome {
    fn from(v: i32) -> Self {
        match v {
            0 => Self::Pending,
            1 => Self::Approved,
            2 => Self::Rejected,
            3 => Self::Expired,
            _ => Self::Pending,
        }
    }
}

/// Result of polling `GetDecisionOutcome`.
#[derive(Debug, Clone)]
pub struct NoxyGetDecisionOutcomeResponse {
    pub request_id: String,
    pub pending: bool,
    pub outcome: NoxyHumanDecisionOutcome,
}

/// Quota status for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum NoxyQuotaStatus {
    QuotaActive = 0,
    QuotaSuspended = 1,
    QuotaDeleted = 2,
}

impl From<i32> for NoxyQuotaStatus {
    fn from(v: i32) -> Self {
        match v {
            0 => Self::QuotaActive,
            1 => Self::QuotaSuspended,
            2 => Self::QuotaDeleted,
            _ => Self::QuotaActive,
        }
    }
}

/// Response for quota query.
#[derive(Debug, Clone)]
pub struct NoxyGetQuotaResponse {
    pub request_id: String,
    pub app_name: String,
    pub quota_total: u64,
    pub quota_remaining: u64,
    pub status: NoxyQuotaStatus,
}

/// Identity device with keys for encryption.
#[derive(Debug, Clone)]
pub struct NoxyIdentityDevice {
    pub device_id: String,
    pub public_key: Vec<u8>,
    pub pq_public_key: Vec<u8>,
}
