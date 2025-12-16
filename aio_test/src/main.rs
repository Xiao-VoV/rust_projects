use env_logger::Env;
use log::{error, info, warn};
use std::io::{self, Read, Write};
use std::net::TcpStream;

mod runtime;

const SERVER_ADDR: &str = "127.0.0.1:8080";
fn main() -> io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("启动kqueues客户端！");
    // let poll = ;
    Ok(())
}
