mod config;

use std::{error::Error, sync::Arc};

use megacommerce_shared::models::{
  errors::{BoxedErr, ErrorType, InternalError},
  translate::translations_init,
};
use tokio::{
  spawn,
  sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
  },
};

use crate::{
  common::{Common, CommonArgs},
  models::config::Config,
};

#[derive(Debug, Clone)]
pub struct Server {
  pub(crate) common: Option<Common>,
  pub(crate) errors_send: Sender<InternalError>,
  pub(crate) service_config: Arc<Mutex<Config>>,
}

impl Server {
  pub async fn new() -> Result<Self, Box<dyn Error>> {
    let (tx, rx) = channel::<InternalError>(100);

    let mut srv = Server {
      common: None,
      errors_send: tx,
      service_config: Arc::new(Mutex::new(Config::default())),
    };

    srv.init_servie_config().await?;

    let common_args = {
      let service_config = srv.service_config.lock().await.clone();
      CommonArgs { service_config }
    };
    match Common::new(common_args).await {
      Ok(com) => srv.common = Some(com),
      Err(err) => return Err(err),
    }

    let srv_clone = srv.clone();
    spawn(async move { srv_clone.errors_listener(rx).await });

    Ok(srv)
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    let ie = |msg: &str, err: BoxedErr| InternalError {
      err_type: ErrorType::Internal,
      temp: false,
      err,
      msg: msg.into(),
      path: "auth.server.run".into(),
    };

    let translations = self.common.as_ref().unwrap().translations(|trans| trans.clone()).await;
    translations_init(translations, 5).map_err(|err| ie("error init trans", Box::new(err)))?;

    Ok(())
  }

  pub async fn errors_listener(&self, mut receiver: Receiver<InternalError>) {
    while let Some(msg) = receiver.recv().await {
      println!("received an internal error: {}", msg)
    }
  }
}
