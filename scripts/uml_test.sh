#!/bin/bash

set -e

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

rm -f test.yaml

./scripts/run.sh context-model -m test.yaml add system SNP -n "SNP System" -d "SNP 核心系统"

./scripts/run.sh context-model -m test.yaml add actor USER -n "User" -t "internal"
./scripts/run.sh context-model -m test.yaml add external-system BD_SOFT -n "BD Soft" -t "REST API"
./scripts/run.sh context-model -m test.yaml add external-system CHIP -n "Chip" -t "Hardware"
./scripts/run.sh context-model -m test.yaml add external-system CHIP_CTRL -n "Chip Controller" -t "Hardware"

./scripts/run.sh context-model -m test.yaml add interface ITF_OMU -n "SNP OMU" -p rest
./scripts/run.sh context-model -m test.yaml add interface ITF_SNP_CFG -n "SNP Config Interface" -p rest
./scripts/run.sh context-model -m test.yaml add interface ITF_DATA_PLANE -n "Data Plane Interface" -p grpc
./scripts/run.sh context-model -m test.yaml add interface ITF_CHIP_CFG -n "Chip Config Interface" -p rest

./scripts/run.sh context-model -m test.yaml add provide-relation SNP ITF_OMU
./scripts/run.sh context-model -m test.yaml add provide-relation SNP ITF_SNP_CFG
./scripts/run.sh context-model -m test.yaml add provide-relation SNP ITF_DATA_PLANE
./scripts/run.sh context-model -m test.yaml add provide-relation CHIP_CTRL ITF_CHIP_CFG

./scripts/run.sh context-model -m test.yaml add interface-usage USER ITF_OMU
./scripts/run.sh context-model -m test.yaml add interface-usage BD_SOFT ITF_SNP_CFG
./scripts/run.sh context-model -m test.yaml add interface-usage CHIP ITF_DATA_PLANE
./scripts/run.sh context-model -m test.yaml add interface-usage SNP ITF_CHIP_CFG

./scripts/run.sh validate -m test.yaml

./scripts/run.sh concept-model -m test.yaml add element subsystem
./scripts/run.sh concept-model -m test.yaml add element component
./scripts/run.sh concept-model -m test.yaml add element module
./scripts/run.sh concept-model -m test.yaml add element submodule
./scripts/run.sh concept-model -m test.yaml add containment system subsystem
./scripts/run.sh concept-model -m test.yaml add containment subsystem component
./scripts/run.sh concept-model -m test.yaml add containment system component
./scripts/run.sh concept-model -m test.yaml add containment component module
./scripts/run.sh concept-model -m test.yaml add containment module submodule
./scripts/run.sh concept-model -m test.yaml add containment submodule submodule

./scripts/run.sh validate -m test.yaml

./scripts/run.sh logic-model -m test.yaml add subsystem CTRL_SUBSYSTEM -n "Controller Subsystem" -d "控制子系统"
./scripts/run.sh logic-model -m test.yaml add component CTRL -n "Controller" -d "控制组件"
./scripts/run.sh logic-model -m test.yaml add module MOTOR_CTRL -n "Motor Controller" -d "电机控制逻辑"
./scripts/run.sh logic-model -m test.yaml add submodule POSITION_LOOP -n "Position Loop" -d "位置环逻辑"
./scripts/run.sh logic-model -m test.yaml add submodule SPEED_LOOP -n "Speed Loop" -d "速度环逻辑"
./scripts/run.sh logic-model -m test.yaml add submodule CURRENT_LOOP -n "Current Loop" -d "电流环逻辑"

./scripts/run.sh logic-model -m test.yaml add containment SNP CTRL_SUBSYSTEM
./scripts/run.sh logic-model -m test.yaml add containment CTRL_SUBSYSTEM CTRL
./scripts/run.sh logic-model -m test.yaml add containment CTRL MOTOR_CTRL
./scripts/run.sh logic-model -m test.yaml add containment MOTOR_CTRL CURRENT_LOOP
./scripts/run.sh logic-model -m test.yaml add containment MOTOR_CTRL SPEED_LOOP
./scripts/run.sh logic-model -m test.yaml add containment MOTOR_CTRL POSITION_LOOP

./scripts/run.sh logic-model -m test.yaml add interface ITF_CTRL_SUBSYSTEM -n "Controller Subsystem Interface" -d "控制子系统接口"
./scripts/run.sh logic-model -m test.yaml add interface ITF_CTRL -n "Controller Interface" -d "控制组件接口"
./scripts/run.sh logic-model -m test.yaml add interface ITF_MOTOR_CTRL -n "Motor Controller Interface" -d "电机控制逻辑接口"
./scripts/run.sh logic-model -m test.yaml add interface ITF_POSITION_LOOP -n "Position Loop Interface" -d "位置环逻辑接口"
./scripts/run.sh logic-model -m test.yaml add interface ITF_SPEED_LOOP -n "Speed Loop Interface" -d "速度环逻辑接口"
./scripts/run.sh logic-model -m test.yaml add interface ITF_CURRENT_LOOP -n "Current Loop Interface" -d "电流环逻辑接口"

