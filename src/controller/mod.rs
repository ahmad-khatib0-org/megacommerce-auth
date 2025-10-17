mod audit;
mod hydra;
mod redis;
mod response;
mod router;
mod routes;
mod token;
mod user_cache;

use std::net::SocketAddr;

use deadpool_redis::Pool as RedisPool;
use hydra::DefaultHydraClient;
use megacommerce_proto::service::auth::v3::authorization_server::AuthorizationServer;
use megacommerce_proto::Config;
use megacommerce_shared::models::errors::{BoxedErr, ErrorType, InternalError};
use megacommerce_shared::models::r_lock::RLock;
use megacommerce_shared::utils::middleware::middleware_context;
use redis::DefaultRedisClient;
use reqwest::Client;
use tonic::service::InterceptorLayer;
use tonic::transport::Server as TonicServer;
use tower::ServiceBuilder;

use crate::store::database::AuthStore;
use crate::utils::net::validate_url_target;

pub struct ControllerArgs {
  pub config: RLock<Config>,
  pub redis_con: RLock<RedisPool>,
  pub store: RLock<dyn AuthStore + Send + Sync>,
}

#[derive(Debug)]
pub struct Controller {
  pub config: RLock<Config>,
  pub hydra: DefaultHydraClient,
  pub redis: DefaultRedisClient,
  pub redis_con: RLock<RedisPool>,
  pub(super) store: RLock<dyn AuthStore + Send + Sync>,

  pub cached_config: CachedConfig,
}

#[derive(Debug, Default)]
struct CachedConfig {
  pub available_languages: Vec<String>,
  pub default_language: String,
}

impl Controller {
  pub async fn new(ca: ControllerArgs) -> Self {
    let urls = {
      let config = ca.config.get().await;
      let oauth = config.oauth.as_ref().unwrap();
      let hydra = oauth.oauth_provider_url().to_owned();
      let id = oauth.oauth_client_id().to_owned();
      let secret = oauth.oauth_client_secret().to_owned();
      (hydra, id, secret)
    };

    let hydra = DefaultHydraClient {
      hydra_url: urls.0,
      http: Client::new(),
      client_id: urls.1,
      client_secret: urls.2,
    };

    let redis = DefaultRedisClient { redis: ca.redis_con.clone() };
    let cfg = ca.config.get().await.localization.clone().unwrap();
    let cached_config = CachedConfig {
      available_languages: cfg.available_locales.clone(),
      default_language: cfg.default_client_locale.clone().unwrap_or("en".to_string()),
    };
    Self {
      config: ca.config,
      hydra,
      redis,
      redis_con: ca.redis_con,
      store: ca.store,
      cached_config,
    }
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

    let layer = ServiceBuilder::new().layer(InterceptorLayer::new(middleware_context)).into_inner();
    TonicServer::builder()
      .layer(layer)
      .add_service(AuthorizationServer::new(self))
      .serve((url.parse::<SocketAddr>()).unwrap())
      .await?;

    Ok(())
  }
}
