use clap::Parser;
use core::str;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::task;
use tokio::time::{self, Duration};

#[derive(Parser, Debug)]
#[command(author,version,about,long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: String,
    #[arg(short,long,value_parser=parse_port_range)]
    ports: (u16, u16),
    #[arg(short, long, default_value_t = 3000)]
    timeout: u64,
    #[arg(long, default_value_t = 1000)]
    concurrency: usize,
}

fn parse_port_range(s: &str) -> Result<(u16, u16), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("端口格式错误：{},(例如:1024-5000)", s));
    }
    let start = parts[0].parse::<u16>().map_err(|_| "无效的开始端口")?;
    let end = parts[1].parse::<u16>().map_err(|_| "无效的结束端口")?;

    if start > end {
        return Err("开始端口不能大于结束端口".to_string());
    }
    Ok((start, end))
}

async fn check_port(host: String, port: u16, timeout_duration: Duration) -> bool {
    let target = format!("{}:{}", host, port);

    match time::timeout(timeout_duration, TcpStream::connect(target)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

#[tokio::main]
async fn main() {

    let args = Args::parse();

    let (start_port, end_port) = args.ports;
    let semaphore = Arc::new(Semaphore::new(args.concurrency));
    let timeout_duration = Duration::from_millis(args.timeout);

    println!("开始扫描 {}:{}-{}", args.ip.clone(), start_port, end_port);
    let mut tasks = Vec::new();
    for port in start_port..=end_port {
        let host = args.ip.clone();
        let permit = semaphore.clone();
        let task = task::spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            if check_port(host, port, timeout_duration).await {
                println!("发现开放端口：{}", port);
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    println!("扫描完毕！...")
}
