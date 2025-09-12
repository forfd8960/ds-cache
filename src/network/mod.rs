use anyhow::{Result, anyhow};
use futures::StreamExt;
use redis_protocol::codec::Resp2;
use redis_protocol::resp2::types::OwnedFrame as Frame;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub async fn handle_conn1(socket: &mut TcpStream) -> Result<Option<Frame>> {
    let mut framed = Framed::new(socket, Resp2::default());

    match framed.next().await {
        Some(frame_res) => match frame_res {
            Ok(ref frame) => {
                println!("read frame from framed: {:?}", frame_res);
                Ok(Some(frame.to_owned_frame()))
            }
            Err(e) => Err(anyhow!("{}", e)),
        },
        None => Err(anyhow!("No frame")),
    }
}
