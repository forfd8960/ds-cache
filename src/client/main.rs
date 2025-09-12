use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use redis_protocol::{codec::Resp2, resp2::types::BytesFrame};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Redis server
    let stream = TcpStream::connect("127.0.0.1:6869").await?;

    // Create framed stream with our RESP codec
    let mut framed = Framed::new(stream, Resp2::default());

    // Send a PING command
    let ping_command = BytesFrame::Array(vec![BytesFrame::BulkString(Bytes::from("PING"))]);

    framed.send(ping_command).await?;

    // Read the response
    if let Some(response) = framed.next().await {
        match response? {
            BytesFrame::SimpleString(bs) => println!("Received: {:?}", bs),
            BytesFrame::Error(e) => println!("Error: {}", e),
            other => println!("Received: {:?}", other),
        }
    }

    Ok(())
}
