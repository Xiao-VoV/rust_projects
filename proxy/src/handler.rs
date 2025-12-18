use std::error::Error;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

// 引入我们封装好的模块
use crate::auth::{self, UserConfig};
use crate::consts::*;
use crate::protocol::SocksRequest;

pub async fn process(mut socket: TcpStream, config: &UserConfig) -> Result<(), Box<dyn Error>> {
    // ==========================================
    // 阶段 1: 协商 (Handshake)
    // ==========================================

    let mut buf = [0u8; 1];
    socket.read_exact(&mut buf).await?;
    if buf[0] != SOCKS_VERSION {
        return Err("仅支持 SOCKS5 协议".into());
    }

    // 读取 NMETHODS
    let mut buf = [0u8; 1];
    socket.read_exact(&mut buf).await?;
    let nmethods = buf[0] as usize;

    let mut methods = vec![0u8; nmethods];
    socket.read_exact(&mut methods).await?;

    let mut should_auth = false;

    if let Some(_user_config) = &config.user {
        if methods.contains(&METHOD_PASSWORD) {
            should_auth = true;
            socket.write_all(&[SOCKS_VERSION, METHOD_PASSWORD]).await?;
        } else {
            socket
                .write_all(&[SOCKS_VERSION, METHOD_NO_ACCEPTABLE])
                .await?;
            return Err("client don't support auth".into());
        }
    } else {
        socket.write_all(&[SOCKS_VERSION, METHOD_NO_AUTH]).await?;
    }

    if should_auth {
        auth::perform_password_auth(&mut socket, config.user.as_ref().unwrap()).await?;
    }
    // ==========================================
    // 阶段 2: 请求 (Request) - 【核心重构点】
    // ==========================================

    let request = SocksRequest::read_from(&mut socket).await?;

    // 检查命令
    if request.cmd != CMD_CONNECT {
        warn!("unsupported command:{}", request.cmd);
        return Err("仅支持 CONNECT 命令".into());
    }

    let target = request.to_string();
    info!("Connect to: {}", target);

    // ==========================================
    // 阶段 3: 转发 (Relay)
    // ==========================================
    let connect_timeout = Duration::from_secs(config.timeout as u64);
    let server_socket_result = timeout(connect_timeout, TcpStream::connect(&target)).await;

    let mut server_socket = match server_socket_result {
        Err(_) => {
            warn!("连接目标超时 ({}s): {}", config.timeout, target);
            // 返回 0x04 (Host Unreachable) 或者 0x06 (TTL Expired)
            let reply = [
                SOCKS_VERSION,
                REP_TTL_EXPIRED,
                0x00,
                ATYP_IPV4,
                0,
                0,
                0,
                0,
                0,
                0,
            ];
            let _ = socket.write_all(&reply).await;
            return Err("连接目标超时".into());
        }
        Ok(connect_result) => match connect_result {
            Ok(s) => s,
            Err(e) => {
                let rep = match e.kind() {
                    std::io::ErrorKind::ConnectionRefused => REP_CONNECTION_REFUSED,
                    std::io::ErrorKind::TimedOut => REP_NETWORK_UNREACHABLE,
                    std::io::ErrorKind::PermissionDenied => REP_CONNECTION_NOT_ALLOWED,
                    _ => REP_HOST_UNREACHABLE,
                };
                error!("目标主机连接失败：{}({})", target, e);

                let replay = [SOCKS_VERSION, rep, 0x00, ATYP_IPV4, 0, 0, 0, 0, 0, 0];

                let _ = socket.write_all(&replay).await;
                return Err(e.into());
            }
        },
    };

    // 告诉客户端连接成功
    let reply = [
        SOCKS_VERSION,
        REP_SUCCESS,
        0x00,
        ATYP_IPV4,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    socket.write_all(&reply).await?;

    // 双向拷贝
    match tokio::io::copy_bidirectional(&mut socket, &mut server_socket).await {
        Ok((up, down)) => debug!("[Relay] 结束: 上行 {}b, 下行 {}b", up, down),
        Err(e) => warn!("[Relay] 中断: {}", e),
    }

    Ok(())
}
