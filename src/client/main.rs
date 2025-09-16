use futures::{SinkExt, StreamExt};
use redis_protocol::{
    codec::{Resp2, resp2_encode_command},
    resp2::types::BytesFrame,
};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Redis server
    let stream = TcpStream::connect("127.0.0.1:6869").await?;

    // Create framed stream with our RESP codec
    let mut framed = Framed::new(stream, Resp2::default());

    send_cmd(&mut framed, "SET Hello 5").await?;

    send_cmd(&mut framed, "GET Hello").await?;

    Ok(())
}

async fn send_cmd(
    framed: &mut Framed<TcpStream, Resp2>,
    cmds: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = resp2_encode_command(cmds);

    framed.send(cmd).await?;

    // Read the response
    if let Some(response) = framed.next().await {
        match response? {
            BytesFrame::SimpleString(data) => println!("Received: {:?}", data),
            BytesFrame::BulkString(data) => println!("Received: {:?}", data),
            BytesFrame::Error(e) => println!("Error: {}", e),
            other => println!("Received: {:?}", other),
        }
    }

    Ok(())
}
