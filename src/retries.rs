//! Retry logic for transient network errors.

use std::time::Duration;
use tonic::Status;

/// gRPC status codes that indicate retryable errors.
const RETRYABLE_CODES: [i32; 4] = [2, 4, 8, 14]; // UNKNOWN, DEADLINE_EXCEEDED, RESOURCE_EXHAUSTED, UNAVAILABLE

/// Check if an error is retryable (network/unavailable).
pub fn is_retryable(err: &(dyn std::error::Error + 'static)) -> bool {
    if let Some(status) = err.downcast_ref::<Status>() {
        let code = status.code() as i32;
        return RETRYABLE_CODES.contains(&code);
    }
    false
}

/// Execute a future with retries on retryable errors.
pub async fn with_retry<F, Fut, T, E>(mut f: F, retries: u32) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut last_err = None;
    for i in 0..retries {
        match f().await {
            Ok(t) => return Ok(t),
            Err(e) => {
                last_err = Some(e);
                if i < retries - 1 && is_retryable(last_err.as_ref().unwrap()) {
                    tokio::time::sleep(Duration::from_millis(2u64.pow(i) * 100)).await;
                } else {
                    break;
                }
            }
        }
    }
    Err(last_err.unwrap())
}
