mod extension;
mod hydra;
mod redis;
mod router;

use std::net::SocketAddr;

use hydra::DefaultHydraClient;
use megacommerce_proto::service::auth::v3::authorization_server::AuthorizationServer;
use megacommerce_proto::Config;
use megacommerce_shared::models::errors::{BoxedErr, ErrorType, InternalError};
use megacommerce_shared::models::r_lock::RLock;
use redis::DefaultRedisClient;
use tonic::transport::Server as TonicServer;

use crate::utils::net::validate_url_target;

pub struct ControllerArgs {
  pub config: RLock<Config>,
}

#[derive(Debug)]
pub struct Controller {
  pub config: RLock<Config>,
  pub hydra: DefaultHydraClient,
  pub redis: DefaultRedisClient,
}

impl Controller {
  pub async fn new(ca: ControllerArgs) -> Self {
    let urls = {
      let config = ca.config.get().await;
      let hydra = config.services.as_ref().unwrap().oauth_provider_url();
      let redis = config.cache.as_ref().unwrap().redis_address();
      (hydra.to_string(), redis.to_string())
    };

    let hydra = DefaultHydraClient { base_url: urls.0 };
    let redis = DefaultRedisClient { connection_url: urls.1 };
    Self { config: ca.config, hydra, redis }
  }

  pub async fn run(self) -> Result<(), BoxedErr> {
    let url = {
      let config = self.config.get().await;
      config.services.as_ref().unwrap().auth_service_grpc_url().to_owned()
    };

    validate_url_target(&url).map_err(|e| {
      Box::new(InternalError {
        temp: false,
        err: Box::new(e),
        err_type: ErrorType::Internal,
        msg: "failed to run auth service server".into(),
        path: "auth.controller.run".into(),
      })
    })?;

    TonicServer::builder()
      .add_service(AuthorizationServer::new(self))
      .serve(url.parse::<SocketAddr>()?)
      .await?;

    Ok(())
  }
}
