#!/bin/bash
# test_concept_model.sh — Concept Model 集成测试

# 前提：test_context_model.sh 已创建 workspace 并包含 context model

# ————— 概念模型不存在时，逻辑元素操作应报错 —————

section "B. Concept Model — 不存在时的约束"

assert_fail "概念模型不存在时添加 subsystem（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add subsystem MOTOR -n "Motor" -d "Motor"

assert_fail "概念模型不存在时添加 component（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add component MOTOR -n "Motor" -d "Motor"

assert_fail "概念模型不存在时添加 module（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add module MOTOR -n "Motor" -d "Motor"

assert_fail "概念模型不存在时添加 submodule（报错）" \
    "$CLI" logic-model -m "$WORKSPACE" add submodule MOTOR -n "Motor" -d "Motor"

# ————— 添加元素类型 —————

section "B. Concept Model — 添加元素类型"

assert_pass "添加 element subsystem" \
    "$CLI" concept-model -m "$WORKSPACE" add element subsystem

assert_pass "添加 element component" \
    "$CLI" concept-model -m "$WORKSPACE" add element component

assert_pass "添加 element module" \
    "$CLI" concept-model -m "$WORKSPACE" add element module

assert_pass "添加 element submodule" \
    "$CLI" concept-model -m "$WORKSPACE" add element submodule

assert_fail "重复添加 element subsystem（报错）" \
    "$CLI" concept-model -m "$WORKSPACE" add element subsystem

# ————— 游离元素验证 —————

section "B. Concept Model — 游离元素验证"

assert_fail "存在游离概念模型元素时 validate 报错" \
    "$CLI" validate -m "$WORKSPACE"

# ————— 添加包含关系 —————

section "B. Concept Model — 添加包含关系"

assert_pass "添加 containment system → subsystem" \
    "$CLI" concept-model -m "$WORKSPACE" add containment system subsystem

assert_pass "添加 containment subsystem → component" \
    "$CLI" concept-model -m "$WORKSPACE" add containment subsystem component

assert_pass "添加 containment system → component" \
    "$CLI" concept-model -m "$WORKSPACE" add containment system component

assert_pass "添加 containment component → module" \
    "$CLI" concept-model -m "$WORKSPACE" add containment component module

assert_pass "添加 containment module → submodule" \
    "$CLI" concept-model -m "$WORKSPACE" add containment module submodule

assert_pass "添加 containment submodule → submodule（递归）" \
    "$CLI" concept-model -m "$WORKSPACE" add containment submodule submodule

assert_fail "重复添加 containment system → subsystem（报错）" \
    "$CLI" concept-model -m "$WORKSPACE" add containment system subsystem

assert_fail "containment 引用不存在的 parent（报错）" \
    "$CLI" concept-model -m "$WORKSPACE" add containment aaa subsystem

assert_fail "containment 引用不存在的 child（报错）" \
    "$CLI" concept-model -m "$WORKSPACE" add containment system aaa

# ————— List & Show —————

section "B. Concept Model — List & Show"

assert_contains "concept-model list 包含 subsystem" "subsystem" \
    "$CLI" concept-model -m "$WORKSPACE" list

assert_contains "concept-model show SYSTEM 包含 SUBSYSTEM" "SUBSYSTEM" \
    "$CLI" concept-model -m "$WORKSPACE" show SYSTEM

# ————— 验证 —————

section "B. Concept Model — 验证"

assert_pass "validate -t concept-model 通过" \
    "$CLI" validate -m "$WORKSPACE" -t concept-model

# ————— 图生成（在添加 SERVICE 之前，保持输出干净） —————

section "B. Concept Model — 图生成"

assert_output_equals "concept-model-diagram 输出匹配" \
'@startuml

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

@enduml' \
    "$CLI" generate -m "$WORKSPACE" concept-model-diagram

# ————— 添加 Level（使用新元素类型 SERVICE，不干扰已有数据） —————

section "B. Concept Model — 添加 Level"

assert_pass "添加 element service" \
    "$CLI" concept-model -m "$WORKSPACE" add element service

assert_pass "添加 level SERVICE -c COMPONENT" \
    "$CLI" concept-model -m "$WORKSPACE" add level SERVICE -n "service" -c COMPONENT

assert_fail "重复添加 level SERVICE（报错）" \
    "$CLI" concept-model -m "$WORKSPACE" add level SERVICE -n "service" -c COMPONENT

assert_fail "重复添加 level SERVICE（报错）" \
    "$CLI" concept-model -m "$WORKSPACE" add level SERVICE -n "service" -c COMPONENT
