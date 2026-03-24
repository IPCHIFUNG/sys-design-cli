#!/bin/bash
# 测试脚本
# 用法: ./test.sh [--unit|--cli|--all]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../code"

MODE="${1:---all}"

echo "=== sys-design 测试 ==="
echo ""

run_unit_tests() {
    echo "--- 运行 Rust 单元测试 ---"
    cargo test
    echo ""
}

run_cli_tests() {
    echo "--- 运行 CLI 集成测试 ---"

    local BINARY="$(pwd)/target/release/sys-design.exe"
    if [ ! -f "$BINARY" ]; then
        echo "Release binary not found, building..."
        cargo build --release
    fi

    # 创建临时目录
    local TEST_DIR=$(mktemp -d)

    local PASS=0
    local FAIL=0
    local YAML_FILE="$TEST_DIR/test-project.yaml"

    # 测试函数
    do_test() {
        local name="$1"
        local cmd="$2"
        local pattern="$3"
        echo -n "  $name ... "
        if eval "$cmd" 2>&1 | grep -q "$pattern"; then
            echo -e "\033[32mPASS\033[0m"
            ((PASS++))
        else
            echo -e "\033[31mFAIL\033[0m"
            ((FAIL++))
        fi
    }

    # 基础命令测试
    echo "Basic Commands:"
    do_test "help" "$BINARY --help" "System architecture"
    do_test "context-model help" "$BINARY context-model --help" "Context model operations"

    # Add 测试 (add system 会自动创建文件)
    echo ""
    echo "Add Commands:"
    cd "$TEST_DIR"
    do_test "add system (auto-create)" "$BINARY context-model -s $YAML_FILE add system test-project --name Test" "Added"
    do_test "YAML auto-created" "ls -la $YAML_FILE" "test-project.yaml"
    do_test "add actor" "$BINARY context-model -s $YAML_FILE add actor user --name User -t external" "Added"
    do_test "add external-system" "$BINARY context-model -s $YAML_FILE add external-system db --name DB" "Added"
    do_test "add interface" "$BINARY context-model -s $YAML_FILE add interface api --name API" "Added"
    do_test "add provide-relation" "$BINARY context-model -s $YAML_FILE add provide-relation test-project api" "Added"
    do_test "add interface-usage" "$BINARY context-model -s $YAML_FILE add interface-usage user api" "Added"

    # List 测试
    echo ""
    echo "List Commands:"
    do_test "list system" "$BINARY context-model -s $YAML_FILE list system" "test-project"
    do_test "list actors" "$BINARY context-model -s $YAML_FILE list actors" "user"
    do_test "list relations" "$BINARY context-model -s $YAML_FILE list relations" "user"

    # Generate 测试
    echo ""
    echo "Generate Command:"
    do_test "generate" "$BINARY generate -s $YAML_FILE -o output.puml" "Generated"
    do_test "PlantUML content" "cat output.puml" "@startuml"

    # Validate 测试
    echo ""
    echo "Validate Command:"
    do_test "validate" "$BINARY validate -s $YAML_FILE" "Validation passed"

    # Remove 测试
    echo ""
    echo "Remove Commands:"
    do_test "remove interface-usage" "$BINARY context-model -s $YAML_FILE remove interface-usage user api" "Removed"
    do_test "remove actor" "$BINARY context-model -s $YAML_FILE remove actor user" "Removed"

    # 总结
    echo ""
    echo "=== CLI 测试结果 ==="
    echo -e "Passed: \033[32m$PASS\033[0m"
    echo -e "Failed: \033[31m$FAIL\033[0m"
    echo ""

    # 清理
    rm -rf "$TEST_DIR"

    if [ $FAIL -eq 0 ]; then
        echo -e "\033[32mAll CLI tests passed!\033[0m"
        return 0
    else
        echo -e "\033[31mSome CLI tests failed!\033[0m"
        return 1
    fi
}

case $MODE in
    --unit)
        run_unit_tests
        ;;
    --cli)
        run_cli_tests
        ;;
    --all|*)
        run_unit_tests
        echo ""
        run_cli_tests
        ;;
esac

echo ""
echo "=== 测试完成 ==="
