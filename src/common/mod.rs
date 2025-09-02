mod config;
mod init;
use std::{error::Error, sync::Arc};

use derive_more::Display;
use megacommerce_proto::{common_service_client::CommonServiceClient, Config as SharedConfig};
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::models::config::Config as ServiceConfig;

#[derive(Debug, Display)]
pub struct CommonArgs {
  pub service_config: ServiceConfig,
}

#[derive(Debug)]
pub struct Common {
  pub(crate) client: Option<CommonServiceClient<Channel>>,
  pub(crate) service_config: ServiceConfig,
  pub(crate) shared_config: Arc<Mutex<SharedConfig>>,
}

impl Common {
  /// New initialize connection to the common service, initialize configurations
  pub async fn new(ca: CommonArgs) -> Result<Common, Box<dyn Error>> {
    let mut com = Common {
      service_config: ca.service_config,
      shared_config: Arc::new(Mutex::new(SharedConfig::default())),
      client: None,
    };

    match com.init_client().await {
      Ok(cli) => com.client = Some(cli),
      Err(e) => return Err(e),
    }

    match com.config_get().await {
      Ok(res) => {
        let mut config = com.shared_config.lock().await;
        *config = res;
      }
      Err(err) => return Err(err),
    }

    Ok(com)
  }

  /// Close the client connection by dropping it
  /// When `client` is dropped, the underlying Channel drops and closes the connection.
  pub fn close(&mut self) {
    self.client = None;
  }

  /// Reconnect: close old client if any, then create new one
  pub async fn reconnect(&mut self) -> Result<(), Box<dyn Error>> {
    self.close(); // drop old client if present
    let client = self.init_client().await?;
    self.client = Some(client);
    Ok(())
  }

  /// Accessor for client with error if not connected
  pub fn client(&mut self) -> Result<&mut CommonServiceClient<Channel>, Box<dyn Error>> {
    self.client.as_mut().ok_or_else(|| "common service client is not connected".into())
  }

  /// Config returns a read only access to config
  pub async fn config<T, F>(&self, f: F) -> T
  where
    F: FnOnce(&SharedConfig) -> T,
  {
    let config = self.shared_config.lock().await;
    f(&config)
  }
}
