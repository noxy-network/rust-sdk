use crate::transport::{proto, AgentServiceClient};
use crate::types::NoxyIdentityDevice;
use tonic::metadata::AsciiMetadataValue;
use tonic::Request;
use tonic::transport::Channel;
use uuid::Uuid;

pub(crate) struct IdentityService;

impl IdentityService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn get_devices(
        &self,
        client: &mut AgentServiceClient<Channel>,
        identity_id: &str,
        auth_value: &AsciiMetadataValue,
    ) -> Result<Vec<NoxyIdentityDevice>, tonic::Status> {
        let req = proto::GetIdentityDevicesRequest {
            request_id: Uuid::new_v4().to_string(),
            identity_id: identity_id.to_string(),
        };
        let mut request = Request::new(req);
        request.metadata_mut().insert("authorization", auth_value.clone());
        let response = client.get_identity_devices(request).await?.into_inner();
        let devices = response
            .devices
            .into_iter()
            .map(|d| NoxyIdentityDevice {
                device_id: d.device_id,
                public_key: d.public_key,
                pq_public_key: d.pq_public_key,
            })
            .collect();
        Ok(devices)
    }
}
