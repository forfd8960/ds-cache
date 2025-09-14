use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

use crate::{config::CacheConfig, network::handle_conn, storage::CacheStore};

#[derive(Debug)]
pub struct Server {
    pub conf: CacheConfig,
    pub store: Arc<Mutex<CacheStore>>,
}

impl Server {
    pub fn new(conf: CacheConfig, cap: usize) -> Self {
        Self {
            conf: conf,
            store: Arc::new(Mutex::new(CacheStore::new(cap))),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let addr = self.conf.addr.clone();
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| anyhow!("Faile to listen on {}: {}", addr, e))?;

        println!("server listen on: {}", addr);

        loop {
            match listener.accept().await {
                Ok((mut sock, client_addr)) => {
                    println!("accept conn from: {}", client_addr);

                    tokio::spawn(async move {
                        match handle_conn(&mut sock).await {
                            Ok(f) => println!("success read frame: {:?}", f),
                            Err(e) => {
                                eprintln!("Failed to handle conn from: {}, {}", client_addr, e)
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Faield to accept conn: {}", e),
            }
        }
    }
}
