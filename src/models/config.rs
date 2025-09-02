use derive_more::Display;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, Display)]
pub struct Config {
  pub service: ServiceConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Display)]
#[display("ServiceConfig: {env} {service_grpc_url} {common_service_grpc_url}")]
pub struct ServiceConfig {
  pub env: String,
  pub service_grpc_url: String,
  pub common_service_grpc_url: String,
}
