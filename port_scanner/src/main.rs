use std::{
    env,
    io::{self, Write},
    net::IpAddr,
    net::TcpStream,
    process::{self},
    str::FromStr,
    sync::mpsc::{Sender, channel},
    thread,
};

const MAX: u16 = 65535;

#[derive(Debug)]
struct Arguments {
    #[allow(unused)]
    flag: String, 
    ipAddr: IpAddr,
    thread: u8,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        #[cfg(debug_assertions)]
        {
            println!("**{:?}", args);
        }

        if args.len() < 2 {
            return Err("Don't have enough arguments!");
        } else if args.len() > 4 {
            return Err("too many arguments!");
        }
        let f = args[1].clone();
        if args.len() == 2
            && let Ok(ipadder) = IpAddr::from_str(&f)
        {
            return Ok(Arguments {
                flag: String::from(""),
                ipAddr: ipadder,
                thread: 4,
            });
        } else {
            let flag = args[2].clone();
            if flag.contains("-h") || flag.contains("help") && args.len() == 2 {
                println!(
                    "
                    Usage: -j to select how many thread you want
                    \r\n -h or -help to show help message
                "
                );
                return Err("help");
            } else if flag.contains("-j") {
                let ipaddr = match IpAddr::from_str(&args[1]) {
                    Ok(s) => s,
                    Err(_) => return Err("not fond valid Ipaddr"),
                };
                let t_ = match args[3].parse::<u8>() {
                    Ok(t) => t,
                    Err(_) => return Err("threar must > 0 and less than 256"),
                };
                Ok(Arguments {
                    flag,
                    ipAddr: ipaddr,
                    thread: t_,
                })
            } else {
                Err("invalid syntax!")
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u8) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!("*{}\n", port);
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {
                #[cfg(debug_assertions)]
                {
                    print!("{}\n", port);
                }
            }
        }

        if (MAX - port) < num_threads as u16 {
            break;
        }
        port += num_threads as u16;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    #[cfg(debug_assertions)]
    {
        println!("{:?}", args);
    }

    let programer = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprintln!("{} exit: {}", programer, err);
            process::exit(0);
        }
    });

    #[cfg(debug_assertions)]
    {
        println!("{:?}", arguments);
    }

    let num_thread = arguments.thread;
    let add = arguments.ipAddr;
    let (tx, rx) = channel();
    for i in 0..num_thread {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i as u16 + 1, add, num_thread.clone());
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        print!("open port:\n {},", v);
    }
}
