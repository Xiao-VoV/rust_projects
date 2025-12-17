# Rust Prox 代理工具

## 第一阶段

最简单的TCP端口转发工具，接收到一个TCP请求，无脑的转发到百度

```bash
    curl -v -H "Host: example.com" http://127.0.0.1:8080
```

## 第二阶段

一个简单的Socks5代理服务器

```bash
    curl -v --socks5-hostname 127.0.0.1:8080 http://www.baidu.com
```
