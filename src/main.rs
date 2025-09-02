use std::error::Error;

use megacommerce_auth::server::Server;
use tracing::{subscriber, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let subscriber = FmtSubscriber::builder().with_max_level(Level::DEBUG).finish();
  subscriber::set_global_default(subscriber).expect("failed to set log subscriber");

  let server = Server::new().await;
  match server {
    Ok(mut srv) => return srv.run().await,
    Err(e) => Err(e),
  }
}
