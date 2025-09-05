use megacommerce_proto::google::rpc::Status;
use megacommerce_proto::service::auth::v3::CheckResponse;
use tonic::Code;

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
