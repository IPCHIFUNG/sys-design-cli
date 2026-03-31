#!/bin/bash
# test_generate_validate.sh — Generate 选项 & Validate 格式测试

# 前提：前面测试文件已创建完整的 workspace

# ————— Generate -o 输出到文件 —————

section "E. Generate — 输出到文件"

OUTPUT_FILE="/tmp/sys_design_test_output.puml"

assert_pass "generate -o 输出到文件" \
    "$CLI" generate -m "$WORKSPACE" -o "$OUTPUT_FILE" context-model-diagram

TOTAL=$((TOTAL + 1))
if [ -f "$OUTPUT_FILE" ] && grep -q "@startuml" "$OUTPUT_FILE"; then
    PASS=$((PASS + 1))
    echo -e "  ${GREEN}PASS${NC} 输出文件存在且包含 @startuml"
else
    FAIL=$((FAIL + 1))
    echo -e "  ${RED}FAIL${NC} 输出文件不存在或内容不正确"
fi

rm -f "$OUTPUT_FILE"

# ————— Validate --format json —————

section "E. Validate — JSON 输出格式"

assert_contains "validate --format json 包含 JSON 格式" "is_valid" \
    "$CLI" validate -m "$WORKSPACE" --format json

# ————— Validate 各 type —————

section "E. Validate — 各类型"

assert_pass "validate -t concept-model 通过" \
    "$CLI" validate -m "$WORKSPACE" -t concept-model

# logic-view 因 H001 会报错，验证错误信息包含 H001
assert_contains "validate -t logic-view 报告 H001" "H001" \
    "$CLI" validate -m "$WORKSPACE" -t logic-view
