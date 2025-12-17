use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("监听在{:?}", &listener.local_addr().unwrap());

    loop {
        let (mut client_socket, addr) = listener.accept().await?;
        tokio::spawn(async move {
            println!("输入:{:?}", addr);
            let mut server_socket = match TcpStream::connect("183.2.172.177:80").await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("传出连接失败:{}", e);
                    return;
                }
            };

            match tokio::io::copy_bidirectional(&mut client_socket, &mut server_socket).await {
                Ok((to_server, to_client)) => {
                    println!(
                        "Relay finished: Up: {} bytes, Down: {} bytes",
                        to_server, to_client
                    );
                }
                Err(e) => eprintln!("Relay error: {}", e),
            }
        });
    }
}
