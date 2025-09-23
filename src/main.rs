mod commands; // handle command, SET, GET, ZADD, etc
mod config; // handle server config.
mod network; // handle network connection handler.
mod persistence; // data persistence.
mod protocol; // redis protocol decode and encode.
mod server; // ds-cache server
mod storage; // data store
mod utils; // util functions.

use crate::{config::CacheConfig, server::Server};
use anyhow::Result;

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    info!("A Redis Server Build with Rust");

    let conf = CacheConfig {
        addr: "0.0.0.0:6869".to_string(),
    };

    let server = Server::new(conf, 1000);
    server.run().await
}
