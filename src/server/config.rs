use std::{env, error::Error, fs};

use crate::models::{
  config::Config,
  errors::{ErrorType, InternalError},
};

use super::Server;

impl Server {
  pub async fn init_servie_config(&self) -> Result<(), Box<dyn Error>> {
    let env = env::var("ENV").unwrap_or_else(|_| "dev".into());

    let yaml_string =
      fs::read_to_string(format!("config.{}.yaml", env)).map_err(|err| InternalError {
        err_type: ErrorType::ConfigError,
        temp: false,
        msg: "failed to load service config file".into(),
        path: "auth.server.load_service_config".into(),
        err: Box::new(err),
      })?;

    let parsed_config: Config =
      serde_yaml::from_str(&yaml_string).map_err(|err| InternalError {
        temp: false,
        err_type: ErrorType::ConfigError,
        msg: "failed to parse config data".into(),
        path: "auth.server.load_service_config".into(),
        err: Box::new(err),
      })?;

    let mut config = self.service_config.lock().await;
    *config = parsed_config;

    Ok(())
  }
}
