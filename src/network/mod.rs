use anyhow::{Result, anyhow};
use bytes::BytesMut;
use redis_protocol::resp2::decode;
use redis_protocol::resp2::types::OwnedFrame as Frame;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn handle_conn(socket: &mut TcpStream) -> Result<()> {
    let mut buffer = BytesMut::with_capacity(4096);
    loop {
        let bs_read = socket
            .read_buf(&mut buffer)
            .await
            .map_err(|e| anyhow!("Failed to read buffer: {}", e))?;

        if bs_read == 0 {
            return Ok(());
        }

        match parse_resp(&mut buffer) {
            Ok(Some(frame)) => {
                println!("success parsed frame: {:?}", frame);

                socket
                    .write_all("OK\r\n".as_bytes())
                    .await
                    .map_err(|e| anyhow!("Failed to write to socket: {}", e))?;
            }

            Ok(None) => continue,

            Err(e) => {
                let error_resp = format!("-ERR parsing failed: {}\r\n", e);
                socket
                    .write_all(error_resp.as_bytes())
                    .await
                    .map_err(|e| anyhow!("Failed to write error to socket: {}", e))?;
                return Err(e);
            }
        }
    }
}

fn parse_resp(buf: &mut BytesMut) -> Result<Option<Frame>> {
    match decode::decode(buf) {
        Ok(Some((frame, _))) => Ok(Some(frame)),
        Ok(None) => Ok(None),
        Err(e) => Err(anyhow!("Failed to decode frame: {}", e)),
    }
}
