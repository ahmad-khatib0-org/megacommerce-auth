mod config;
mod init;

use std::sync::Arc;

use deadpool_redis::Pool;
use megacommerce_proto::Config as SharedConfig;
use megacommerce_shared::models::{
  errors::{BoxedErr, ErrorType, InternalError},
  r_lock::RLock,
  translate::translations_init,
};
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
};

#[derive(Debug, Clone)]
pub struct Server {
  pub(crate) common: Common,
  pub(crate) errors_send: Sender<InternalError>,
  pub(crate) service_config: Arc<Mutex<Config>>,
  pub(crate) shared_config: Arc<RwLock<SharedConfig>>,
  pub(crate) redis: Option<Arc<Pool>>,
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

    self.redis = Some(Arc::new(self.init_redis().await?));

    let translations = self.common.translations(|trans| trans.clone()).await;
    translations_init(translations, 5).map_err(|err| ie("error init trans", Box::new(err)))?;

    let controller_args = { ControllerArgs { config: self.config() } };
    let controller = Controller::new(controller_args).await;
    controller.run().await?;

    Ok(())
  }

  pub async fn errors_listener(&self, mut receiver: Receiver<InternalError>) {
    while let Some(msg) = receiver.recv().await {
      println!("received an internal error: {}", msg)
    }
  }

  /// Return a read-only config to pass downstream
  pub fn config(&self) -> RLock<SharedConfig> {
    RLock::<SharedConfig>(self.shared_config.clone())
  }
}
