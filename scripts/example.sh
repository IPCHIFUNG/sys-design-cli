#!/bin/bash
# 示例脚本：展示如何使用 sys-design.exe 生成4+1视图
# 本脚本禁止任何AI Agent自动修改

set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# 配置
CLI="run.sh"
WORKSPACE_FILE="test_workspace.yaml"
OUTPUT_DIR="test_output"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

rm -f "$WORKSPACE_FILE"

echo ""
echo -e "${YELLOW}1. 创建系统${NC}"
./$CLI context-model -s "$WORKSPACE_FILE" add system SNP -n "SNP System" -d "SNP 核心系统"
# 重复调用时应该报错
echo -e "${CYAN}  测试重复创建系统（预期报错）${NC}"
if ./$CLI context-model -s "$WORKSPACE_FILE" add system SNP -n "SNP System" -d "SNP 核心系统" 2>&1; then
    echo -e "${RED}  错误：重复创建系统应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复创建系统报错${NC}"
fi


echo ""
echo -e "${YELLOW}2. 添加外部系统${NC}"
./$CLI context-model -s "$WORKSPACE_FILE" add external-system BD_SOFT -n "BD Soft" -t "REST API"
./$CLI context-model -s "$WORKSPACE_FILE" add external-system CHIP -n "Chip" -t "Hardware"
./$CLI context-model -s "$WORKSPACE_FILE" add external-system CHIP_CTRL -n "Chip Controller" -t "Hardware"
# 重复调用时应该报错
echo -e "${CYAN}  测试重复添加外部系统（预期报错）${NC}"
if ./$CLI context-model -s "$WORKSPACE_FILE" add external-system BD_SOFT -n "BD Soft" -t "REST API" 2>&1; then
    echo -e "${RED}  错误：重复添加外部系统应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复添加外部系统报错${NC}"
fi

echo ""
echo -e "${YELLOW}3. 添加接口${NC}"
./$CLI context-model -s "$WORKSPACE_FILE" add interface ITF_SNP_CFG -n "SNP Config Interface" -p rest
./$CLI context-model -s "$WORKSPACE_FILE" add interface ITF_DATA_PLANE -n "Data Plane Interface" -p grpc
./$CLI context-model -s "$WORKSPACE_FILE" add interface ITF_CHIP_CFG -n "Chip Config Interface" -p rest
# 重复调用时应该报错
echo -e "${CYAN}  测试重复添加接口（预期报错）${NC}"
if ./$CLI context-model -s "$WORKSPACE_FILE" add interface ITF_SNP_CFG -n "SNP Config Interface" -p rest 2>&1; then
    echo -e "${RED}  错误：重复添加接口应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复添加接口报错${NC}"
fi

echo ""
echo -e "${YELLOW}4. 添加提供关系（系统提供接口）${NC}"
./$CLI context-model -s "$WORKSPACE_FILE" add provide-relation SNP ITF_SNP_CFG
./$CLI context-model -s "$WORKSPACE_FILE" add provide-relation SNP ITF_DATA_PLANE
./$CLI context-model -s "$WORKSPACE_FILE" add provide-relation CHIP_CTRL ITF_CHIP_CFG
# 重复调用时应该报错
echo -e "${CYAN}  测试重复添加提供关系（预期报错）${NC}"
if ./$CLI context-model -s "$WORKSPACE_FILE" add provide-relation SNP ITF_SNP_CFG 2>&1; then
    echo -e "${RED}  错误：重复添加提供关系应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复添加提供关系报错${NC}"
fi

echo ""
echo -e "${YELLOW}5. 添加使用关系${NC}"
./$CLI context-model -s "$WORKSPACE_FILE" add interface-usage BD_SOFT ITF_SNP_CFG
./$CLI context-model -s "$WORKSPACE_FILE" add interface-usage CHIP ITF_DATA_PLANE
./$CLI context-model -s "$WORKSPACE_FILE" add interface-usage SNP ITF_CHIP_CFG
# 重复调用时应该报错
echo -e "${CYAN}  测试重复添加使用关系（预期报错）${NC}"
if ./$CLI context-model -s "$WORKSPACE_FILE" add interface-usage BD_SOFT ITF_SNP_CFG 2>&1; then
    echo -e "${RED}  错误：重复添加使用关系应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复添加使用关系报错${NC}"
fi

./$CLI validate -s "$WORKSPACE_FILE"


echo ""
echo -e "${YELLOW}6. 添加逻辑架构概念模型里不存在的元素${NC}"
if ./$CLI logic-model -s "$WORKSPACE_FILE" add subsystem MOTOR -n "Motor Module" -d "电机控制模块" 2>&1; then
    echo -e "${RED}  错误：逻辑架构概念模型里没有 subsystem 元素，因此应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复添加使用关系报错${NC}"
fi
if ./$CLI logic-model -s "$WORKSPACE_FILE" add component MOTOR -n "Motor Module" -d "电机控制模块" 2>&1; then
    echo -e "${RED}  错误：逻辑架构概念模型里没有 component 元素，因此应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复添加使用关系报错${NC}"
fi
if ./$CLI logic-model -s "$WORKSPACE_FILE" add module MOTOR -n "Motor Module" -d "电机控制模块" 2>&1; then
    echo -e "${RED}  错误：逻辑架构概念模型里没有 module 元素，因此应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复添加使用关系报错${NC}"
fi
if ./$CLI logic-model -s "$WORKSPACE_FILE" add submodule MOTOR -n "Motor Module" -d "电机控制模块" 2>&1; then
    echo -e "${RED}  错误：逻辑架构概念模型里没有 submodule 元素，因此应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复添加使用关系报错${NC}"
fi

echo ""
echo -e "${YELLOW}7. 添加逻辑架构概念模型元素${NC}"
./$CLI concept-model -s "$WORKSPACE_FILE" add element subsystem
./$CLI concept-model -s "$WORKSPACE_FILE" add element component
./$CLI concept-model -s "$WORKSPACE_FILE" add element module
./$CLI concept-model -s "$WORKSPACE_FILE" add element submodule
# 重复添加应该报错
echo -e "${CYAN}  测试重复添加概念模型元素（预期报错）${NC}"
if ./$CLI concept-model -s "$WORKSPACE_FILE" add element subsystem 2>&1; then
    echo -e "${RED}  错误：重复添加概念模型元素应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：重复添加概念模型元素报错${NC}"
fi

echo ""
echo -e "${YELLOW}8. 存在游离逻辑架构概念模型元素时应该报错${NC}"
if ./$CLI validate -s "$WORKSPACE_FILE" 2>&1; then
    echo -e "${RED}  错误：存在游离逻辑架构概念模型元素时应该报错但没有报错${NC}"
    exit 1
else
    echo -e "${GREEN}  符合预期：存在游离逻辑架构概念模型元素时报错${NC}"
fi
