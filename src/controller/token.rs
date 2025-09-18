use deadpool_redis::redis::AsyncCommands;
use megacommerce_proto::CachedTokenStatus;
use megacommerce_shared::{
  models::{
    errors::{BoxedErr, ErrorType, InternalError},
    redis::auth_token_status_key,
  },
  utils::time::time_get_seconds,
};

use super::redis::{DefaultRedisClient, RedisCheck, RedisClient};

pub(super) async fn check_token(r: &DefaultRedisClient, jti: &str) -> Result<RedisCheck, BoxedErr> {
  let res = r.get_token(jti, "auth.controller.check_token").await?;

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

pub(super) async fn revoke_token(r: &DefaultRedisClient, jti: &str) -> Result<(), BoxedErr> {
  let path = "auth.controller.check_token";
  let res = r.get_token(&jti, path).await?;

  if res.is_none() {
    return Err(DefaultRedisClient::not_found_err(path, jti));
  }

  let mut payload = res.unwrap();
  payload.revoked = true;
  r.set_token(jti, &payload, path).await?;

  Ok(())
}

// TODO: get the device id
pub(super) async fn mark_checked_ok(r: &DefaultRedisClient, jti: &str) -> Result<(), BoxedErr> {
  let path = "auth.controller.mark_checked_ok";
  let res = r.get_token(&jti, path).await?;

  if res.is_none() {
    let payload = CachedTokenStatus {
      revoked: false,
      last_checked: time_get_seconds() as i64,
      dev_id: "".into(),
    };
    r.set_token(jti, &payload, &path).await?;
    return Ok(());
  }

  let mut payload = res.unwrap();
  payload.revoked = false;
  payload.last_checked = time_get_seconds() as i64;
  r.set_token(jti, &payload, &path).await?;

  Ok(())
}

pub async fn get_token(
  r: &DefaultRedisClient,
  token: &str,
  path: &str,
) -> Result<Option<CachedTokenStatus>, BoxedErr> {
  let ie = |err: BoxedErr, msg: &str| {
    InternalError::new(path.into(), err, ErrorType::Internal, false, msg.into())
  };

  let key = auth_token_status_key(token);
  let mut con = r.get_conn(path).await?;

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
  r: &DefaultRedisClient,
  jti: &str,
  data: &CachedTokenStatus,
  path: &str,
) -> Result<(), BoxedErr> {
  let ie = |err: BoxedErr, msg: &str| {
    InternalError::new(path.into(), err, ErrorType::Internal, false, msg.into())
  };

  let key = auth_token_status_key(jti);
  let mut con = r.get_conn(path).await?;

  let value = serde_json::to_string(data)
    .map_err(|err| ie(Box::new(err), "failed to serialize CachedTokenStatus"))?;

  let _: () = con
    .set(key, value)
    .await
    .map_err(|err| ie(Box::new(err), "failed to set CachedTokenStatus in redis"))?;

  Ok(())
}
