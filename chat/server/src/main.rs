use std::io::{ErrorKind,Read,Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;


const LOCAL:&str = "127.0.0.1:6000";
const MSG_SIZE:u8 = 32;

fn sleep(){
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server.set_nonblocking(true).expect("failed to initialize non_blocking");

    let mut clinet = vec![];

    let (tx,rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket,addr)) = server.accept(){
            println!("client {} connected",addr);

            let tx = tx.clone();
            clinet.push(socket.try_clone().expect("failed to clone client"));


            thread::spawn( move || loop{
                let mut buff: Vec<u8> = vec![0,MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_)=>{
                        let msg = buff.into_iter().take_while(|&x|x!=0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message!");

                        println!("{}: {}",addr,msg);
                        tx.send(msg).expect("failed to send msg to rx");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock=>(),
                    Err(_)=>{
                        println!("closing connection with: {}",addr);
                        break;
                    }
                }
                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv(){
            clinet = clinet.into_iter().filter_map(|mut client|{
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE as usize, 0);
                

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }
    }
}
