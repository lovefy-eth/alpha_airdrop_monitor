# Alpha Airdrop Monitor

一个基于 Rust 开发的空投监控机器人，用于监控 Binance Alpha 空投活动并自动推送通知到 Telegram 频道和微信企业群。

## 🚀 功能特性

- **实时监控**: 每30秒自动检查 Binance Alpha 空投活动
- **多平台通知**: 支持 Telegram 频道和微信企业群推送
- **智能去重**: 避免重复推送相同的空投信息
- **交互式命令**: 提供机器人命令接口进行状态查询和测试
- **环境配置**: 支持开发和生产环境的不同配置
- **代理支持**: 支持 HTTP/HTTPS 代理配置

## 📋 空投信息包含

- 🪙 Token 符号
- 🎁 空投数量
- 📈 积分门槛
- 💸 积分消耗
- 📦 合约地址
- 🕒 开始时间
- ⏳ 结束时间
- 🚦 活动状态

## 🛠️ 技术栈

- **语言**: Rust 2024 Edition
- **异步运行时**: Tokio
- **HTTP 客户端**: Reqwest
- **序列化**: Serde + Serde JSON
- **Telegram Bot**: Teloxide
- **日志**: Log + Env Logger
- **时间处理**: Chrono
- **环境变量**: Dotenv

## 📦 安装和运行

### 方法一：使用安装脚本（推荐）

```bash
# 下载并运行安装脚本
curl -sSL https://raw.githubusercontent.com/lovefy-eth/alpha_airdrop_monitor/main/install_and_run.sh | bash
```

安装脚本会自动：
- 安装 Rust（如果未安装）
- 克隆项目代码
- 构建项目
- 启动监控服务

### 方法二：手动安装

1. **克隆项目**
```bash
git clone https://github.com/lovefy-eth/alpha_airdrop_monitor.git
cd alpha_airdrop_monitor
```

2. **安装 Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

3. **配置环境变量**
```bash
# 复制环境配置文件
cp .env.development.local .env
# 编辑配置文件
nano .env
```

4. **构建和运行**
```bash
cargo build
./target/debug/alpha_airdrop_monitor
```

## ⚙️ 配置说明

### 环境变量配置

创建 `.env` 文件或使用现有的环境配置文件：

```bash
# Telegram Bot 配置
TELOXIDE_TOKEN=your_telegram_bot_token
TG_CHAT_ID=your_channel_chat_id
BOTID=@your_bot_username

# 微信企业群配置（可选）
WX_WEBHOOK_URL=your_wechat_webhook_url

# 代理配置（可选）
HTTPS_PROXY=http://localhost:1080
HTTP_PROXY=http://localhost:1080

# 环境标识
RUST_ENV=development  # 或 production
```

### 获取 Telegram Chat ID

使用提供的示例程序获取频道 Chat ID：

```bash
cargo run --example get_chatid
```

然后在目标频道发送消息，程序会输出 Chat ID。

## 🤖 机器人命令

机器人支持以下交互式命令：

- `/ping` - 检查机器人是否在线
- `/help` - 显示帮助信息
- `/airdrops` - 获取最近的空投列表
- `/msgtest` - 测试频道消息发送

## 📁 项目结构

```
alpha_airdrop_monitor/
├── src/
│   └── main.rs              # 主程序文件
├── examples/
│   └── get_chatid.rs        # 获取 Chat ID 的示例
├── .env                     # 环境变量配置
├── .env.development.local   # 开发环境配置
├── .env.production.local    # 生产环境配置
├── install_and_run.sh       # 安装和运行脚本
├── Cargo.toml              # Rust 项目配置
└── README.md               # 项目文档
```

## 🔧 开发

### 本地开发

1. 使用开发环境配置：
```bash
export RUST_ENV=development
cargo run
```

2. 查看日志：
```bash
tail -f output.log
```

### 构建发布版本

```bash
cargo build --release
```

## 📊 监控逻辑

1. **定时检查**: 每30秒向 Binance API 发送请求
2. **数据解析**: 解析返回的空投配置信息
3. **状态过滤**: 只处理非结束状态的空投
4. **去重处理**: 使用 HashSet 避免重复推送
5. **消息格式化**: 将空投信息格式化为易读的消息
6. **多平台推送**: 同时发送到 Telegram 和微信


## 🚨 故障排除

### 常见问题

1. **Bot Token 无效**
   - 检查 `TELOXIDE_TOKEN` 是否正确
   - 确保 Bot 已添加到目标频道

2. **Chat ID 错误**
   - 使用 `get_chatid` 示例重新获取
   - 确保 Bot 有频道发送消息权限

3. **网络连接问题**
   - 检查代理配置
   - 确认网络连接正常

4. **权限问题**
   - 确保 Bot 有发送消息权限
   - 检查频道管理员设置

### 日志查看

```bash
# 查看实时日志
tail -f output.log

# 查看错误日志
grep "ERROR" output.log
```

## 📝 更新日志

### v1.0.0
- 初始版本发布
- 支持 Telegram 和微信双平台推送
- 实现空投监控和自动通知
- 添加交互式机器人命令

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目采用 MIT 许可证。

## 📞 联系方式

如有问题或建议，请通过以下方式联系：
- GitHub Issues: [项目 Issues 页面](https://github.com/lovefy-eth/alpha_airdrop_monitor/issues)
- Telegram: @alpha_airdrop_monitor_bot
