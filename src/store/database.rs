use std::{fmt, sync::Arc};

use megacommerce_shared::{models::context::Context, store::errors::DBError};

#[tonic::async_trait]
pub trait AuthStore: fmt::Debug + Send + Sync {
  /// Gets user information about auth status, E,g if user registered with social account
  /// roles, user type (E,g supplier), ....
  async fn get_user_auth_data(&self, ctx: Arc<Context>, email: &str) -> Result<(), DBError>;
}
