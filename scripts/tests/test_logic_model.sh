#!/bin/bash
# test_logic_model.sh — Logic Model 集成测试

# 前提：test_context_model.sh 和 test_concept_model.sh 已完成

# ————— 添加逻辑元素 —————

section "C. Logic Model — 添加元素"

assert_pass "添加 subsystem CTRL_SUBSYSTEM" \
    "$CLI" logic-model -m "$WORKSPACE" add subsystem CTRL_SUBSYSTEM -n "Controller Subsystem" -d "控制子系统"

assert_pass "添加 component CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add component CTRL -n "Controller" -d "控制组件"

assert_pass "添加 module MOTOR_CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add module MOTOR_CTRL -n "Motor Controller" -d "电机控制逻辑"

assert_pass "添加 submodule POSITION_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add submodule POSITION_LOOP -n "Position Loop" -d "位置环逻辑"

assert_pass "添加 submodule SPEED_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add submodule SPEED_LOOP -n "Speed Loop" -d "速度环逻辑"

assert_pass "添加 submodule CURRENT_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add submodule CURRENT_LOOP -n "Current Loop" -d "电流环逻辑"

# ————— 添加元素（负向） —————

section "C. Logic Model — 添加元素（负向）"

assert_fail "重复添加 subsystem（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add subsystem CTRL_SUBSYSTEM -n "dup" -d "dup"

assert_fail "不存在的类型 abc（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add abc AAA -n "AAA" -d "AAA模块"

assert_fail "add component --subsystem（报错 not yet supported）" \
    "$CLI" logic-model -m "$WORKSPACE" add component DUP_CTRL -n "Dup" -d "Dup" --subsystem CTRL_SUBSYSTEM

# ————— 游离元素验证 —————

assert_fail "存在游离逻辑元素时 validate 报错" \
    "$CLI" validate -m "$WORKSPACE"

# ————— 添加包含关系 —————

section "C. Logic Model — 添加包含关系"

assert_pass "containment SNP → CTRL_SUBSYSTEM" \
    "$CLI" logic-model -m "$WORKSPACE" add containment SNP CTRL_SUBSYSTEM

assert_pass "containment CTRL_SUBSYSTEM → CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add containment CTRL_SUBSYSTEM CTRL

assert_pass "containment CTRL → MOTOR_CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add containment CTRL MOTOR_CTRL

assert_pass "containment MOTOR_CTRL → CURRENT_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add containment MOTOR_CTRL CURRENT_LOOP

assert_pass "containment MOTOR_CTRL → SPEED_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add containment MOTOR_CTRL SPEED_LOOP

assert_pass "containment MOTOR_CTRL → POSITION_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add containment MOTOR_CTRL POSITION_LOOP

# 包含关系（负向）

assert_fail "重复 containment SNP → CTRL_SUBSYSTEM（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add containment SNP CTRL_SUBSYSTEM

assert_fail "containment 引用不存在的 parent（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add containment AAA CTRL_SUBSYSTEM

assert_fail "containment 引用不存在的 child（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add containment SNP AAA

assert_fail "containment 违反层级（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add containment CTRL_SUBSYSTEM CURRENT_LOOP

# ————— 添加接口 —————

section "C. Logic Model — 添加接口"

assert_pass "添加接口 ITF_CTRL_SUBSYSTEM" \
    "$CLI" logic-model -m "$WORKSPACE" add interface ITF_CTRL_SUBSYSTEM -n "Controller Subsystem Interface" -d "控制子系统接口"

assert_pass "添加接口 ITF_CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add interface ITF_CTRL -n "Controller Interface" -d "控制组件接口"

assert_pass "添加接口 ITF_MOTOR_CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add interface ITF_MOTOR_CTRL -n "Motor Controller Interface" -d "电机控制逻辑接口"

assert_pass "添加接口 ITF_POSITION_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add interface ITF_POSITION_LOOP -n "Position Loop Interface" -d "位置环逻辑接口"

assert_pass "添加接口 ITF_SPEED_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add interface ITF_SPEED_LOOP -n "Speed Loop Interface" -d "速度环逻辑接口"

assert_pass "添加接口 ITF_CURRENT_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add interface ITF_CURRENT_LOOP -n "Current Loop Interface" -d "电流环逻辑接口"

assert_fail "重复添加接口 ITF_CTRL（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add interface ITF_CTRL -n "dup" -d "dup"

# ————— 添加提供关系 —————

section "C. Logic Model — 添加提供关系"

assert_pass "provide-relation CTRL_SUBSYSTEM → ITF_CTRL_SUBSYSTEM" \
    "$CLI" logic-model -m "$WORKSPACE" add provide-relation CTRL_SUBSYSTEM ITF_CTRL_SUBSYSTEM

