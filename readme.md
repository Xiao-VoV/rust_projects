# Rust 小项目

## Rust贪吃蛇游戏

一个使用Rust语言和Piston游戏引擎开发的经典贪吃蛇游戏。

### 功能特性

1. 经典的贪吃蛇玩法
2. 平滑的控制系统
3. 随机生成食物
4. 碰撞检测(墙壁和蛇身)
5. 游戏结束自动重启

### 控制方式

↑: 向上移动
↓: 向下移动
←: 向左移动
→: 向右移动

### 安装说明

1. 确保已安装 Rust 和 Cargo
2. 克隆此仓库:
3. 编译运行:

### 项目结构

```bash
snake_game/
├── src/
│   ├── main.rs    // 游戏入口
│   ├── game.rs    // 游戏核心逻辑
│   ├── snake.rs   // 蛇的实现
│   └── draw.rs    // 绘图相关函数
└── Cargo.toml     // 项目配置和依赖
```

### 游戏截图

![alt text](./snake_game2/imgs/image.png)

#### 依赖项

piston_window - 游戏窗口和图形
rand - 随机数生成
