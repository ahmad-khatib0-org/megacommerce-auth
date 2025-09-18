use std::sync::Arc;

use megacommerce_proto::CachedUserData;
use megacommerce_shared::{models::context::Context, store::errors::DBError};

use crate::store::database::AuthStore;

use super::{user::user_get_auth_data, AuthStoreImpl};

#[tonic::async_trait]
impl AuthStore for AuthStoreImpl {
  async fn user_get_auth_data(
    &self,
    ctx: Arc<Context>,
    email: &str,
  ) -> Result<CachedUserData, DBError> {
    user_get_auth_data(self, ctx, email).await
  }
}
