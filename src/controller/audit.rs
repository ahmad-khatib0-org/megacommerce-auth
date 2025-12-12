use megacommerce_shared::models::errors::BoxedErr;
use tokio::spawn;

use super::Controller;

impl Controller {
  pub fn report_internal_error(&self, _err: BoxedErr) {
    let redis = self.redis_con.clone();
    spawn(async move {
      let _con = redis.get().await.get().await;
    });
  }
}
