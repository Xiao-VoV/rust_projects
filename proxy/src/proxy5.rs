use std::{error::Error, vec};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Proxy {
    VER: u8,
    NMETHODS: u8,
    METHODS: Vec<u8>,
}

pub async fn auth(
    socket: &mut TcpStream,
    password_: &str,
    username_: &str,
) -> Result<(), Box<dyn Error>> {
    let mut header = [0u8; 2];
    socket.read_exact(&mut header).await?;

    let auth_ver = header[0];
    let ulen = header[1] as usize;

    if auth_ver != 0x01 {
        return Err(format!("wrong auth version {}", auth_ver).into());
    }

    // get username
    let mut username_buf = vec![0u8; ulen];
    socket.read_exact(&mut username_buf).await?;
    let username = String::from_utf8(username_buf).unwrap_or("".to_string());

    //get password
    let mut plen_buf = [0u8; 1];
    socket.read_exact(&mut plen_buf).await?;
    let plen = plen_buf[0] as usize;

    let mut pass_buf = vec![0u8; plen];
    socket.read_exact(&mut pass_buf).await?;
    let password = String::from_utf8(pass_buf).unwrap_or("".to_string());

    println!("tring to auth user: {}", &username);
    if username == username_ && password == password_ {
        socket.write_all(&[0x01, 0x00]).await?;
        return Ok(());
    } else {
        socket.write_all(&[0x01, 0x01]).await?;
        Err("fail to auth!".into())
    }
}
