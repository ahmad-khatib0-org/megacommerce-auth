use std::io::{Error, ErrorKind};

use deadpool_redis::{Connection, Pool};
use megacommerce_proto::CachedTokenStatus;
use megacommerce_shared::models::{
  errors::{BoxedErr, ErrorType, InternalError},
  r_lock::RLock,
};
use tonic::async_trait;
use tower::BoxError;

use super::token::{check_token, get_token, mark_checked_ok, revoke_token, set_token};

/// Represents Redis check results
#[derive(Debug)]
pub enum RedisCheck {
  Allowed { status: Option<CachedTokenStatus> },
  Revoked(String), // reason
}

#[async_trait]
pub trait RedisClient: Send + Sync {
  async fn check_token(&self, token: &str) -> Result<RedisCheck, BoxedErr>;
  async fn revoke_token(&self, token: &str) -> Result<(), BoxedErr>;
  async fn mark_checked_ok(&self, token: &str) -> Result<(), BoxedErr>;
  async fn get_token(&self, token: &str, path: &str)
    -> Result<Option<CachedTokenStatus>, BoxedErr>;
  async fn set_token(
    &self,
    jti: &str,
    data: &CachedTokenStatus,
    path: &str,
  ) -> Result<(), BoxedErr>;
}

/// Concrete Redis client wrapper
#[derive(Debug, Clone)]
pub struct DefaultRedisClient {
  pub redis: RLock<Pool>,
}

impl DefaultRedisClient {
  pub fn not_found_err(path: &str, jti: &str) -> BoxError {
    let msg = format!("redis key not found: {}", jti);
    let err = Box::new(Error::new(ErrorKind::NotFound, msg.clone()));
    Box::new(InternalError::new(path.into(), err, ErrorType::Internal, false, msg))
  }

  pub async fn get_conn(&self, path: &str) -> Result<Connection, BoxedErr> {
    Ok(self.redis.get().await.get().await.map_err(|err| {
      InternalError::new(
        path.into(),
        Box::new(err),
        ErrorType::Internal,
        false,
        "failed to get a redis connection from pool".into(),
      )
    })?)
  }
}

#[async_trait]
impl RedisClient for DefaultRedisClient {
  async fn get_token(
    &self,
    token: &str,
    path: &str,
  ) -> Result<Option<CachedTokenStatus>, BoxedErr> {
    get_token(self, &token, &path).await
  }

  async fn set_token(
    &self,
    jti: &str,
    data: &CachedTokenStatus,
    path: &str,
  ) -> Result<(), BoxedErr> {
    set_token(self, jti, data, path).await
  }

  async fn check_token(&self, jti: &str) -> Result<RedisCheck, BoxedErr> {
    check_token(&self, &jti).await
  }

  async fn revoke_token(&self, jti: &str) -> Result<(), BoxedErr> {
    revoke_token(&self, &jti).await
  }

  async fn mark_checked_ok(&self, jti: &str) -> Result<(), BoxedErr> {
    mark_checked_ok(&self, &jti).await
  }
}
