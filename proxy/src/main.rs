use std::error::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("监听在{:?}", &listener.local_addr().unwrap());

    loop {
        let (client_socket, addr) = listener.accept().await?;
        println!("接收到来自: {:?} 的连接", addr);
        tokio::spawn(async move {
            if let Err(e) = process(client_socket).await {
                eprintln!("处理连接出错:{}", e);
            }
        });
    }
}

async fn process(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    // ------------------------------------------

    /*
     *  客户端发送
     *  +----+----------+----------+
     *  |VER | NMETHODS | METHODS  |
     *  +----+----------+----------+
     *  | 1  |    1     | 1 to 255 |
     *  +----+----------+----------+
     *  服务器响应
     *  +----+--------+
     *  |VER | METHOD |
     *  +----+--------+
     *  | 1  |   1    |
     *  +----+--------+
     *
     */
    let mut buf = [0u8; 1];
    socket.read_exact(&mut buf).await?;
    if buf[0] != 0x05 {
        return Err("仅支持 SOCKS5".into());
    }

    let mut buf = [0u8; 1];
    socket.read_exact(&mut buf).await?;
    let n_methods = buf[0] as usize;

    let mut methods = vec![0u8; n_methods];
    socket.read_exact(&mut methods).await?;

    socket.write_all(&[0x05, 0x00]).await?;

    // ------------------------------------------

    /*
    客户端发送
    +----+-----+-------+------+----------+----------+
    |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
    +----+-----+-------+------+----------+----------+
    | 1  |  1  | X'00' |  1   | Variable |    2     |
    +----+-----+-------+------+----------+----------+
    服务器响应
    +----+-----+-------+------+----------+----------+
    |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
    +----+-----+-------+------+----------+----------+
    | 1  |  1  | X'00' |  1   | Variable |    2     |
    +----+-----+-------+------+----------+----------+
    */
    let mut head = [0u8; 4];
    socket.read_exact(&mut head).await?;

    let ver = head[0];
    let cmd = head[1];
    let _rev = head[2];
    let atyp = head[3];

    if ver != 0x05 {
        return Err(format!("不支持的协议版本:0x{:02x},仅支持Socks5", ver).into());
    }

    if cmd != 0x01 {
        return Err(format!("仅支持 CONNECT 命令, cmd: 0x{:02x}", cmd).into());
    }

    let target_addr: String = match atyp {
        0x01 => {
            // ipv4
            let mut buf = [0u8; 4];
            socket.read_exact(&mut buf).await?;
            format!("{}.{}.{}.{}", buf[0], buf[1], buf[2], buf[3])
        }
        0x03 => {
            // 域名
            let mut len_buf = [0x8; 1];
            socket.read_exact(&mut len_buf).await?;
            let len = len_buf[0] as usize;

            let mut buf = vec![0u8; len];
            socket.read_exact(&mut buf).await?;

            String::from_utf8_lossy(&buf).to_string()
        }
        0x04 => {
            // ipv6地址
            let mut buf = [0u8; 16];
            socket.read_exact(&mut buf).await?;

            use std::net::Ipv6Addr;
            let addr = Ipv6Addr::from(buf);
            format!("[{}]", addr)
        }
        _ => return Err(format!("未知的地址类型: {}", atyp).into()),
    };

    let mut port_buf = [0u8; 2];
    socket.read_exact(&mut port_buf).await?;
    let port = u16::from_be_bytes(port_buf);

    let target = format!("{}:{}", target_addr, port);

    let mut server_socket = match TcpStream::connect(&target).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("无法连接目标服务器:{}--{}", &target, e);
            return Err(e.into());
        }
    };

    let replay = [0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    socket.write_all(&replay).await?;

    // ------------------------------------------

    println!("开始转发:Client <-> {}", target);
    match tokio::io::copy_bidirectional(&mut socket, &mut server_socket).await {
        Ok((up, down)) => {
            println!("转发结束：上行{}b, 下行{}b", up, down);
        }
        Err(e) => eprintln!("转发错误:{}", e),
    }
    Ok(())
}
