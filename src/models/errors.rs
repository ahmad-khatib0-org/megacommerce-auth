use std::{error::Error, fmt};

use derive_more::Display;
use serde::{Deserialize, Serialize};

pub type BoxedErr = Box<dyn Error + Sync + Send>;
pub type OptionalErr = Option<BoxedErr>;
pub const MSG_ID_ERR_INTERNAL: &str = "server.internal.error";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
  NoRows,
  UniqueViolation,
  ForeignKeyViolation,
  NotNullViolation,
  JsonMarshal,
  JsonUnmarshal,
  Connection,
  Privileges,
  Internal,
  DBConnectionError,
  ConfigError,
}

impl fmt::Display for ErrorType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ErrorType::DBConnectionError => write!(f, "db_connection_error"),
      ErrorType::NoRows => write!(f, "no_rows"),
      ErrorType::UniqueViolation => write!(f, "unique_violation"),
      ErrorType::ForeignKeyViolation => write!(f, "foreign_key_violation"),
      ErrorType::NotNullViolation => write!(f, "not_null_violation"),
      ErrorType::JsonMarshal => write!(f, "json_marshal"),
      ErrorType::JsonUnmarshal => write!(f, "json_unmarshal"),
      ErrorType::Connection => write!(f, "connection_exception"),
      ErrorType::Privileges => write!(f, "insufficient_privilege"),
      ErrorType::ConfigError => write!(f, "config_error"),
      ErrorType::Internal => write!(f, "internal_error"),
    }
  }
}

#[derive(Debug, Display)]
#[display("InternalError: {path}: {msg}, temp: {temp}, err: {err_type} {err}")]
pub struct InternalError {
  pub err: Box<dyn Error + Send + Sync>,
  pub err_type: ErrorType,
  pub temp: bool,
  pub msg: String,
  pub path: String,
}

impl Error for InternalError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    Some(self.err.as_ref())
  }
}
