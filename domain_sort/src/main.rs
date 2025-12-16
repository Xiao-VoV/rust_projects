use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    process,
};

use clap::Parser;

// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    thread: u16,

    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,
}

fn readfile(input_file: &str) -> Result<BufReader<File>, &str> {
    println!("reading file {}...", input_file);
    if !Path::new(input_file).exists() {
        eprintln!("{}：文件不存在！", input_file);
        return Err("输入文件未找到");
    }

    let input = File::open(input_file).map_err(|_| "文件无法打开!")?;
    let read = BufReader::new(input);
    Ok(read)
}

fn valid(reader: BufReader<File>) -> Result<Vec<String>, &'static str> {
    let mut valid_domains: Vec<String> = Vec::new();
    for line_result in reader.lines() {
        let line = line_result.map_err(|_| "行错误")?;
        let domain = line.trim();

        // 1. 检查是否以 .xyz 结尾，并提取前缀
        if let Some(prefix) = domain.strip_suffix(".xyz") {
            // 2. 检查前缀长度是否在 6 到 9 之间（包含）
            let len = prefix.len();
            if (6..=9).contains(&len) {
                // 3. 检查前缀是否只包含数字
                if prefix.chars().all(|c| c.is_digit(10)) {
                    // 如果所有条件都满足，则保留该域名
                    valid_domains.push(domain.to_string());
                }
            }
        }
    }
    Ok(valid_domains)
}

fn save(output: &str, mut result: Vec<String>) -> Result<(), &'static str> {
    result.sort_by_key(|s| s.len());
    let mut output_file = File::create(output).map_err(|_| "无法创建文件")?;

    for domain in result {
        writeln!(output_file, "{}", domain);
    }
    Ok(())
}

fn run(args: &Args) -> Result<(), &str> {
    let reader = readfile(&args.input)?;
    let domains = valid(reader)?;
    save(&args.output, domains)?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    #[cfg(debug_assertions)]
    {
        println!("{:?}", args);
    }

    if let Err(e) = run(&args) {
        eprintln!("错误: {}", e);
        process::exit(1);
    }
}
