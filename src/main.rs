mod commands; // handle command, SET, GET, ZADD, etc
mod config; // handle server config.
mod network; // handle network connection handler.
mod persistence; // data persistence.
mod protocol; // redis protocol decode and encode.
mod server; // ds-cache server
mod storage; // data store
mod utils; // util functions.

use anyhow::Result;

use crate::{config::CacheConfig, server::Server};

#[tokio::main]
async fn main() -> Result<()> {
    println!("A Redis Server Build with Rust");

    let conf = CacheConfig {
        addr: "0.0.0.0:6869".to_string(),
    };

    let server = Server::new(conf, 1000);
    server.run().await
}
