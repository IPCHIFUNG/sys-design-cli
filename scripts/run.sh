#!/bin/bash
# 运行脚本
# 用法: ./run.sh [command] [args...]
# 示例:
#   ./run.sh init --name my-project
#   ./run.sh --src model.yaml context-model list relations
#   ./run.sh --src model.yaml generate -o output.puml

set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$SCRIPT_DIR/../code/target/release/sys-design"

# 检查二进制是否存在
if [ ! -f "$BINARY" ]; then
    echo "Binary not found. Building release..."
    cd "$SCRIPT_DIR/../code"
    cargo build --release
fi

# 运行命令
echo "Running: $BINARY $@"
echo ""
exec "$BINARY" "$@"
