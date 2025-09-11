mod commands; // handle command, SET, GET, ZADD, etc
mod config; // handle server config.
mod network; // handle network connection handler.
mod persistence; // data persistence.
mod protocol; // redis protocol decode and encode.
mod storage; // data store
mod utils; // util functions.

fn main() {
    println!("A Redis Server build with Rust");
}
