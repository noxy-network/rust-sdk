//! gRPC transport layer for Noxy relay communication.

pub(crate) mod proto {
    include!(concat!(env!("OUT_DIR"), "/noxy.push.rs"));
}

pub(crate) use proto::push_service_client::PushServiceClient;
