use chrono::Local;
use megacommerce_shared::models::errors::BoxedErr;
use tonic::async_trait;

/// Represents Redis check results
#[derive(Debug)]
pub enum RedisCheck {
  Allowed { last_checked: Option<i64> },
  Revoked(String), // reason
}

#[async_trait]
pub trait RedisClient: Send + Sync {
  async fn check_token(&self, token: &str) -> Result<RedisCheck, BoxedErr>;
  async fn revoke_token(&self, token: &str) -> Result<(), BoxedErr>;
  async fn mark_checked_ok(&self, token: &str, ts: i64) -> Result<(), BoxedErr>;
}

/// Concrete Redis client wrapper
#[derive(Debug, Clone)]
pub struct DefaultRedisClient {
  pub connection_url: String,
}

#[async_trait]
impl RedisClient for DefaultRedisClient {
  async fn check_token(&self, token: &str) -> Result<RedisCheck, BoxedErr> {
    Ok(RedisCheck::Allowed { last_checked: None }) // always allowed stub
  }

  async fn revoke_token(&self, _token: &str) -> Result<(), BoxedErr> {
    Ok(())
  }

  async fn mark_checked_ok(&self, _token: &str, _ts: i64) -> Result<(), BoxedErr> {
    Ok(())
  }
}
