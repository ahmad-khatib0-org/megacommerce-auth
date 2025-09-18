use std::sync::Arc;

use megacommerce_proto::CachedUserData;
use megacommerce_shared::{
  models::context::Context,
  store::errors::{handle_db_error, DBError},
};
use sqlx::query;

use super::AuthStoreImpl;

pub async fn user_get_auth_data(
  s: &AuthStoreImpl,
  _ctx: Arc<Context>,
  email: &str,
) -> Result<CachedUserData, DBError> {
  let row =
    query!(r#"SELECT user_type, roles, props, auth_service FROM users WHERE email = $1"#, email)
      .fetch_one(&s.db.get().await.clone())
      .await
      .map_err(|err| handle_db_error(err, "auth.store.user_get_auth_data"))?;

  Ok(CachedUserData {
    is_oauth: !row.auth_service.unwrap_or_default().is_empty(),
    roles: row.roles.join(","),
    props: row.props.unwrap_or_default().join(","),
  })
}
