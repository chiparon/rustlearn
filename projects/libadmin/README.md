# 图书馆管理系统

这是按 `projects/数据库实验三报告.docx` 实现的 Rust + SQLite 本地图书馆管理系统。

## 技术栈

- Rust 2024
- Axum Web 框架
- rusqlite + SQLite
- Cookie 会话

## 运行

Windows 侧当前没有 `cargo`，请用 WSL 运行：

```powershell
wsl.exe -- bash -lc 'cd /mnt/e/Eproject/rustlearn/projects/libadmin && cargo run'
```

启动后访问：

```text
http://127.0.0.1:8088
```

若当前 WSL 的 localhost 转发不可用，先查看 WSL IP：

```powershell
wsl.exe --exec hostname -I
```

然后访问：

```text
http://<WSL_IP>:8088
```

本机当前验证可用地址是 `http://192.168.192.189:8088`。若 8088 被占用，程序会自动尝试 8089 到 8098。

## 测试账号

- 管理员：`A001` / `admin123`
- 普通读者：`R001` / `reader001`

## 功能范围

- 读者注册、读者登录、管理员登录、会话 Cookie
- 读者个人信息维护、账号注销校验
- 图书多条件查询、借阅、归还、续借
- 管理员维护读者、图书、管理员账号
- 借阅/归还记录查询与管理员代办借还
- 超期、损坏、丢失等异常申报、赔偿处理、状态办结
- SQLite 自动建表、初始化 220 本图书、80 名读者、4 个管理员和测试借还/异常数据
- 每日自动备份和管理员手动备份，备份目录为 `data/backups`

## 验证

```powershell
wsl.exe -- bash -lc 'cd /mnt/e/Eproject/rustlearn/projects/libadmin && cargo fmt --check && cargo check'
```
