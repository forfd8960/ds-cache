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

    // send_cmds(
    //     &mut framed,
    //     vec![
    //         "LPUSH mylist world",
    //         "LPUSH mylist hello",
    //         "LLEN mylist",
    //         "LPOP mylist",
    //         "LRANGE mylist 0 -1",
    //     ],
    // )
    // .await?;

    /*
        Received: Integer(1)
        Received: Integer(1)
        Received: Integer(0)
        Received: Integer(2)
        Received: [BulkString(b"world"), BulkString(b"hello")]
        Received: Integer(1)
        Received: Integer(0)
        Received: Integer(1)
        Received: [BulkString(b"world")]
    */
    send_cmds(
        &mut framed,
        vec![
            "SADD myset hello",
            "SADD myset world",
            "SADD myset hello", // duplicate
            "SCARD myset",
            "SMEMBERS myset",
            "SISMEMBER myset hello",
            "SISMEMBER myset foo",
            "SREM myset hello",
            "SMEMBERS myset",
        ],
    )
    .await?;

    Ok(())
}

async fn send_string_cmd(
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

async fn send_cmds(
    framed: &mut Framed<TcpStream, Resp2>,
    cmds: Vec<&'static str>,
) -> Result<(), Box<dyn std::error::Error>> {
    for cmd_str in cmds {
        let cmd = resp2_encode_command(cmd_str);

        framed.send(cmd).await?;
        // Read the response
        if let Some(response) = framed.next().await {
            match response? {
                BytesFrame::Array(data) => println!("Received: {:?}", data),
                BytesFrame::BulkString(data) => println!("Received: {:?}", data),
                BytesFrame::Error(e) => println!("Error: {}", e),
                other => println!("Received: {:?}", other),
            }
        }
    }

    Ok(())
}
