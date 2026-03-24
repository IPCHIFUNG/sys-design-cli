#!/bin/bash
# 发布构建脚本
# 用法: ./release.sh [target]
# target: windows-x86-64 (默认)
#
# 输出目录: output/{target}/

set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
OUTPUT_DIR="$ROOT_DIR/output"

# 解析参数
TARGET="${1:-windows-x86-64}"

echo "=========================================="
echo "Building release for: $TARGET"
echo "=========================================="
echo ""

cd "$ROOT_DIR/code"

# 根据目标平台构建
case "$TARGET" in
    windows-x86-64)
        BINARY_NAME="sys-design.exe"
        echo "Building Windows x86-64..."
        cargo build --release
        ;;
    *)
        echo "Unknown target: $TARGET"
        echo "Supported targets: windows-x86-64"
        exit 1
        ;;
esac

echo ""
echo "=========================================="
echo "Packaging release..."
echo "=========================================="

# 创建输出目录
PACKAGE_DIR="$OUTPUT_DIR/$TARGET"
rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"

# 复制二进制文件
cp "target/release/$BINARY_NAME" "$PACKAGE_DIR/"

# 复制文档
cp "$ROOT_DIR/README.md" "$PACKAGE_DIR/" 2>/dev/null || echo "README.md not found, skipping..."
cp "$ROOT_DIR/LICENSE" "$PACKAGE_DIR/" 2>/dev/null || echo "LICENSE not found, skipping..."

# 复制模板目录
if [ -d "$ROOT_DIR/template" ]; then
    cp -r "$ROOT_DIR/template" "$PACKAGE_DIR/"
fi

echo ""
echo "=========================================="
echo "Release package created:"
echo "  $PACKAGE_DIR/"
echo ""
ls -la "$PACKAGE_DIR/"
echo "=========================================="
