use std::sync::Arc;

use chrono::Utc;
use megacommerce_proto::{
  config::core::v3::{header_value_option::HeaderAppendAction, HeaderValue, HeaderValueOption},
  google::{protobuf::BoolValue, rpc::Status},
  service::auth::v3::{check_response::HttpResponse, CheckRequest, CheckResponse, OkHttpResponse},
  JwtClaims,
};
use megacommerce_shared::models::{
  context::{Context, Session},
  errors::{BoxedErr, ErrorType, InternalError},
  network::Header,
  translate::tr,
};
use tonic::{Code, Request, Response};

use crate::utils::net::{extract_jwt_token_from_request, get_essential_http_headers};

use super::Controller;

impl Controller {
  // TODO: handle getting the real ip
  pub fn get_context(&self, req: &CheckRequest) -> Arc<Context> {
    let h = get_essential_http_headers(req);
    let path = req
      .attributes
      .as_ref()
      .and_then(|a| a.request.as_ref())
      .and_then(|r| r.http.as_ref())
      .map(|h| h.path.clone())
      .unwrap_or_default();

    Arc::new(Context {
      session: Session::default(),
      ip_address: h.x_forwarded_for.clone(),
      x_forwarded_for: h.x_forwarded_for,
      request_id: h.x_request_id,
      path,
      user_agent: h.user_agent,
      accept_language: h.accept_language,
    })
  }

  pub async fn response_ok(
    &self,
    ctx: &Arc<Context>,
    req: &Request<CheckRequest>,
    claims: Option<JwtClaims>,
  ) -> Response<CheckResponse> {
    let headers = self.prepare_headers(ctx, req, claims).await;
    if headers.is_err() {
      self.report_internal_error(headers.unwrap_err());
      return Response::new(CheckResponse {
        status: Some(Status {
          code: Code::Internal as i32,
          message: Controller::int_err_msg(&ctx.accept_language),
          details: vec![],
        }),
        ..Default::default()
      });
    }

    Response::new(CheckResponse {
      status: Some(Status { code: Code::Ok as i32, message: "".into(), details: vec![] }),
      http_response: Some(HttpResponse::OkResponse(OkHttpResponse {
        headers: headers.unwrap(),
        ..Default::default()
      })),
      ..Default::default()
    })
  }

  pub async fn prepare_headers(
    &self,
    ctx: &Arc<Context>,
    req: &Request<CheckRequest>,
    claims: Option<JwtClaims>,
  ) -> Result<Vec<HeaderValueOption>, BoxedErr> {
    let mut headers: Vec<HeaderValueOption> = vec![];
    let device_id = "dump device id";

    let header = |header: Header, value: String| {
      HeaderValueOption {
        append: Some(BoolValue { value: false }),
        append_action: HeaderAppendAction::OverwriteIfExistsOrAdd.into(),
        keep_empty_value: false, // don't add header if value is empty,
        header: Some(HeaderValue {
          key: header.to_string(),
          value,
          raw_value: Vec::new(), // leave empty unless raw bytes is needed
        }),
      }
    };

    if claims.is_some() {
      let c = claims.unwrap().clone();
      if c.jti.clone().is_empty() && c.sub.clone().is_empty() {
        let token = extract_jwt_token_from_request(req).unwrap_or_default();
        let user_id = c.sub.clone();
        let auth_data =
          self.get_or_insert_auth_cached_user_data(ctx.clone(), &user_id).await.map_err(|err| {
            let msg =
              "failed to get/insert uesr data to be fowarded to downstream services as metadata";
            InternalError {
              err,
              err_type: ErrorType::Internal,
              msg: msg.into(),
              temp: true,
              path: "auth.controller.prepare_headers".into(),
            }
          })?;

        headers.push(header(Header::SessionId, c.jti));
        headers.push(header(Header::Token, token));
        headers.push(header(
          Header::CreatedAt,
          c.iat.and_then(|t| t.seconds.to_string().into()).unwrap_or_default(),
        ));
        headers.push(header(
          Header::ExpiresAt,
          c.exp.and_then(|t| t.seconds.to_string().into()).unwrap_or_default(),
        ));
        headers.push(header(Header::LastActivityAt, Utc::now().timestamp().to_string()));
        headers.push(header(Header::UserId, c.sub));
        headers.push(header(Header::DeviceId, device_id.to_string()));
        headers.push(header(Header::Roles, auth_data.roles));
        headers.push(header(Header::IsOauth, auth_data.is_oauth.to_string()));
        headers.push(header(Header::Props, auth_data.props));
      }
    }

    headers.push(header(Header::XRequestId, ctx.request_id.clone()));
    headers.push(header(Header::XIpAddress, ctx.ip_address.clone()));
    headers.push(header(Header::XForwardedFor, ctx.x_forwarded_for.clone()));
    headers.push(header(Header::Path, ctx.path.clone()));
    headers.push(header(Header::UserAgent, ctx.user_agent.clone()));
    headers.push(header(Header::AcceptLanguage, ctx.accept_language.clone()));
    Ok(headers)
  }

  pub fn not_found_msg(lang: &str) -> String {
    return tr::<String>(lang, "error.not_found", None)
      .unwrap_or("The requested path is not provided!".into());
  }

  pub fn invalid_token_msg(lang: &str) -> String {
    return tr::<String>(lang, "jwt.payload.invalid", None)
      .unwrap_or("Sorry, the authentication payload is invalid, please login first".into());
  }

  pub fn int_err_msg(lang: &str) -> String {
    return tr::<String>(lang, "error.internal", None).unwrap_or(
      "Sorry, Unexpected internal server error. Our team has been notified. Please try again"
        .into(),
    );
  }
}

pub trait CheckResponseExt {
  fn denied(msg: &str) -> Self;
}

impl CheckResponseExt for CheckResponse {
  fn denied(msg: &str) -> Self {
    Self {
      status: Some(Status {
        code: Code::PermissionDenied as i32,
        message: msg.to_string(),
        details: vec![],
      }),
      ..Default::default()
    }
  }
}
