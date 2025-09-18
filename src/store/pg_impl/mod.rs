mod router;

use megacommerce_shared::models::r_lock::RLock;
use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct AuthStoreImpl {
  pub(crate) db: RLock<Pool<Postgres>>,
}

#[derive(Debug)]
pub struct AuthStoreImplArgs {
  pub db: RLock<Pool<Postgres>>,
}

impl AuthStoreImpl {
  pub fn new(args: AuthStoreImplArgs) -> Self {
    Self { db: args.db }
  }
}