assert_pass "provide-relation CTRL → ITF_CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add provide-relation CTRL ITF_CTRL

assert_pass "provide-relation MOTOR_CTRL → ITF_MOTOR_CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add provide-relation MOTOR_CTRL ITF_MOTOR_CTRL

assert_pass "provide-relation POSITION_LOOP → ITF_POSITION_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add provide-relation POSITION_LOOP ITF_POSITION_LOOP

assert_pass "provide-relation SPEED_LOOP → ITF_SPEED_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add provide-relation SPEED_LOOP ITF_SPEED_LOOP

assert_pass "provide-relation CURRENT_LOOP → ITF_CURRENT_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add provide-relation CURRENT_LOOP ITF_CURRENT_LOOP

assert_fail "重复 provide-relation CTRL → ITF_CTRL（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add provide-relation CTRL ITF_CTRL

# ————— 添加依赖 —————

section "C. Logic Model — 添加依赖"

assert_pass "dependency POSITION_LOOP → ITF_SPEED_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add dependency POSITION_LOOP ITF_SPEED_LOOP

assert_pass "dependency SPEED_LOOP → ITF_CURRENT_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add dependency SPEED_LOOP ITF_CURRENT_LOOP

# ————— 添加接口包含关系（接口层级） —————

section "C. Logic Model — 接口包含关系"

assert_pass "containment ITF_MOTOR_CTRL → ITF_POSITION_LOOP" \
    "$CLI" logic-model -m "$WORKSPACE" add containment ITF_MOTOR_CTRL ITF_POSITION_LOOP

assert_pass "containment ITF_CTRL → ITF_MOTOR_CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add containment ITF_CTRL ITF_MOTOR_CTRL

assert_pass "containment ITF_CTRL_SUBSYSTEM → ITF_CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" add containment ITF_CTRL_SUBSYSTEM ITF_CTRL

assert_pass "containment ITF_OMU → ITF_CTRL_SUBSYSTEM" \
    "$CLI" logic-model -m "$WORKSPACE" add containment ITF_OMU ITF_CTRL_SUBSYSTEM

# ————— List & Show —————

section "C. Logic Model — List & Show"

assert_contains "logic-model list subsystems 包含 CTRL_SUBSYSTEM" "CTRL_SUBSYSTEM" \
    "$CLI" logic-model -m "$WORKSPACE" list subsystems

assert_contains "logic-model list components 包含 CTRL" "CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" list components

assert_contains "logic-model list interfaces 包含 ITF_CTRL" "ITF_CTRL" \
    "$CLI" logic-model -m "$WORKSPACE" list interfaces

assert_contains "logic-model show CTRL 包含 Controller" "Controller" \
    "$CLI" logic-model -m "$WORKSPACE" show CTRL

# ————— 验证 —————

section "C. Logic Model — 验证"

# H001: System 无法直接包含 Module（模块应通过 Component 层级嵌套）
# 这是已知的设计约束，validate -t logic-view 会报 H001 错误
assert_contains "validate -t logic-view 报告 H001 层级错误" "H001" \
    "$CLI" validate -m "$WORKSPACE" -t logic-view

# ————— 图生成（在添加临时数据之前） —————

section "C. Logic Model — 图生成"

# 不存在的 root
assert_output_equals "logic-model-diagram 不存在的 root 报错" \
'@startuml

top to bottom direction
skinparam defaultTextAlignment center

'\'' ERROR: Element '\''MOTOR'\'' not found in diagram
'\'' Available elements: MOTOR_CTRL, POSITION_LOOP, SPEED_LOOP, CURRENT_LOOP, CTRL, CTRL_SUBSYSTEM, ITF_CTRL_SUBSYSTEM, ITF_CTRL, ITF_MOTOR_CTRL, ITF_POSITION_LOOP, ITF_SPEED_LOOP, ITF_CURRENT_LOOP

@enduml' \
    "$CLI" generate -m "$WORKSPACE" logic-model-diagram MOTOR

# MOTOR_CTRL root
assert_output_equals "logic-model-diagram MOTOR_CTRL root 输出匹配" \
'@startuml

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
POSITION_LOOP ..> ITF_SPEED_LOOP
SPEED_LOOP ..> ITF_CURRENT_LOOP

@enduml' \
    "$CLI" generate -m "$WORKSPACE" logic-model-diagram MOTOR_CTRL

# SNP root（只展开一层）
assert_output_equals "logic-model-diagram SNP root 输出匹配" \
'@startuml

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

@enduml' \
    "$CLI" generate -m "$WORKSPACE" logic-model-diagram SNP

# 不指定 root（完整图）
assert_contains "logic-model-diagram 不指定 root 生成完整图" "@startuml" \
    "$CLI" generate -m "$WORKSPACE" logic-model-diagram
