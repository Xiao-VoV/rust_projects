use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde_json;
use std::{
    fs,
    time::{Duration, Instant},
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Parms {
    #[arg(short, long,default_value_t = 3)]
    expire_time: u64,
    #[arg(short, long,default_value_t = 200)]
    concurrent_num: usize,
    #[arg(short, long)]
    file_path: String,
    #[arg(short,long,default_value = "./")]
    output:String,
}

#[derive(Debug)]
enum UrlStatus {
    Success(String, reqwest::StatusCode, Duration),
    Error(String, String, Duration),
}

fn arse_json_file(file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(file_path)?;
    let urls = serde_json::from_str(&data)?;
    Ok(urls)
}

fn create_shared_client(expire_time:& u64) -> Result<Client, &'static str> {
    Client::builder()
        .timeout(Duration::from_secs(*expire_time))
        .build()
        .map_err(|_| "无法创建Client")
}

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

async fn run(urls_to_check: Vec<String>, client: &Client, concurrent_requests: &usize)->Vec<UrlStatus> {
    let results_stream = stream::iter(urls_to_check)
        .map(|url| {
            let client_clone = client.clone();
            check_url(client_clone, url)
        })
        .buffer_unordered(*concurrent_requests);

    results_stream.collect().await
}

fn print_result(results:Vec<UrlStatus>){
    for result in results{
        match result{
            UrlStatus::Success(url, status, duration) => {
                println!("成功: {} - 状态码: {} - 延迟: {:?}", url, status, duration);
            }
            UrlStatus::Error(url, error, duration) => {
                println!("失败: {} - 错误: {} - 延迟: {:?}", url, error, duration);
            }
        }
    }

}
#[tokio::main]
async fn main() {
    //读取命令行参数
    let parms = Parms::parse();
    //读取json文件网址
    let client = create_shared_client(&parms.expire_time).unwrap();

    let urls_to_check = arse_json_file(&parms.file_path).unwrap();
    //
    let results = run(urls_to_check, &client, &parms.concurrent_num);
    
    print_result(results.await);
}
