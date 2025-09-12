mod commands; // handle command, SET, GET, ZADD, etc
mod config; // handle server config.
mod network; // handle network connection handler.
mod persistence; // data persistence.
mod protocol; // redis protocol decode and encode.
mod storage; // data store
mod utils; // util functions.

use anyhow::{Result, anyhow};
use network::handle_conn1;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    println!("A Redis Server Build with Rust");

    let addr = "0.0.0.0:6869";
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow!("Faile to listen on {}: {}", addr, e))?;

    println!("server listen on: {}", addr);

    loop {
        match listener.accept().await {
            Ok((mut sock, client_addr)) => {
                println!("accept conn from: {}", client_addr);

                tokio::spawn(async move {
                    match handle_conn1(&mut sock).await {
                        Ok(f) => println!("success read frame: {:?}", f),
                        Err(e) => eprintln!("Failed to handle conn from: {}, {}", client_addr, e),
                    }
                });
            }
            Err(e) => eprintln!("Faield to accept conn: {}", e),
        }
    }
}
