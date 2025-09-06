use std::io::{Error, ErrorKind};

use deadpool_redis::{redis::AsyncCommands, Connection, Pool};
use megacommerce_proto::CachedTokenStatus;
use megacommerce_shared::{
  models::{
    errors::{BoxedErr, ErrorType, InternalError},
    r_lock::RLock,
    redis::auth_token_status_key,
  },
  utils::time::time_get_seconds,
};
use tonic::async_trait;
use tower::BoxError;

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

  pub async fn get_token(
    &self,
    token: &str,
    path: &str,
  ) -> Result<Option<CachedTokenStatus>, BoxedErr> {
    let ie = |err: BoxedErr, msg: &str| {
      InternalError::new(path.into(), err, ErrorType::Internal, false, msg.into())
    };

    let key = auth_token_status_key(token);
    let mut con = self.get_conn(path).await?;

    let res: Option<String> =
      con.get(key).await.map_err(|err| ie(Box::new(err), "failed to get user data from redis"))?;

    match res {
      Some(json_str) => {
        let token_status: CachedTokenStatus = serde_json::from_str(&json_str)
          .map_err(|err| ie(Box::new(err), "failed to deserialize CachedTokenStatus"))?;
        Ok(Some(token_status))
      }
      None => Ok(None),
    }
  }

  pub async fn set_token(
    &self,
    jti: &str,
    data: &CachedTokenStatus,
    path: &str,
  ) -> Result<(), BoxedErr> {
    let ie = |err: BoxedErr, msg: &str| {
      InternalError::new(path.into(), err, ErrorType::Internal, false, msg.into())
    };

    let key = auth_token_status_key(jti);
    let mut con = self.get_conn(path).await?;

    let value = serde_json::to_string(data)
      .map_err(|err| ie(Box::new(err), "failed to serialize CachedTokenStatus"))?;

    let _: () = con
      .set(key, value)
      .await
      .map_err(|err| ie(Box::new(err), "failed to set CachedTokenStatus in redis"))?;

    Ok(())
  }
}

#[async_trait]
impl RedisClient for DefaultRedisClient {
  async fn check_token(&self, jti: &str) -> Result<RedisCheck, BoxedErr> {
    let res = self.get_token(jti, "auth.controller.check_token").await?;

    match res {
      Some(status) => {
        if status.revoked {
          return Ok(RedisCheck::Revoked("token got revoked".into()));
        }

        Ok(RedisCheck::Allowed { status: Some(status) })
      }
      None => Ok(RedisCheck::Allowed { status: None }),
    }
  }

  async fn revoke_token(&self, jti: &str) -> Result<(), BoxedErr> {
    let path = "auth.controller.check_token";
    let res = self.get_token(&jti, path).await?;

    if res.is_none() {
      return Err(DefaultRedisClient::not_found_err(path, jti));
    }

    let mut payload = res.unwrap();
    payload.revoked = true;
    self.set_token(jti, &payload, path).await?;

    Ok(())
  }

  // TODO: get the device id
  async fn mark_checked_ok(&self, jti: &str) -> Result<(), BoxedErr> {
    let path = "auth.controller.mark_checked_ok";
    let res = self.get_token(&jti, path).await?;

    if res.is_none() {
      let payload = CachedTokenStatus {
        revoked: false,
        last_checked: time_get_seconds() as i64,
        dev_id: "".into(),
      };
      self.set_token(jti, &payload, &path).await?;
      return Ok(());
    }

    let mut payload = res.unwrap();
    payload.revoked = false;
    payload.last_checked = time_get_seconds() as i64;
    self.set_token(jti, &payload, &path).await?;

    Ok(())
  }
}
