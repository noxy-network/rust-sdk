use crate::config::NoxyConfig;
use crate::decision_outcome::{
    SendDecisionAndWaitNoDecisionIdError, SendDecisionAndWaitOptions, WaitForDecisionOutcomeOptions,
    WaitForDecisionOutcomeTimeoutError,
};
use crate::kyber_provider::KyberProvider;
use crate::services::{DecisionService, IdentityService, QuotaService};
use crate::transport::proto;
use crate::transport::AgentServiceClient;
use crate::types::{
    NoxyDeliveryOutcome, NoxyGetDecisionOutcomeResponse, NoxyGetQuotaResponse, NoxyHumanDecisionOutcome,
    NoxyIdentityAddress,
};
use tonic::metadata::AsciiMetadataValue;
use tonic::transport::Channel;
use tonic::Request;
use uuid::Uuid;

pub struct NoxyAgentClient {
    config: NoxyConfig,
    channel: Channel,
    auth_value: AsciiMetadataValue,
    decision_service: DecisionService,
    identity_service: IdentityService,
    quota_service: QuotaService,
}

impl NoxyAgentClient {
    pub(crate) fn new(
        config: NoxyConfig,
        channel: Channel,
        kyber_provider: KyberProvider,
    ) -> Self {
        let auth_value: AsciiMetadataValue = format!("Bearer {}", config.auth_token)
            .parse()
            .expect("valid auth token");
        let decision_service = DecisionService::new(kyber_provider);
        let identity_service = IdentityService::new();
        let quota_service = QuotaService::new();
        Self {
            config,
            channel,
            auth_value,
            decision_service,
            identity_service,
            quota_service,
        }
    }

    fn create_client(&self) -> AgentServiceClient<Channel> {
        AgentServiceClient::new(self.channel.clone())
    }

    /// Route an actionable decision to all devices registered for the identity.
    /// Uses one client-generated `decision_id` (UUID) for the whole batch so every device shares the same logical decision.
    pub async fn send_decision<T>(
        &self,
        identity_address: NoxyIdentityAddress,
        actionable_decision: &T,
    ) -> Result<Vec<NoxyDeliveryOutcome>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let mut client = self.create_client();
        let devices = self
            .identity_service
            .get_devices(&mut client, &identity_address, &self.auth_value)
            .await?;
        self.decision_service
            .send(
                &mut client,
                &devices,
                actionable_decision,
                self.config.decision_ttl_seconds,
                &self.auth_value,
            )
            .await
    }

    /// Single poll for human-in-the-loop resolution.
    pub async fn get_decision_outcome(
        &self,
        decision_id: &str,
        identity_id: &str,
    ) -> Result<NoxyGetDecisionOutcomeResponse, tonic::Status> {
        let mut client = self.create_client();
        let req = proto::GetDecisionOutcomeRequest {
            request_id: Uuid::new_v4().to_string(),
            decision_id: decision_id.to_string(),
            identity_id: identity_id.to_string(),
        };
        let mut request = Request::new(req);
        request
            .metadata_mut()
            .insert("authorization", self.auth_value.clone());
        let response = client.get_decision_outcome(request).await?.into_inner();
        Ok(NoxyGetDecisionOutcomeResponse {
            request_id: response.request_id,
            pending: response.pending,
            outcome: NoxyHumanDecisionOutcome::from(response.outcome),
        })
    }

    /// Poll `GetDecisionOutcome` with exponential backoff until terminal outcome or `pending == false`.
    pub async fn wait_for_decision_outcome(
        &self,
        options: WaitForDecisionOutcomeOptions,
    ) -> Result<NoxyGetDecisionOutcomeResponse, Box<dyn std::error::Error + Send + Sync>> {
        let initial_poll_interval_ms = options.initial_poll_interval_ms.unwrap_or(400);
        let max_poll_interval_ms = options.max_poll_interval_ms.unwrap_or(30_000);
        let max_wait_ms = options.max_wait_ms.unwrap_or(900_000);
        let backoff_multiplier = options.backoff_multiplier.unwrap_or(1.6);

        let started = std::time::Instant::now();
        let mut interval = initial_poll_interval_ms;
        let decision_id = options.decision_id.clone();
        let identity_id = options.identity_id.clone();

        loop {
            if started.elapsed().as_millis() as u64 > max_wait_ms {
                return Err(Box::new(WaitForDecisionOutcomeTimeoutError));
            }

            let raw = self
                .get_decision_outcome(&decision_id, &identity_id)
                .await
                .map_err(|e| format!("GetDecisionOutcome: {}", e))?;

            if crate::decision_outcome::is_terminal_human_outcome(raw.outcome) {
                return Ok(raw);
            }
            if !raw.pending {
                return Ok(raw);
            }

            tokio::time::sleep(std::time::Duration::from_millis(
                interval.min(max_poll_interval_ms),
            ))
            .await;
            interval = ((interval as f64 * backoff_multiplier) as u64).min(max_poll_interval_ms);
        }
    }

    /// [`send_decision`] then [`wait_for_decision_outcome`] using the first delivery with a non-empty `decision_id`.
    pub async fn send_decision_and_wait_for_outcome<T>(
        &self,
        identity_address: NoxyIdentityAddress,
        actionable_decision: &T,
        options: Option<SendDecisionAndWaitOptions>,
    ) -> Result<NoxyGetDecisionOutcomeResponse, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let deliveries = self
            .send_decision(identity_address.clone(), actionable_decision)
            .await?;
        let with_id = deliveries.iter().find(|d| !d.decision_id.is_empty());
        let Some(d) = with_id else {
            return Err(Box::new(SendDecisionAndWaitNoDecisionIdError));
        };
        let o = options.unwrap_or_default();
        self.wait_for_decision_outcome(WaitForDecisionOutcomeOptions {
            decision_id: d.decision_id.clone(),
            identity_id: identity_address,
            initial_poll_interval_ms: o.initial_poll_interval_ms,
            max_poll_interval_ms: o.max_poll_interval_ms,
            max_wait_ms: o.max_wait_ms,
            backoff_multiplier: o.backoff_multiplier,
        })
        .await
    }

    pub async fn get_quota(&self) -> Result<NoxyGetQuotaResponse, tonic::Status> {
        let mut client = self.create_client();
        self.quota_service
            .get(&mut client, &self.auth_value)
            .await
    }
}
