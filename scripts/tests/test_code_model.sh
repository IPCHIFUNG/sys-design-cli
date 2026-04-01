#!/bin/bash
# test_code_model.sh — Code Model 集成测试（F节）

# 前提：前面测试文件已创建完整的 workspace（含 logic_view）

# ————— CRUD 正向 —————

section "F. Code Model — CRUD 正向"

assert_pass "add package CORE_LIB" \
    "$CLI" code-model -m "$WORKSPACE" add package CORE_LIB -n "Core Library" -l rust --path src/core

assert_pass "add package UTIL_LIB" \
    "$CLI" code-model -m "$WORKSPACE" add package UTIL_LIB -n "Utility Library" -l rust --path src/util

assert_pass "add dependency CORE_LIB → UTIL_LIB" \
    "$CLI" code-model -m "$WORKSPACE" add dependency CORE_LIB UTIL_LIB

# ————— CRUD 负向 —————

section "F. Code Model — CRUD 负向"

assert_fail "重复添加 package CORE_LIB（报错）" \
    "$CLI" code-model -m "$WORKSPACE" add package CORE_LIB -n "dup"

assert_fail "重复添加 dependency CORE_LIB → UTIL_LIB（报错）" \
    "$CLI" code-model -m "$WORKSPACE" add dependency CORE_LIB UTIL_LIB

assert_fail "dependency 引用不存在的 from（报错）" \
    "$CLI" code-model -m "$WORKSPACE" add dependency NONEXIST UTIL_LIB

assert_fail "dependency 引用不存在的 to（报错）" \
    "$CLI" code-model -m "$WORKSPACE" add dependency CORE_LIB NONEXIST

# ————— List & Show —————

section "F. Code Model — List & Show"

assert_contains "list packages 包含 CORE_LIB" "CORE_LIB" \
    "$CLI" code-model -m "$WORKSPACE" list packages

assert_contains "list dependencies 包含 UTIL_LIB" "UTIL_LIB" \
    "$CLI" code-model -m "$WORKSPACE" list dependencies

assert_contains "show CORE_LIB 包含 Core Library" "Core Library" \
    "$CLI" code-model -m "$WORKSPACE" show CORE_LIB

# ————— Validate & Generate —————

section "F. Code Model — Validate & Generate"

assert_pass "validate -t code-model 通过" \
    "$CLI" validate -m "$WORKSPACE" -t code-model

assert_contains "generate code-model-diagram 包含 @startuml" "@startuml" \
    "$CLI" generate -m "$WORKSPACE" code-model-diagram

assert_contains "generate code-model-diagram 包含 CORE_LIB" "CORE_LIB" \
    "$CLI" generate -m "$WORKSPACE" code-model-diagram

# ————— Remove（临时元素，保留 CORE_LIB 和 UTIL_LIB 给后续 G 节） —————

section "F. Code Model — Remove"

assert_pass "添加临时 package TEMP_PKG" \
    "$CLI" code-model -m "$WORKSPACE" add package TEMP_PKG -n "Temp"

assert_pass "添加临时 dependency CORE_LIB → TEMP_PKG" \
    "$CLI" code-model -m "$WORKSPACE" add dependency CORE_LIB TEMP_PKG

assert_pass "remove dependency CORE_LIB → TEMP_PKG" \
    "$CLI" code-model -m "$WORKSPACE" remove dependency CORE_LIB TEMP_PKG

# 级联删除：删除 package 时引用该 package 的 dependency 也应被删除
assert_pass "添加 TEMP_PKG2 和依赖" \
    "$CLI" code-model -m "$WORKSPACE" add package TEMP_PKG2 -n "Temp2"

assert_pass "添加依赖 UTIL_LIB → TEMP_PKG2" \
    "$CLI" code-model -m "$WORKSPACE" add dependency UTIL_LIB TEMP_PKG2

assert_pass "remove package TEMP_PKG2（级联删除引用依赖）" \
    "$CLI" code-model -m "$WORKSPACE" remove package TEMP_PKG2

# 验证级联：UTIL_LIB → TEMP_PKG2 的 dependency 应已消失
TOTAL=$((TOTAL + 1))
DEPS=$("$CLI" code-model -m "$WORKSPACE" list dependencies 2>&1)
if echo "$DEPS" | grep -q "TEMP_PKG2"; then
    FAIL=$((FAIL + 1))
    echo -e "  ${RED}FAIL${NC} 删除 package 后引用依赖未被级联删除"
else
    PASS=$((PASS + 1))
    echo -e "  ${GREEN}PASS${NC} 删除 package 后引用依赖被级联删除"
fi

assert_pass "remove package TEMP_PKG" \
    "$CLI" code-model -m "$WORKSPACE" remove package TEMP_PKG

assert_fail "remove 不存在的 package（报错）" \
    "$CLI" code-model -m "$WORKSPACE" remove package NONEXIST
