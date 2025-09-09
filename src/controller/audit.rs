use megacommerce_shared::models::errors::BoxedErr;
use tokio::spawn;

use super::Controller;

impl Controller {
  pub fn report_internal_error(&self, err: BoxedErr) {
    let redis = self.redis_con.clone();
    spawn(async move {
      let con = redis.get().await.get().await;
    });
  }
}
