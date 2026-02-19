use crate::transport::{proto, PushServiceClient};
use crate::types::{NoxyGetQuotaResponse, NoxyQuotaStatus};
use tonic::metadata::AsciiMetadataValue;
use tonic::Request;
use tonic::transport::Channel;
use uuid::Uuid;

pub(crate) struct QuotaService;

impl QuotaService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn get(
        &self,
        client: &mut PushServiceClient<Channel>,
        auth_value: &AsciiMetadataValue,
    ) -> Result<NoxyGetQuotaResponse, tonic::Status> {
        let req = proto::GetQuotaRequest {
            request_id: Uuid::new_v4().to_string(),
        };
        let mut request = Request::new(req);
        request.metadata_mut().insert("authorization", auth_value.clone());
        let response = client.get_quota(request).await?.into_inner();
        Ok(NoxyGetQuotaResponse {
            request_id: response.request_id,
            app_name: response.app_name,
            quota_total: response.quota_total,
            quota_remaining: response.quota_remaining,
            status: NoxyQuotaStatus::from(response.status),
        })
    }
}
