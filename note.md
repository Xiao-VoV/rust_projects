# 注意事项

## Cargo workspace添加项目

在 my_workspace 目录下，使用 cargo new 命令生成子项目。

```bash
    cargo new project_name --lib #如果是库项目
```

直接在子项目中添加第三方crate

```bash
    cd app
    cargo add tokio 
    cargo add tokio --features full
```

```bash
cargo new my_new_lib --vcs none
```