./scripts/run.sh logic-model -m test.yaml add provide-relation CTRL_SUBSYSTEM ITF_CTRL_SUBSYSTEM
./scripts/run.sh logic-model -m test.yaml add provide-relation CTRL ITF_CTRL
./scripts/run.sh logic-model -m test.yaml add provide-relation MOTOR_CTRL ITF_MOTOR_CTRL
./scripts/run.sh logic-model -m test.yaml add provide-relation POSITION_LOOP ITF_POSITION_LOOP
./scripts/run.sh logic-model -m test.yaml add provide-relation SPEED_LOOP ITF_SPEED_LOOP
./scripts/run.sh logic-model -m test.yaml add provide-relation CURRENT_LOOP ITF_CURRENT_LOOP

./scripts/run.sh logic-model -m test.yaml add dependency POSITION_LOOP ITF_SPEED_LOOP
./scripts/run.sh logic-model -m test.yaml add dependency SPEED_LOOP ITF_CURRENT_LOOP

./scripts/run.sh logic-model -m test.yaml add containment ITF_MOTOR_CTRL ITF_POSITION_LOOP
./scripts/run.sh logic-model -m test.yaml add containment ITF_CTRL ITF_MOTOR_CTRL
./scripts/run.sh logic-model -m test.yaml add containment ITF_CTRL_SUBSYSTEM ITF_CTRL
./scripts/run.sh logic-model -m test.yaml add containment ITF_OMU ITF_CTRL_SUBSYSTEM

./scripts/run.sh validate -m test.yaml

echo ""
echo -e "${YELLOW}15. 测试 context-model-diagram 生成输出${NC}"
ACTUAL_OUTPUT=$(code/target/release/sys-design generate -m test.yaml context-model-diagram 2>&1)

EXPECTED_OUTPUT='@startuml

skinparam defaultTextAlignment center

rectangle "<<EXTERNAL_SYSTEM>>\nBD Soft" as BD_SOFT
rectangle "<<EXTERNAL_SYSTEM>>\nChip" as CHIP
rectangle "<<EXTERNAL_SYSTEM>>\nChip Controller" as CHIP_CTRL

actor "<<INTERNAL_ACTOR>>\nUser" as USER

interface ITF_OMU
interface ITF_SNP_CFG
interface ITF_DATA_PLANE
interface ITF_CHIP_CFG

rectangle "<<SYSTEM>>\nSNP System" as SNP

USER ..> ITF_OMU
BD_SOFT ..> ITF_SNP_CFG
CHIP ..> ITF_DATA_PLANE
SNP ..> ITF_CHIP_CFG

ITF_OMU --- SNP
ITF_SNP_CFG --- SNP
ITF_DATA_PLANE --- SNP
ITF_CHIP_CFG --- CHIP_CTRL

@enduml'

if [ "$ACTUAL_OUTPUT" = "$EXPECTED_OUTPUT" ]; then
    echo -e "${GREEN}  符合预期：context-model-diagram 输出正确${NC}"
else
    echo -e "${RED}  错误：context-model-diagram 输出不匹配${NC}"
    echo -e "${CYAN}  预期输出:${NC}"
    echo "$EXPECTED_OUTPUT"
    echo ""
    echo -e "${CYAN}  实际输出:${NC}"
    echo "$ACTUAL_OUTPUT"
    exit 1
fi

echo ""
echo -e "${YELLOW}16. 测试 concept-model-diagram 生成输出${NC}"
ACTUAL_OUTPUT=$(code/target/release/sys-design generate -m test.yaml concept-model-diagram 2>&1)

EXPECTED_OUTPUT='@startuml

skinparam defaultTextAlignment center

rectangle SYSTEM
rectangle SUBSYSTEM
rectangle COMPONENT
rectangle MODULE
rectangle SUBMODULE

SYSTEM o.. SUBSYSTEM
SYSTEM o.. COMPONENT
SUBSYSTEM o.. COMPONENT
COMPONENT o.. MODULE
MODULE o.. SUBMODULE

SUBMODULE o.. SUBMODULE

@enduml'

if [ "$ACTUAL_OUTPUT" = "$EXPECTED_OUTPUT" ]; then
    echo -e "${GREEN}  符合预期：concept-model-diagram 输出正确${NC}"
else
    echo -e "${RED}  错误：concept-model-diagram 输出不匹配${NC}"
    echo -e "${CYAN}  预期输出:${NC}"
    echo "$EXPECTED_OUTPUT"
    echo ""
    echo -e "${CYAN}  实际输出:${NC}"
    echo "$ACTUAL_OUTPUT"
    exit 1
fi

echo ""
echo -e "${YELLOW}17. 测试 logic-model-diagram 生成输出（精确匹配，不存在的元素报错）${NC}"
ACTUAL_OUTPUT=$(code/target/release/sys-design generate -m test.yaml logic-model-diagram MOTOR 2>&1)

EXPECTED_OUTPUT='@startuml

top to bottom direction
skinparam defaultTextAlignment center

