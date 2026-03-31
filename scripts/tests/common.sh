#!/bin/bash
# common.sh — 集成测试辅助函数

# ————— 计数器 —————
PASS=0
FAIL=0
TOTAL=0

# ————— 颜色 —————
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# 打印分段标题
section() {
    echo ""
    echo -e "${YELLOW}══ $1 ══${NC}"
}

# 期望命令成功（exit code 0）
assert_pass() {
    local desc="$1"; shift
    TOTAL=$((TOTAL + 1))
    if "$@" >/dev/null 2>&1; then
        PASS=$((PASS + 1))
        echo -e "  ${GREEN}PASS${NC} $desc"
    else
        FAIL=$((FAIL + 1))
        echo -e "  ${RED}FAIL${NC} $desc"
    fi
}

# 期望命令失败（exit code != 0）
assert_fail() {
    local desc="$1"; shift
    TOTAL=$((TOTAL + 1))
    if "$@" >/dev/null 2>&1; then
        FAIL=$((FAIL + 1))
        echo -e "  ${RED}FAIL${NC} $desc (expected failure but succeeded)"
    else
        PASS=$((PASS + 1))
        echo -e "  ${GREEN}PASS${NC} $desc"
    fi
}

# 输出精确匹配
assert_output_equals() {
    local desc="$1" expected="$2"; shift 2
    TOTAL=$((TOTAL + 1))
    local actual
    actual=$("$@" 2>&1) || true
    if [ "$actual" = "$expected" ]; then
        PASS=$((PASS + 1))
        echo -e "  ${GREEN}PASS${NC} $desc"
    else
        FAIL=$((FAIL + 1))
        echo -e "  ${RED}FAIL${NC} $desc (output mismatch)"
        echo -e "  ${CYAN}--- Expected (first 10 lines) ---${NC}"
        echo "$expected" | head -10
        echo -e "  ${CYAN}--- Actual (first 10 lines) ---${NC}"
        echo "$actual" | head -10
        echo -e "  ${CYAN}---${NC}"
    fi
}

# 输出包含指定字符串
assert_contains() {
    local desc="$1" pattern="$2"; shift 2
    TOTAL=$((TOTAL + 1))
    local actual
    actual=$("$@" 2>&1) || true
    if echo "$actual" | grep -qF "$pattern"; then
        PASS=$((PASS + 1))
        echo -e "  ${GREEN}PASS${NC} $desc"
    else
        FAIL=$((FAIL + 1))
        echo -e "  ${RED}FAIL${NC} $desc"
        echo -e "  ${CYAN}Expected to contain: $pattern${NC}"
        echo -e "  ${CYAN}Actual:${NC}"
        echo "$actual" | head -5
    fi
}

# 打印汇总结果
summary() {
    echo ""
    echo -e "${YELLOW}═══════════════════════════════════════${NC}"
    echo -e "  Total:  $TOTAL"
    echo -e "  ${GREEN}Passed: $PASS${NC}"
    echo -e "  ${RED}Failed: $FAIL${NC}"
    echo -e "${YELLOW}═══════════════════════════════════════${NC}"

    if [ "$FAIL" -eq 0 ]; then
        echo -e "\n${GREEN}All $TOTAL tests passed!${NC}\n"
        return 0
    else
        echo -e "\n${RED}$FAIL test(s) failed!${NC}\n"
        return 1
    fi
}
