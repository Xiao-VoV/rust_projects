use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::Duration;
use std::{clone, thread};

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: u8 = 32;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Listener failed to bind");
    client
        .set_nonblocking(true)
        .expect("failed to initialize non_blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        loop {
            let mut buff = vec![0, MSG_SIZE];
            match client.read_exact(&mut buff) {
                Ok(_) => {
                    let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    println!("message recv {:?}", msg);
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    print!("connection with server was severed");
                    break;
                }
            }

            match rx.try_recv() {
                Ok(msg)=>{
                    let mut buff  = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE as usize, 0);
                    client.write_all(&buff).expect("writing to socket failed");
                    println!("message send {:?}",msg);
                }
                Err(_)=>{

                }
                
            }
            thread::sleep(Duration::from_micros(100));
        }
    });


    println!("write a message:");
    loop{
        let mut buff = String::new ();
        io::stdin().read_line(&mut buff).expect("read from stdin failed");
        let msg = buff.trim().to_string();

        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }
    println!("bye bye!")
}