'\'' ERROR: Element '\''MOTOR'\'' not found in diagram
'\'' Available elements: MOTOR_CTRL, POSITION_LOOP, SPEED_LOOP, CURRENT_LOOP, CTRL, CTRL_SUBSYSTEM, ITF_CTRL_SUBSYSTEM, ITF_CTRL, ITF_MOTOR_CTRL, ITF_POSITION_LOOP, ITF_SPEED_LOOP, ITF_CURRENT_LOOP

@enduml'

if [ "$ACTUAL_OUTPUT" = "$EXPECTED_OUTPUT" ]; then
    echo -e "${GREEN}  符合预期：精确匹配，MOTOR 元素不存在时正确报错${NC}"
else
    echo -e "${RED}  错误：logic-model-diagram 输出不匹配${NC}"
    echo -e "${CYAN}  预期输出:${NC}"
    echo "$EXPECTED_OUTPUT"
    echo ""
    echo -e "${CYAN}  实际输出:${NC}"
    echo "$ACTUAL_OUTPUT"
    exit 1
fi

echo ""
echo -e "${YELLOW}18. 测试 logic-model-diagram 生成输出（MOTOR_CTRL 精确匹配）${NC}"
ACTUAL_OUTPUT=$(code/target/release/sys-design generate -m test.yaml logic-model-diagram MOTOR_CTRL 2>&1)

EXPECTED_OUTPUT='@startuml

top to bottom direction
skinparam defaultTextAlignment center

interface ITF_MOTOR_CTRL

rectangle "<<MODULE>>\nMOTOR_CTRL" as MOTOR_CTRL {

    interface ITF_CURRENT_LOOP
    interface ITF_SPEED_LOOP
    interface ITF_POSITION_LOOP

    rectangle "<<SUBMODULE>>\nCURRENT_LOOP" as CURRENT_LOOP
    rectangle "<<SUBMODULE>>\nSPEED_LOOP" as SPEED_LOOP
    rectangle "<<SUBMODULE>>\nPOSITION_LOOP" as POSITION_LOOP
}

ITF_MOTOR_CTRL *.. ITF_POSITION_LOOP

ITF_MOTOR_CTRL --- MOTOR_CTRL
ITF_POSITION_LOOP --- POSITION_LOOP
ITF_SPEED_LOOP --- SPEED_LOOP
ITF_CURRENT_LOOP --- CURRENT_LOOP
SPEED_LOOP ..> ITF_CURRENT_LOOP
POSITION_LOOP ..> ITF_SPEED_LOOP

@enduml'

if [ "$ACTUAL_OUTPUT" = "$EXPECTED_OUTPUT" ]; then
    echo -e "${GREEN}  符合预期：MOTOR_CTRL 精确匹配成功，输出完整逻辑视图${NC}"
else
    echo -e "${RED}  错误：logic-model-diagram 输出不匹配${NC}"
    echo -e "${CYAN}  预期输出:${NC}"
    echo "$EXPECTED_OUTPUT"
    echo ""
    echo -e "${CYAN}  实际输出:${NC}"
    echo "$ACTUAL_OUTPUT"
    exit 1
fi

echo ""
echo -e "${YELLOW}19. 测试 logic-model-diagram 生成输出（SNP 系统元素）${NC}"
ACTUAL_OUTPUT=$(code/target/release/sys-design generate -m test.yaml logic-model-diagram SNP 2>&1)

EXPECTED_OUTPUT='@startuml

top to bottom direction
skinparam defaultTextAlignment center

interface ITF_OMU
interface ITF_SNP_CFG
interface ITF_DATA_PLANE

rectangle "<<SYSTEM>>\nSNP" as SNP {

    interface ITF_CTRL_SUBSYSTEM

    rectangle "<<SUBSYSTEM>>\nCTRL_SUBSYSTEM" as CTRL_SUBSYSTEM
}

ITF_OMU *.. ITF_CTRL_SUBSYSTEM

ITF_OMU --- SNP
ITF_SNP_CFG --- SNP
ITF_DATA_PLANE --- SNP
ITF_CTRL_SUBSYSTEM --- CTRL_SUBSYSTEM

@enduml'

if [ "$ACTUAL_OUTPUT" = "$EXPECTED_OUTPUT" ]; then
    echo -e "${GREEN}  符合预期：SNP 系统元素输出正确，只展开一层，从上下文图获取接口${NC}"
else
    echo -e "${RED}  错误：logic-model-diagram SNP 输出不匹配${NC}"
    echo -e "${CYAN}  预期输出:${NC}"
    echo "$EXPECTED_OUTPUT"
    echo ""
    echo -e "${CYAN}  实际输出:${NC}"
    echo "$ACTUAL_OUTPUT"
    exit 1
fi

echo ""
echo -e "${GREEN}所有测试通过！${NC}"



rm -f local_diagram.md
echo "\`\`\`plantuml" > local_diagram.md
code/target/release/sys-design generate --model_file test.yaml logic-model-diagram CTRL_SUBSYSTEM >> local_diagram.md
echo "\`\`\`" >> local_diagram.md
