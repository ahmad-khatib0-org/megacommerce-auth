use std::sync::Arc;

use megacommerce_shared::{models::context::Context, store::errors::DBError};

use crate::store::database::AuthStore;

use super::AuthStoreImpl;

#[tonic::async_trait]
impl AuthStore for AuthStoreImpl {
  async fn get_user_auth_data(&self, ctx: Arc<Context>, email: &str) -> Result<(), DBError> {
    Ok(())
  }
}
