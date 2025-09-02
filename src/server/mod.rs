mod config;

use std::{error::Error, sync::Arc};

use megacommerce_shared::models::errors::InternalError;
use tokio::{
  spawn,
  sync::{
    mpsc::{self, Receiver},
    Mutex,
  },
};

use crate::{
  common::{Common, CommonArgs},
  models::config::Config,
};

#[derive(Debug)]
pub struct Server {
  pub(crate) common: Option<Common>,
  pub(crate) errors: mpsc::Sender<InternalError>,
  pub(crate) service_config: Arc<Mutex<Config>>,
}

impl Server {
  pub async fn new() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<InternalError>(100);

    let mut srv =
      Server { common: None, errors: tx, service_config: Arc::new(Mutex::new(Config::default())) };

    srv.init_servie_config().await?;

    let common_args = {
      let service_config = srv.service_config.lock().await.clone();
      CommonArgs { service_config }
    };
    match Common::new(common_args).await {
      Ok(com) => srv.common = Some(com),
      Err(err) => return Err(err),
    }

    spawn(async move { srv.errors_listener(rx).await });

    Ok(())
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    Ok(())
  }

  pub async fn errors_listener(&self, mut receiver: Receiver<InternalError>) {
    while let Some(msg) = receiver.recv().await {
      println!("received an internal error: {}", msg)
    }
  }
}
