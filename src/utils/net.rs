use std::io::{Error, ErrorKind};

use http::Uri;
use megacommerce_proto::{service::auth::v3::CheckRequest, JwtClaims, Timestamp};
use tonic::Request;

use crate::models::network::EssentialHttpHeaders;

pub fn validate_url_target(url: &str) -> Result<Uri, Error> {
  url.parse::<Uri>().map_err(|e| Error::new(ErrorKind::InvalidInput, format!("invalid URL: {}", e)))
}

pub fn extract_jwt_token_from_request<T>(req: &Request<T>) -> Option<String> {
  req.metadata().get("authorization")?.to_str().ok()?.strip_prefix("Bearer ")?.to_string().into()
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

pub fn get_essential_http_headers(req: &CheckRequest) -> EssentialHttpHeaders {
  let headers = req
    .attributes
    .as_ref()
    .and_then(|a| a.request.as_ref())
    .and_then(|r| r.http.as_ref())
    .map(|h| {
      h.headers
        .iter()
        .map(|(k, v)| (k.to_lowercase(), v.clone()))
        .collect::<std::collections::HashMap<_, _>>()
    })
    .unwrap_or_default();

  let get = |key: &str| headers.get(key).cloned().unwrap_or_default();

  EssentialHttpHeaders {
    path: req
      .attributes
      .as_ref()
      .and_then(|a| a.request.as_ref())
      .and_then(|r| r.http.as_ref())
      .map(|h| h.path.clone())
      .unwrap_or_default(),

    method: req
      .attributes
      .as_ref()
      .and_then(|a| a.request.as_ref())
      .and_then(|r| r.http.as_ref())
      .map(|h| h.method.clone())
      .unwrap_or_default(),

    user_agent: get("user-agent"),
    x_forwarded_for: get("x-forwarded-for"),
    x_request_id: get("x-request-id"),
    accept_language: get("accept-language"),
    headers,
  }
}
