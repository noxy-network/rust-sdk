use crate::crypto;
use crate::kyber_provider::KyberProvider;
use crate::retries;
use crate::transport::{proto, PushServiceClient};
use crate::types::{NoxyIdentityDevice, NoxyPushDeliveryStatus, NoxyPushResponse};
use tonic::metadata::AsciiMetadataValue;
use tonic::Request;
use tonic::transport::Channel;
use uuid::Uuid;

pub(crate) struct PushService {
    kyber_provider: KyberProvider,
}

impl PushService {
    pub(crate) fn new(kyber_provider: KyberProvider) -> Self {
        Self { kyber_provider }
    }

    fn encrypt_notification(
        &self,
        device_pq_public_key: &[u8],
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        let (kyber_ct, shared_secret) = self
            .kyber_provider
            .encapsulate(device_pq_public_key)
            .map_err(|e| format!("Kyber encapsulate: {}", e))?;
        let (ciphertext, nonce) = crypto::encrypt(&shared_secret, plaintext)
            .map_err(|e| format!("AES-GCM encrypt: {}", e))?;
        Ok((kyber_ct, nonce, ciphertext))
    }

    async fn send_to_network(
        &self,
        client: &mut PushServiceClient<Channel>,
        ciphertext: Vec<u8>,
        ttl_seconds: u32,
        target_device_id: &str,
        kyber_ct: Vec<u8>,
        nonce: Vec<u8>,
        auth_value: &AsciiMetadataValue,
    ) -> Result<NoxyPushResponse, tonic::Status> {
        let req = proto::PushNotificationRequest {
            request_id: Uuid::new_v4().to_string(),
            ciphertext,
            ttl_seconds,
            target_device_id: target_device_id.to_string(),
            kyber_ct,
            nonce,
        };
        let mut last_err = None;
        for attempt in 0..3 {
            let mut request = Request::new(req.clone());
            request.metadata_mut().insert("authorization", auth_value.clone());
            match client.push_notification(request).await
            {
                Ok(resp) => {
                    let response = resp.into_inner();
                    return Ok(NoxyPushResponse {
                        status: NoxyPushDeliveryStatus::from(response.status),
                        request_id: response.request_id,
                    });
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt < 2 && retries::is_retryable(last_err.as_ref().unwrap()) {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            100 * 2u64.pow(attempt),
                        ))
                        .await;
                    } else {
                        break;
                    }
                }
            }
        }
        Err(last_err.unwrap())
    }

    pub(crate) async fn send<T>(
        &self,
        client: &mut PushServiceClient<Channel>,
        devices: &[NoxyIdentityDevice],
        push_notification: &T,
        ttl_seconds: u32,
        auth_value: &AsciiMetadataValue,
    ) -> Result<Vec<NoxyPushResponse>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let plaintext = serde_json::to_vec(push_notification)?;
        let mut results = Vec::with_capacity(devices.len());
        for device in devices {
            let (kyber_ct, nonce, ciphertext) =
                self.encrypt_notification(&device.pq_public_key, &plaintext)?;
            let response = self
                .send_to_network(
                    client,
                    ciphertext,
                    ttl_seconds,
                    &device.device_id,
                    kyber_ct,
                    nonce,
                    auth_value,
                )
                .await
                .map_err(|e| format!("gRPC PushNotification: {}", e))?;
            results.push(response);
        }
        Ok(results)
    }
}
