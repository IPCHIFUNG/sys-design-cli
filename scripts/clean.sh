#!/bin/bash
# 清理脚本
# 用法: ./clean.sh

set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Cleaning build artifacts..."
cd "$SCRIPT_DIR/../code"
cargo clean

echo "Done!"
