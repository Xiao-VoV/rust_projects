// src/auth.rs
use crate::consts::*;
use std::error::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

// 简单的用户配置结构
#[derive(Debug, Clone)]
pub struct UserConfig {
    pub username: String,
    pub password: String,
}

pub async fn perform_password_auth(
    socket: &mut TcpStream,
    user_config: &UserConfig,
) -> Result<(), Box<dyn Error>> {
    // 1. 读取版本号和用户名长度 [VER, ULEN]
    let mut header = [0u8; 2];
    socket.read_exact(&mut header).await?;

    let ver = header[0];
    let ulen = header[1] as usize;

    if ver != AUTH_VERSION {
        return Err(format!("unsupport auth version: {}", ver).into());
    }

    // 2. 读取用户名
    let mut user_buf = vec![0u8; ulen];
    socket.read_exact(&mut user_buf).await?;
    let username = String::from_utf8(user_buf).unwrap_or_default();

    // 3. 读取密码长度
    let mut plen_buf = [0u8; 1];
    socket.read_exact(&mut plen_buf).await?;
    let plen = plen_buf[0] as usize;

    // 4. 读取密码
    let mut pass_buf = vec![0u8; plen];
    socket.read_exact(&mut pass_buf).await?;
    let password = String::from_utf8(pass_buf).unwrap_or_default();

    println!("[Auth] 尝试认证: {} / ***", username);

    // 5. 校验
    if username == user_config.username && password == user_config.password {
        socket.write_all(&[AUTH_VERSION, AUTH_SUCCESS]).await?;
        Ok(())
    } else {
        socket.write_all(&[AUTH_VERSION, AUTH_FAILURE]).await?;
        Err("身份验证失败".into())
    }
}
