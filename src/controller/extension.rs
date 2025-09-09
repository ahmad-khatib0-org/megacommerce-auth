use megacommerce_proto::google::rpc::Status;
use megacommerce_proto::service::auth::v3::CheckResponse;
use megacommerce_shared::models::translate::tr;
use tonic::Code;

use super::Controller;

pub trait CheckResponseExt {
  fn ok() -> Self;
  fn denied(msg: &str) -> Self;
}

impl CheckResponseExt for CheckResponse {
  fn ok() -> Self {
    Self {
      status: Some(Status { code: Code::Ok as i32, message: "".into(), details: vec![] }),
      ..Default::default()
    }
  }

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

impl Controller {
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
