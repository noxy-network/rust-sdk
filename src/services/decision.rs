use crate::crypto;
use crate::kyber_provider::KyberProvider;
use crate::retries;
use crate::transport::{proto, AgentServiceClient};
use crate::types::{NoxyDeliveryOutcome, NoxyDeliveryStatus, NoxyIdentityDevice};
use tonic::metadata::AsciiMetadataValue;
use tonic::Request;
use tonic::transport::Channel;
use uuid::Uuid;

pub(crate) struct DecisionService {
    kyber_provider: KyberProvider,
}

impl DecisionService {
    pub(crate) fn new(kyber_provider: KyberProvider) -> Self {
        Self { kyber_provider }
    }

    fn encrypt_decision(
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
        client: &mut AgentServiceClient<Channel>,
        ciphertext: Vec<u8>,
        ttl_seconds: u32,
        target_device_id: &str,
        kyber_ct: Vec<u8>,
        nonce: Vec<u8>,
        decision_id: String,
        auth_value: &AsciiMetadataValue,
    ) -> Result<NoxyDeliveryOutcome, tonic::Status> {
        let req = proto::RouteDecisionRequest {
            request_id: Uuid::new_v4().to_string(),
            ciphertext,
            ttl_seconds,
            target_device_id: target_device_id.to_string(),
            kyber_ct,
            nonce,
            decision_id: decision_id.clone(),
        };
        let mut last_err = None;
        for attempt in 0..3 {
            let mut request = Request::new(req.clone());
            request.metadata_mut().insert("authorization", auth_value.clone());
            match client.route_decision(request).await {
                Ok(resp) => {
                    let response = resp.into_inner();
                    let mut out_decision_id = response.decision_id;
                    if out_decision_id.is_empty() {
                        out_decision_id = decision_id.clone();
                    }
                    return Ok(NoxyDeliveryOutcome {
                        status: NoxyDeliveryStatus::from(response.status),
                        request_id: response.request_id,
                        decision_id: out_decision_id,
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

    /// One `decision_id` (UUID) per batch, reused for every device in the fan-out.
    pub(crate) async fn send<T>(
        &self,
        client: &mut AgentServiceClient<Channel>,
        devices: &[NoxyIdentityDevice],
        actionable_decision: &T,
        ttl_seconds: u32,
        auth_value: &AsciiMetadataValue,
    ) -> Result<Vec<NoxyDeliveryOutcome>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let plaintext = serde_json::to_vec(actionable_decision)?;
        let decision_id = Uuid::new_v4().to_string();
        let mut results = Vec::with_capacity(devices.len());
        for device in devices {
            let (kyber_ct, nonce, ciphertext) =
                self.encrypt_decision(&device.pq_public_key, &plaintext)?;
            let response = self
                .send_to_network(
                    client,
                    ciphertext,
                    ttl_seconds,
                    &device.device_id,
                    kyber_ct,
                    nonce,
                    decision_id.clone(),
                    auth_value,
                )
                .await
                .map_err(|e| format!("gRPC RouteDecision: {}", e))?;
            results.push(response);
        }
        Ok(results)
    }
}
