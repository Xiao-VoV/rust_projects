use std::error::Error;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::consts::*;

#[derive(Debug)]
pub enum Address {
    IpV4(Ipv4Addr),
    Domain(String),
    IpV6(Ipv6Addr),
}

#[derive(Debug)]
pub struct SocksRequest {
    pub cmd: u8,
    pub address: Address,
    pub port: u16,
}

impl fmt::Display for SocksRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.address {
            Address::IpV4(ip) => write!(f, "{}:{}", ip, self.port),
            Address::Domain(domain) => write!(f, "{}:{}", domain, self.port),
            Address::IpV6(ip) => write!(f, "[{}],{}", ip, self.port),
        }
    }
}

impl SocksRequest {
    pub async fn read_from(socket: &mut TcpStream) -> Result<Self, Box<dyn Error>> {
        let mut head = [0u8; 4];
        socket.read_exact(&mut head).await?;

        let ver = head[0];
        let cmd = head[1];
        let atyp = head[3];

        if ver != SOCKS_VERSION {
            return Err(format!("unsupport socks version: 0x{:02x}", ver).into());
        }

        let address = match atyp {
            ATYP_IPV4 => {
                let mut buf = [0u8; 4];
                socket.read_exact(&mut buf).await?;
                Address::IpV4(Ipv4Addr::from(buf))
            }
            ATYP_DOMAIN => {
                // 先读 1 字节长度
                let mut len_buf = [0u8; 1];
                socket.read_exact(&mut len_buf).await?;
                let len = len_buf[0] as usize;

                // 再读 N 字节域名
                let mut buf = vec![0u8; len];
                socket.read_exact(&mut buf).await?;

                // 转换成 String
                let domain = String::from_utf8(buf).map_err(|_| "wrong domain")?;
                Address::Domain(domain)
            }
            ATYP_IPV6 => {
                let mut buf = [0u8; 16];
                socket.read_exact(&mut buf).await?;
                Address::IpV6(Ipv6Addr::from(buf))
            }
            _ => return Err(format!("unknow address type: 0x{:02x}", atyp).into()),
        };

        let mut port_buf = [0u8; 2];
        socket.read_exact(&mut port_buf).await?;
        let port = u16::from_be_bytes(port_buf);

        Ok(SocksRequest { cmd, address, port })
    }
}
