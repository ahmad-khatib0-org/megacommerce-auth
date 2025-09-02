use std::{error::Error, time::Duration};

use megacommerce_proto::{common_service_client::CommonServiceClient, PingRequest};
use megacommerce_shared::models::errors::{ErrorType, InternalError};
use tokio::time::timeout;
use tonic::{transport::Channel, Request};
use tower::BoxError;

use crate::utils::net::validate_url_target;

use super::Common;

impl Common {
  pub async fn init_client(&mut self) -> Result<CommonServiceClient<Channel>, Box<dyn Error>> {
    let return_err = |msg: &str, err: BoxError| {
      Box::new(InternalError {
        err,
        err_type: ErrorType::Internal,
        temp: false,
        msg: msg.into(),
        path: "auth.common.init_client".into(),
      }) as Box<dyn Error>
    };

    let url = self.service_config.service.common_service_grpc_url.clone();
    if let Err(e) = validate_url_target(&url) {
      return Err(return_err("failed to validate common client URL", Box::new(e)));
    }

    let mut client = CommonServiceClient::connect(url)
      .await
      .map_err(|err| return_err("failed to connect to common client", Box::new(err)))?;

    let request = Request::new(PingRequest {});
    let response = timeout(Duration::from_secs(5), client.ping(request)).await;
    match response {
      Ok(Ok(_)) => {}
      Ok(Err(e)) => {
        return Err(return_err("failed to ping the common client service", Box::new(e)))
      }
      Err(e) => return Err(return_err("the ping to common client service timedout", Box::new(e))),
    };

    Ok(client)
  }
}
