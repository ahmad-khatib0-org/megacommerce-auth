use std::sync::Arc;

use megacommerce_shared::models::{
  context::Context,
  errors::{BoxedErr, ErrorType, InternalError},
};

use super::Controller;

impl Controller {
  pub async fn insert_auth_cached_user_data(
    &self,
    ctx: Arc<Context>,
    email: &str,
  ) -> Result<(), BoxedErr> {
    let path = "auth.controller.insert_auth_cached_user_data";
    let int_err = |err: BoxedErr, msg: &str| InternalError {
      err,
      msg: msg.into(),
      temp: true,
      path: path.into(),
      err_type: ErrorType::Internal,
    };
    let data = self
      .store
      .get()
      .await
      .user_get_auth_data(ctx, email)
      .await
      .map_err(|err| int_err(Box::new(err), "failed to get user auth data"))?;

    let con = self.redis.get_conn(&path).await?;

    Ok(())
  }
}
