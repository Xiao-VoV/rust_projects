use colored::*;
use hickory_resolver::TokioAsyncResolver;
use hickory_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::task;
use tokio::time::Instant;

struct DNSResult {
    server_ip: String,
    domain: String,
    latency: Duration,
    success: bool,
    ips: Vec<String>,
    error_msg: Option<String>,
}

async fn test_dns(server: String, domain: String) -> DNSResult {
    let ip_res: Result<IpAddr, _> = server.parse();
    if ip_res.is_err() {
        return DNSResult {
            server_ip: server,
            domain,
            latency: Duration::MAX,
            success: false,
            ips: vec![],
            error_msg: Some("无效的服务器".to_string()),
        };
    }
    let socket_addr = SocketAddr::new(ip_res.unwrap(), 53);

    let mut config = ResolverConfig::new();
    config.add_name_server(NameServerConfig::new(socket_addr, Protocol::Udp));

    let mut opts = ResolverOpts::default();
    opts.timeout = Duration::from_millis(2000);
    opts.attempts = 1;

    let resolver = TokioAsyncResolver::tokio(config, opts);

    let start = Instant::now();

    match resolver.lookup_ip(domain.clone()).await {
        Ok(response) => {
            let duration = start.elapsed();
            let ips: Vec<String> = response.iter().map(|ip| ip.to_string()).collect();
            DNSResult {
                server_ip: server,
                domain,
                latency: duration,
                success: true,
                ips,
                error_msg: None,
            }
        }
        Err(_) => {
            let duration = start.elapsed();
            DNSResult {
                server_ip: server,
                domain,
                latency: duration,
                success: true,
                ips: vec![],
                error_msg: None,
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let dns_servers = vec![
        "114.114.114.114", // 国内通用
        "223.5.5.5",       // 阿里 DNS
        "119.29.29.29",    // 腾讯 DNS
        "8.8.8.8",         // Google
        "1.1.1.1",         // Cloudflare
        "180.76.76.76",    // 百度 DNS
        "1.2.3.4",         // 一个故意错误的 DNS 用于测试
    ];

    let test_domains = vec![
        "www.baidu.com",
        "www.google.com",
        "github.com",
        "rust-lang.org",
    ];

    println!("🚀 开始 DNS 延迟基准测试...");
    println!(
        "待测服务器: {} 个, 待测域名: {} 个\n",
        dns_servers.len(),
        test_domains.len()
    );

    let mut tasks = Vec::new();

    for server in &dns_servers {
        for domain in &test_domains {
            let s = server.to_string();
            let d = domain.to_string();

            tasks.push(task::spawn(async move { test_dns(s, d).await }));
        }
    }

    let mut result = Vec::new();

    for task in tasks {
        if let Ok(res) = task.await {
            result.push(res);
        }
    }

    result.sort_by(|a, b| {
        if a.success != b.success {
            return b.success.cmp(&a.success);
        }
        if a.server_ip != b.server_ip {
            return a.server_ip.cmp(&b.server_ip);
        }
        a.latency.cmp(&b.latency)
    });

    print_table(result);
}

fn print_table(results: Vec<DNSResult>) {
    println!(
        "{:<16} | {:<20} | {:<10} | {:<10} | {}",
        "DNS Server", "Domain", "Status", "Latency", "Result / Error"
    );
    println!("{}", "-".repeat(90));

    let mut server_ip = String::new();

    for res in results {
        // 如果换了域名，打印一个空行分隔，视觉清晰
        if res.server_ip != server_ip {
            if !server_ip.is_empty() {
                println!("{}", "-".repeat(90));
            }
            server_ip = res.server_ip.clone();
        }

        let status_str = if res.success {
            "SUCCESS".green()
        } else {
            "FAILED".red()
        };

        // 延迟显示颜色：<50ms 绿色, <200ms 黄色, >200ms 红色
        let latency_ms = res.latency.as_millis();
        let latency_str = format!("{} ms", latency_ms);
        let latency_colored = if !res.success {
            latency_str.normal()
        } else if latency_ms < 50 {
            latency_str.green()
        } else if latency_ms < 200 {
            latency_str.yellow()
        } else {
            latency_str.red()
        };

        let detail = if res.success {
            // 只显示第一个解析到的 IP，防止太长
            format!("IP: {:?}...", res.ips.first().unwrap_or(&String::from("?")))
        } else {
            format!("Error: {}", res.error_msg.unwrap_or_default())
        };

        println!(
            "{:<16} | {:<20} | {:<10} | {:<10} | {}",
            res.server_ip, res.domain, status_str, latency_colored, detail
        );
    }
}
