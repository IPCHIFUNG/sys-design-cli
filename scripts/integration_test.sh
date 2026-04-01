#!/bin/bash
# integration_test.sh — sys-design CLI 集成测试主入口
# 用法: bash scripts/integration_test.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

# ————— 配置 —————
CLI="$PROJECT_DIR/code/target/release/sys-design"
WORKSPACE="$PROJECT_DIR/integration_test_workspace.yaml"

# ————— 构建检查 —————
if [ ! -f "$CLI" ]; then
    echo "Release binary not found. Building..."
    cd "$PROJECT_DIR/code" && cargo build --release
    cd "$PROJECT_DIR"
fi

# ————— 清理旧文件 —————
rm -f "$WORKSPACE"

# ————— 加载辅助函数 —————
source "$SCRIPT_DIR/tests/common.sh"

echo ""
echo -e "${YELLOW}═══════════════════════════════════════════${NC}"
echo -e "${YELLOW}  sys-design 集成测试${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════${NC}"

# ————— 依次执行各测试文件 —————
source "$SCRIPT_DIR/tests/test_context_model.sh"
source "$SCRIPT_DIR/tests/test_concept_model.sh"
source "$SCRIPT_DIR/tests/test_logic_model.sh"
source "$SCRIPT_DIR/tests/test_runtime_model.sh"
source "$SCRIPT_DIR/tests/test_code_model.sh"
source "$SCRIPT_DIR/tests/test_build_model.sh"
source "$SCRIPT_DIR/tests/test_delivery_model.sh"
source "$SCRIPT_DIR/tests/test_deployment_model.sh"
source "$SCRIPT_DIR/tests/test_generate_validate.sh"

# ————— 汇总 —————
summary

# ————— 清理 —————
rm -f "$WORKSPACE"

# 返回退出码
if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
