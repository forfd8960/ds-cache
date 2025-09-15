use anyhow::{Result, anyhow};
use futures::{SinkExt, StreamExt};
use redis_protocol::codec::Resp2;
use std::sync::Arc;
use tokio::io;
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::commands::Command;
use crate::commands::handlers::CmdHandler;
use crate::protocol::encode::{encode_error, encode_value};
use crate::{config::CacheConfig, storage::CacheStore};

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
                Ok((mut socket, client_addr)) => {
                    println!("accept conn from: {}", client_addr);

                    let store = Arc::clone(&self.store);

                    tokio::spawn(async move {
                        // Split the socket into read and write halves
                        let (reader, writer) = io::split(socket);

                        // Create framed reader and writer with Resp2Codec
                        let mut framed_read = FramedRead::new(reader, Resp2::default());

                        let mut framed_write = FramedWrite::new(writer, Resp2::default());

                        match framed_read.next().await {
                            Some(frame_res) => match frame_res {
                                Ok(ref frame) => {
                                    println!("read frame from framed: {:?}", frame_res);
                                    let owned_frame = frame.to_owned_frame();

                                    let cmd = Command::from(owned_frame);
                                    let mut store = store.lock().await;

                                    let entry_res = CmdHandler::handle_cmd(cmd, &mut store);

                                    let encode_frame = match entry_res {
                                        Ok(res) => encode_value(res.value),
                                        Err(e) => encode_error(e.to_string()),
                                    };

                                    if let Ok(write_frame) = encode_frame {
                                        let _ = framed_write
                                            .send(write_frame.into_bytes_frame())
                                            .await
                                            .map_err(|e| anyhow!("Failed to send response: {}", e));
                                    } else {
                                        eprintln!(
                                            "failed to encode frame: {:?}",
                                            encode_frame.err()
                                        );
                                    }
                                }
                                Err(e) => eprintln!("fail read frame: {:?}", e),
                            },
                            None => eprintln!("No frame"),
                        }
                    });
                }
                Err(e) => eprintln!("Faield to accept conn: {}", e),
            }
        }
    }
}
