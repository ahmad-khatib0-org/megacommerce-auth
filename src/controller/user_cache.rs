use megacommerce_shared::models::{errors::BoxedErr, redis::auth_user_data_key};

use super::Controller;

impl Controller {
  pub async fn insert_auth_cached_user_data(&self, email: &str) -> Result<(), BoxedErr> {
    let key = auth_user_data_key(email);

    Ok(())
  }
}
