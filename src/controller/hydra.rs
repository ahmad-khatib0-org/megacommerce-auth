use megacommerce_shared::models::errors::BoxedErr;
use tonic::async_trait;

/// Represents the result of a Hydra token validation.
#[derive(Debug)]
pub enum HydraValidation {
  Valid,
  Invalid(String), // reason why token is invalid
}

/// Trait for Hydra client behavior
#[async_trait]
pub trait HydraClient: Send + Sync {
  async fn validate_token(&self, token: &str) -> Result<HydraValidation, BoxedErr>;
}

/// Concrete Hydra client
#[derive(Debug)]
pub struct DefaultHydraClient {
  pub base_url: String,
}

#[async_trait]
impl HydraClient for DefaultHydraClient {
  async fn validate_token(&self, token: &str) -> Result<HydraValidation, BoxedErr> {
    Ok(HydraValidation::Valid)
  }
}
