use anyhow::{Result, anyhow};
use futures::{SinkExt, StreamExt};
use redis_protocol::codec::Resp2;
use std::sync::Arc;
use tokio::io;
use tokio::sync::RwLock;
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{info, warn};

use crate::commands::Command;
use crate::commands::handlers::CmdHandler;
use crate::{config::CacheConfig, storage::CacheStore};

#[derive(Debug)]
pub struct Server {
    pub conf: CacheConfig,
    pub store: Arc<RwLock<CacheStore>>,
}

impl Server {
    pub fn new(conf: CacheConfig, cap: usize) -> Self {
        Self {
            conf: conf,
            store: Arc::new(RwLock::new(CacheStore::new(cap))),
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
                Ok((socket, client_addr)) => {
                    info!("accept conn from: {}", client_addr);

                    let store = Arc::clone(&self.store);

                    tokio::spawn(async move {
                        // Split the socket into read and write halves
                        let (reader, writer) = io::split(socket);

                        // Create framed reader and writer with Resp2Codec
                        let mut framed_read = FramedRead::new(reader, Resp2::default());

                        let mut framed_write = FramedWrite::new(writer, Resp2::default());

                        loop {
                            match framed_read.next().await {
                                Some(frame_res) => match frame_res {
                                    Ok(ref frame) => {
                                        info!("read frame from framed: {:?}", frame_res);
                                        let owned_frame = frame.to_owned_frame();

                                        let cmd = Command::from(owned_frame);
                                        info!("success parsed Command: {:?}", cmd);

                                        let mut cmd_handler = CmdHandler::new(Arc::clone(&store));

                                        let cmd_res = cmd_handler.handle_cmd(cmd).await;

                                        if let Ok(write_frame) = cmd_res {
                                            let _ =
                                                framed_write.send(write_frame).await.map_err(|e| {
                                                    anyhow!("Failed to send response: {}", e)
                                                });
                                        } else {
                                            eprintln!(
                                                "failed to encode frame: {:?}",
                                                cmd_res.err()
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        warn!("fail read frame: {:?}", e);
                                        break;
                                    }
                                },
                                None => {
                                    warn!("No frame");
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => warn!("Faield to accept conn: {}", e),
            }
        }
    }
}
