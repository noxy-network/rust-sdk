//! gRPC transport layer for Noxy relay communication.

pub(crate) mod proto {
    include!(concat!(env!("OUT_DIR"), "/noxy.agent.rs"));
}

pub(crate) use proto::agent_service_client::AgentServiceClient;
