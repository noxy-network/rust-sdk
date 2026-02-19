use crate::config::NoxyConfig;
use crate::kyber_provider::KyberProvider;
use crate::services::{IdentityService, PushService, QuotaService};
use crate::transport::PushServiceClient;
use crate::types::{NoxyGetQuotaResponse, NoxyIdentityAddress, NoxyPushResponse};
use tonic::metadata::AsciiMetadataValue;
use tonic::transport::Channel;

pub struct NoxyPushClient {
    config: NoxyConfig,
    channel: Channel,
    auth_value: AsciiMetadataValue,
    push_service: PushService,
    identity_service: IdentityService,
    quota_service: QuotaService,
}

impl NoxyPushClient {
    pub(crate) fn new(
        config: NoxyConfig,
        channel: Channel,
        kyber_provider: KyberProvider,
    ) -> Self {
        let auth_value: AsciiMetadataValue = format!("Bearer {}", config.auth_token)
            .parse()
            .expect("valid auth token");
        let push_service = PushService::new(kyber_provider);
        let identity_service = IdentityService::new();
        let quota_service = QuotaService::new();
        Self {
            config,
            channel,
            auth_value,
            push_service,
            identity_service,
            quota_service,
        }
    }

    fn create_client(&self) -> PushServiceClient<Channel> {
        PushServiceClient::new(self.channel.clone())
    }

    pub async fn send_push<T>(
        &self,
        identity_address: NoxyIdentityAddress,
        push_notification: &T,
    ) -> Result<Vec<NoxyPushResponse>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let mut client = self.create_client();
        let devices = self
            .identity_service
            .get_devices(&mut client, &identity_address, &self.auth_value)
            .await?;
        self.push_service
            .send(
                &mut client,
                &devices,
                push_notification,
                self.config.notification_ttl_seconds,
                &self.auth_value,
            )
            .await
    }

    pub async fn get_quota(&self) -> Result<NoxyGetQuotaResponse, tonic::Status> {
        let mut client = self.create_client();
        self.quota_service
            .get(&mut client, &self.auth_value)
            .await
    }
}
