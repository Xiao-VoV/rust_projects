// src/main.rs
use std::error::Error;
use tokio::net::TcpListener;

mod auth;
mod consts;
mod handler;
mod protocol;

use auth::UserConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("SOCKS5 Server running on {}", addr);

    //
    let config = Some(UserConfig {
        username: "root".to_string(),
        password: "1234".to_string(),
    });

    let config = std::sync::Arc::new(config);

    loop {
        let (socket, addr) = listener.accept().await?;
        let config_clone = config.clone();

        tokio::spawn(async move {
            // as_ref() 把 Option<UserConfig> 变成 Option<&UserConfig>
            if let Err(e) = handler::process(socket, config_clone.as_ref().as_ref()).await {
                eprintln!("[Error] from {:?} : {}", addr, e);
            }
        });
    }
}
