use megacommerce_auth::server::Server;
use megacommerce_shared::models::errors::BoxedErr;
use tracing::{subscriber, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), BoxedErr> {
  let subscriber = FmtSubscriber::builder().with_max_level(Level::DEBUG).finish();
  subscriber::set_global_default(subscriber).expect("failed to set log subscriber");

  let server = Server::new().await;
  match server {
    Ok(mut srv) => return srv.run().await,
    Err(e) => Err(e),
  }
}
