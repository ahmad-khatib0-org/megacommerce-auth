mod config;

use std::{error::Error, sync::Arc};

use megacommerce_proto::Config as SharedConfig;
use tokio::sync::{mpsc, Mutex};

use crate::models::{config::Config, errors::InternalError};

#[derive(Debug)]
pub struct Server {
  pub(crate) errors: mpsc::Sender<InternalError>,
  pub(crate) shared_config: Arc<Mutex<SharedConfig>>,
  pub(crate) service_config: Arc<Mutex<Config>>,
}

impl Server {
  pub async fn new() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<InternalError>(100);

    let srv = Server {
      errors: tx,
      shared_config: Arc::new(Mutex::new(SharedConfig::default())),
      service_config: Arc::new(Mutex::new(Config::default())),
    };

    srv.init_servie_config().await?;

    Ok(())
  }
}
