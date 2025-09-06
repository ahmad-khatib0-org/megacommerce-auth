use std::{
  io::{Error, ErrorKind},
  sync::Arc,
};

use http::Uri;
use megacommerce_proto::{service::auth::v3::CheckRequest, JwtClaims, Timestamp};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, OptionalErr},
  network::Header,
};
use serde_json::{from_str, Value};
use tonic::{Code, Request};

pub fn validate_url_target(url: &str) -> Result<Uri, Error> {
  url.parse::<Uri>().map_err(|e| Error::new(ErrorKind::InvalidInput, format!("invalid URL: {}", e)))
}

pub fn extract_jwt_from_request(req: &CheckRequest) -> Option<String> {
  req
    .attributes
    .as_ref()?
    .request
    .as_ref()?
    .http
    .as_ref()?
    .headers
    .get(Header::Authorization.as_str())
    .and_then(|h| {
      if h.to_lowercase().starts_with("bearer ") {
        Some(h[7..].to_string()) // strip "Bearer "
      } else {
        None
      }
    })
}

pub fn extract_jwt_claims_from_request<T>(req: &Request<T>) -> JwtClaims {
  let meta = req.metadata();

  let get_header = |key: &str| -> String {
    meta.get(key).and_then(|v| v.to_str().ok()).unwrap_or_default().to_string()
  };

  // Helper to extract numeric timestamp (exp, nbf, iat)
  let get_timestamp = |key: &str| -> Option<Timestamp> {
    meta
      .get(key)
      .and_then(|v| v.to_str().ok())
      .and_then(|s| s.parse::<i64>().ok().map(|secs| Timestamp { seconds: secs, nanos: 0 }))
  };

  JwtClaims {
    iss: get_header("x-jwt-iss"),
    sub: get_header("x-jwt-sub"),
    aud: meta
      .get("x-jwt-aud")
      .and_then(|v| v.to_str().ok())
      .map(|s| vec![s.to_string()])
      .unwrap_or_default(),
    exp: get_timestamp("x-jwt-exp"),
    nbf: get_timestamp("x-jwt-nbf"),
    iat: get_timestamp("x-jwt-iat"),
    jti: get_header("x-jwt-jti"),
    custom: Default::default(),
  }
}

/// TODO: not used
pub fn extract_jti_from_request<T>(
  ctx: Arc<Context>,
  path: &str,
  req: &Request<T>,
) -> Result<String, AppError> {
  let id = "jwt.payload.invalid";
  let ae = |msg: &str, err: OptionalErr| {
    AppError::new(
      ctx.clone(),
      path,
      msg,
      None,
      "",
      Code::Unauthenticated as i32,
      Some(AppErrorErrors { err, ..Default::default() }),
    )
  };

  let payload = req.metadata().get("jwt_payload").ok_or_else(|| ae("jwt.payload.missing", None))?;

  // Metadata is bytes → turn into string
  let payload_str = payload.to_str().map_err(|err| ae(id, Some(Box::new(err))))?;
  let json: Value = from_str(payload_str).map_err(|err| ae(id, Some(Box::new(err))))?;
  let jti = json.get("jti").and_then(|v| v.as_str()).ok_or_else(|| ae(id, None))?;

  Ok(jti.to_string())
}
