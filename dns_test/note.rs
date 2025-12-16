use futures::stream::{self, StreamExt};
use reqwest::Client;
use std::time::{Duration, Instant}; // 引入 Instant
use std::fs; // 引入标准库的文件系统模块

// --- 配置项 ---
const JSON_FILE_PATH: &str = "websites.json"; // JSON 文件路径
const CONCURRENT_REQUESTS: usize = 100;
const REQUEST_TIMEOUT_SECONDS: u64 = 5;
// --- 配置项结束 ---

// 定义一个枚举来更清晰地表示结果
#[derive(Debug)]
enum UrlStatus {
    /// 成功连接，并获取到 HTTP 状态码和延迟
    Success(String, reqwest::StatusCode, Duration),
    /// 连接失败 (超时, DNS错误, 拒绝连接等) 和所用时间
    Error(String, String, Duration),
}

/**
 * 核心函数：异步检查单个 URL 并测量延迟
 */
async fn check_url(client: Client, url: String) -> UrlStatus {
    let start_time = Instant::now(); // 在请求开始前记录时间

    match client.head(&url).send().await {
        Ok(response) => {
            let elapsed = start_time.elapsed(); // 计算总耗时
            UrlStatus::Success(url, response.status(), elapsed)
        }
        Err(e) => {
            let elapsed = start_time.elapsed(); // 即使失败也计算耗时
            UrlStatus::Error(url, e.to_string(), elapsed)
        }
    }
}

/**
 * 新增：从 JSON 文件加载 URL 列表
 */
fn load_urls_from_json(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let urls: Vec<String> = serde_json::from_str(&data)?;
    Ok(urls)
}

#[tokio::main]
async fn main() {
    // --- 步骤 1: 从 JSON 文件加载 URL 列表 ---
    let urls_to_check = match load_urls_from_json(JSON_FILE_PATH) {
        Ok(urls) => {
            if urls.is_empty() {
                println!("错误: JSON 文件 '{}' 为空。", JSON_FILE_PATH);
                return;
            }
            urls
        }
        Err(e) => {
            println!("加载 JSON 文件 '{}' 失败: {}", JSON_FILE_PATH, e);
            println!("请确保文件存在且格式正确，例如: [\"https://google.com\"]");
            return;
        }
    };

    println!(
        "从 {} 加载了 {} 个网站。",
        JSON_FILE_PATH,
        urls_to_check.len()
    );
    println!(
        "开始检测... (并发数: {}, 超时: {}s)",
        CONCURRENT_REQUESTS, REQUEST_TIMEOUT_SECONDS
    );

    // --- 步骤 2: 创建一个共享的 HTTP Client ---
    let client = Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .build()
        .expect("无法创建 HTTP client");

    // --- 步骤 3: 异步并发处理 (与之前相同) ---
    let results_stream = stream::iter(urls_to_check)
        .map(|url| {
            let client_clone = client.clone();
            check_url(client_clone, url)
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    // --- 步骤 4: 收集结果 (与之前相同) ---
    let results: Vec<UrlStatus> = results_stream.collect().await;

    // --- 步骤 5: 打印报告 (更新以显示延迟) ---
    println!("\n--- 检测完成 ---");

    let mut successful_links = 0;
    let mut problematic_links = 0;

    for status in results {
        match status {
            UrlStatus::Success(url, status_code, duration) => {
                let duration_ms = duration.as_millis();
                
                if status_code.is_success() {
                    println!("[正常] {} - 状态: {} ({}ms)", url, status_code, duration_ms);
                    successful_links += 1;
                } else {
                    println!("[注意] {} - 状态: {} ({}ms)", url, status_code, duration_ms);
                    problematic_links += 1;
                }
            }
            UrlStatus::Error(url, err_msg, duration) => {
                let duration_ms = duration.as_millis();
                let short_error = err_msg.lines().next().unwrap_or("未知错误");
                
                // 特别标记超时
                if err_msg.contains("timeout") {
                    println!("[超时] {} - 超过 {}s (耗时: {}ms)", url, REQUEST_TIMEOUT_SECONDS, duration_ms);
                } else {
                    println!("[错误] {} - 无法访问: {} (耗时: {}ms)", url, short_error, duration_ms);
                }
                problematic_links += 1;
            }
        }
    }

    println!("\n--- 总结 ---");
    println!("正常链接 (2xx): {}", successful_links);
    println!("异常链接 (4xx, 5xx, 或连接错误): {}", problematic_links);
}