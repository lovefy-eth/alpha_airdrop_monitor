#!/bin/bash

set -e

# 设置环境变量
export PATH="$HOME/.cargo/bin:$PATH"
REPO_URL="https://github.com/lovefy-eth/alpha_airdrop_monitor.git"
DIR_NAME="alpha_airdrop_monitor"
BINARY_NAME="alpha_airdrop_monitor" # 默认构建后的可执行文件名

# 安装 Rust（如果未安装）
if ! command -v rustc >/dev/null 2>&1; then
    echo "正在安装 Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# 克隆或强制更新项目
if [ -d "$DIR_NAME" ]; then
    echo "项目已存在，强制更新..."
    cd "$DIR_NAME"
    git reset --hard
    git pull origin main
else
    echo "克隆项目..."
    git clone "$REPO_URL"
    cd "$DIR_NAME"
fi

# 检查是否已有程序运行并终止
echo "检查旧进程..."
OLD_PID=$(pgrep -f "target/debug/$BINARY_NAME" || true)
if [ -n "$OLD_PID" ]; then
    echo "检测到进程 $OLD_PID 正在运行，正在终止..."
    kill -9 $OLD_PID
    sleep 1
fi

chmod +x install_and_run.sh

# 构建项目
echo "构建中..."
cargo build

# 启动程序（后台运行）
echo "启动程序..."
nohup ./target/debug/$BINARY_NAME > output.log 2>&1 &

echo "✅ 启动完成，日志写入 output.log"
