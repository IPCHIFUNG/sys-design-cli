#!/bin/bash
# test_build_model.sh — Build Model 集成测试（G节）

# 前提：F 节已保留 CORE_LIB 和 UTIL_LIB

# ————— CRUD 正向 —————

section "G. Build Model — CRUD 正向"

assert_pass "add artifact CORE_BIN" \
    "$CLI" build-model -m "$WORKSPACE" add artifact CORE_BIN -n "Core Binary" \
        --build-tool cargo --output-type binary --source-packages CORE_LIB

assert_pass "add artifact UTIL_LIB_A" \
    "$CLI" build-model -m "$WORKSPACE" add artifact UTIL_LIB_A -n "Util Library Artifact" \
        --build-tool cargo --output-type library --source-packages UTIL_LIB

assert_pass "add dependency CORE_BIN → UTIL_LIB_A" \
    "$CLI" build-model -m "$WORKSPACE" add dependency CORE_BIN UTIL_LIB_A

# ————— CRUD 负向 —————

section "G. Build Model — CRUD 负向"

assert_fail "重复添加 artifact CORE_BIN（报错）" \
    "$CLI" build-model -m "$WORKSPACE" add artifact CORE_BIN -n "dup"

assert_fail "重复添加 dependency CORE_BIN → UTIL_LIB_A（报错）" \
    "$CLI" build-model -m "$WORKSPACE" add dependency CORE_BIN UTIL_LIB_A

assert_fail "dependency 引用不存在的 from（报错）" \
    "$CLI" build-model -m "$WORKSPACE" add dependency NONEXIST UTIL_LIB_A

assert_fail "dependency 引用不存在的 to（报错）" \
    "$CLI" build-model -m "$WORKSPACE" add dependency CORE_BIN NONEXIST

# ————— List & Show —————

section "G. Build Model — List & Show"

assert_contains "list artifacts 包含 CORE_BIN" "CORE_BIN" \
    "$CLI" build-model -m "$WORKSPACE" list artifacts

assert_contains "list dependencies 包含 UTIL_LIB_A" "UTIL_LIB_A" \
    "$CLI" build-model -m "$WORKSPACE" list dependencies

assert_contains "show CORE_BIN 包含 Core Binary" "Core Binary" \
    "$CLI" build-model -m "$WORKSPACE" show CORE_BIN

# ————— Validate & Generate —————

section "G. Build Model — Validate & Generate"

assert_pass "validate -t build-model 通过" \
    "$CLI" validate -m "$WORKSPACE" -t build-model

assert_contains "generate build-model-diagram 包含 @startuml" "@startuml" \
    "$CLI" generate -m "$WORKSPACE" build-model-diagram

assert_contains "generate build-model-diagram 包含 CORE_BIN" "CORE_BIN" \
    "$CLI" generate -m "$WORKSPACE" build-model-diagram

# ————— Remove（临时元素，保留 CORE_BIN 和 UTIL_LIB_A 给后续 H 节） —————

section "G. Build Model — Remove"

assert_pass "添加临时 artifact TEMP_ART" \
    "$CLI" build-model -m "$WORKSPACE" add artifact TEMP_ART -n "Temp Artifact"

assert_pass "添加临时 dependency CORE_BIN → TEMP_ART" \
    "$CLI" build-model -m "$WORKSPACE" add dependency CORE_BIN TEMP_ART

assert_pass "remove dependency CORE_BIN → TEMP_ART" \
    "$CLI" build-model -m "$WORKSPACE" remove dependency CORE_BIN TEMP_ART

# 级联删除：删除 artifact 时引用该 artifact 的 dependency 也应被删除
assert_pass "添加 TEMP_ART2 和依赖" \
    "$CLI" build-model -m "$WORKSPACE" add artifact TEMP_ART2 -n "Temp2"

assert_pass "添加依赖 UTIL_LIB_A → TEMP_ART2" \
    "$CLI" build-model -m "$WORKSPACE" add dependency UTIL_LIB_A TEMP_ART2

assert_pass "remove artifact TEMP_ART2（级联删除引用依赖）" \
    "$CLI" build-model -m "$WORKSPACE" remove artifact TEMP_ART2

# 验证级联：UTIL_LIB_A → TEMP_ART2 的 dependency 应已消失
TOTAL=$((TOTAL + 1))
DEPS=$("$CLI" build-model -m "$WORKSPACE" list dependencies 2>&1)
if echo "$DEPS" | grep -q "TEMP_ART2"; then
    FAIL=$((FAIL + 1))
    echo -e "  ${RED}FAIL${NC} 删除 artifact 后引用依赖未被级联删除"
else
    PASS=$((PASS + 1))
    echo -e "  ${GREEN}PASS${NC} 删除 artifact 后引用依赖被级联删除"
fi

assert_pass "remove artifact TEMP_ART" \
    "$CLI" build-model -m "$WORKSPACE" remove artifact TEMP_ART

assert_fail "remove 不存在的 artifact（报错）" \
    "$CLI" build-model -m "$WORKSPACE" remove artifact NONEXIST
