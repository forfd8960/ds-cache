use futures::{SinkExt, StreamExt};
use redis_protocol::{
    codec::{Resp2, resp2_encode_command},
    resp2::types::BytesFrame,
};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

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
    // send_cmds(
    //     &mut framed,
    //     vec![
    //         "SADD myset hello",
    //         "SADD myset world",
    //         "SADD myset hello", // duplicate
    //         "SCARD myset",
    //         "SMEMBERS myset",
    //         "SISMEMBER myset hello",
    //         "SISMEMBER myset foo",
    //         "SREM myset hello",
    //         "SMEMBERS myset",
    //     ],
    // )
    // .await?;

    // Send hash commands
    send_cmds(
        &mut framed,
        vec![
            "HSET myhash field1 value1",
            "HSET myhash field2 value2",
            "HGET myhash field1",
            "HGET myhash field2",
            "HGET myhash field3", // non-existing field
            "HMSET alice:1 name Alice age 30 city Wonderland",
            "HMGET alice:1 name age city country", // country does not exist
            "HLEN myhash",
            "HKEYS myhash",
            "HVALS myhash",
            "HGETALL myhash",
            "HEXISTS myhash field1",
            "HEXISTS myhash field3",
            "HDEL myhash field1",
            "HGETALL myhash",
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

    framed.send(cmd.clone()).await?;

    // Read the response
    if let Some(response) = framed.next().await {
        match response? {
            BytesFrame::SimpleString(data) => info!("Cmd: {:?}, Received: {:?}", cmd, data),
            BytesFrame::BulkString(data) => info!("Cmd: {:?}, Received: {:?}", cmd, data),
            BytesFrame::Error(e) => info!("Error: {}", e),
            other => info!("Received: {:?}", other),
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

        framed.send(cmd.clone()).await?;
        // Read the response
        if let Some(response) = framed.next().await {
            match response? {
                BytesFrame::Array(data) => info!("Cmd: {:?}, Received: {:?}", cmd, data),
                BytesFrame::BulkString(data) => info!("Cmd: {:?}, Received: {:?}", cmd, data),
                BytesFrame::Error(e) => println!("Error: {}", e),
                other => info!("Cmd: {:?}, Received: {:?}", cmd, other),
            }
        }
    }

    Ok(())
}
