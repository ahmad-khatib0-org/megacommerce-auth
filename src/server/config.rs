use std::{env, error::Error, fs};

use megacommerce_shared::models::errors::{BoxedErr, ErrorType, InternalError};

use crate::models::config::Config;

use super::Server;

impl Server {
  pub async fn init_servie_config(&self) -> Result<(), Box<dyn Error>> {
    let env = env::var("ENV").unwrap_or_else(|_| "dev".into());
    let return_err = |msg: &str, err: BoxedErr| InternalError {
      err_type: ErrorType::ConfigError,
      temp: false,
      msg: msg.into(),
      path: "auth.server.init_service_config".into(),
      err,
    };

    let yaml_string = fs::read_to_string(format!("config.{}.yaml", env))
      .map_err(|err| return_err("failed to load service config file", Box::new(err)))?;

    let parsed_config: Config = serde_yaml::from_str(&yaml_string)
      .map_err(|err| return_err("failed to parse config data", Box::new(err)))?;

    let mut config = self.service_config.lock().await;
    *config = parsed_config;

    Ok(())
  }
}
