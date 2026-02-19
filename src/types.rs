//! Type definitions matching the proto and SDK API.


/// EVM wallet address in 0x format.
pub type NoxyIdentityAddress = String;

/// Delivery status for a push notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum NoxyPushDeliveryStatus {
    Delivered = 0,
    Queued = 1,
    NoDevices = 2,
    Rejected = 3,
    Error = 4,
}

impl From<i32> for NoxyPushDeliveryStatus {
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

/// Response for a push notification send.
#[derive(Debug, Clone)]
pub struct NoxyPushResponse {
    pub status: NoxyPushDeliveryStatus,
    pub request_id: String,
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
