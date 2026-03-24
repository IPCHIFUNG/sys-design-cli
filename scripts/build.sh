#!/bin/bash
# 构建脚本
# 用法: ./build.sh [--release]

set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../code"

if [ "$1" == "--release" ]; then
    echo "Building release..."
    cargo build --release
    echo ""
    echo "Done! Binary: target/release/sys-design.exe"
else
    echo "Building debug..."
    cargo build
    echo ""
    echo "Done! Binary: target/debug/sys-design.exe"
fi
