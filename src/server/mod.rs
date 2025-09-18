mod config;
mod database;
mod getters;
mod init;

use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use megacommerce_proto::Config as SharedConfig;
use megacommerce_shared::models::{
  errors::{BoxedErr, ErrorType, InternalError},
  translate::translations_init,
};
use sqlx::{Pool, Postgres};
use tokio::{
  spawn,
  sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex, RwLock,
  },
};

use crate::{
  common::{Common, CommonArgs},
  controller::{Controller, ControllerArgs},
  models::config::Config,
  store::{
    database::AuthStore,
    pg_impl::{AuthStoreImpl, AuthStoreImplArgs},
  },
};

#[derive(Debug, Clone)]
pub struct Server {
  pub(crate) common: Common,
  pub(crate) errors_send: Sender<InternalError>,
  pub(crate) service_config: Arc<Mutex<Config>>,
  pub(crate) shared_config: Arc<RwLock<SharedConfig>>,
  pub(crate) redis: Option<Arc<RwLock<RedisPool>>>,
  pub(crate) db: Option<Arc<RwLock<Pool<Postgres>>>>,
  pub(crate) store: Option<Arc<RwLock<dyn AuthStore + Send + Sync>>>,
}

impl Server {
  pub async fn new() -> Result<Self, BoxedErr> {
    let (tx, rx) = channel::<InternalError>(100);

    let mut srv = Server {
      common: Common::default(),
      errors_send: tx,
      service_config: Arc::new(Mutex::new(Config::default())),
      shared_config: Arc::new(RwLock::new(SharedConfig::default())),
      redis: None,
      db: None,
      store: None,
    };

    srv.init_servie_config().await?;

    let common_args = {
      let service_config = srv.service_config.lock().await.clone();
      CommonArgs { service_config }
    };

    srv.common = Common::new(common_args).await?;

    {
      let cfg = srv.common.config(|cfg| cfg.clone()).await;
      let mut old_cfg = srv.shared_config.write().await;
      *old_cfg = cfg;
    }

    let srv_clone = srv.clone();
    spawn(async move { srv_clone.errors_listener(rx).await });

    Ok(srv)
  }

  pub async fn run(&mut self) -> Result<(), BoxedErr> {
    let ie = |msg: &str, err: BoxedErr| InternalError {
      err_type: ErrorType::Internal,
      temp: false,
      err,
      msg: msg.into(),
      path: "auth.server.run".into(),
    };

    self.redis = Some(Arc::new(RwLock::new(self.init_redis().await?)));
    self.db = Some(Arc::new(RwLock::new(self.init_database().await?)));

    let translations = self.common.translations(|trans| trans.clone()).await;
    translations_init(translations, 5).map_err(|err| ie("error init trans", Box::new(err)))?;

    self.store =
      Some(Arc::new(RwLock::new(AuthStoreImpl::new(AuthStoreImplArgs { db: self.db() }))));

    let controller_args =
      { ControllerArgs { config: self.config(), redis_con: self.redis(), store: self.store() } };

    let controller = Controller::new(controller_args).await;
    controller.run().await?;

    Ok(())
  }

  pub async fn errors_listener(&self, mut receiver: Receiver<InternalError>) {
    while let Some(msg) = receiver.recv().await {
      println!("received an internal error: {}", msg)
    }
  }
}
