#!/bin/bash
# test_context_model.sh — Context Model 集成测试

section "A. Context Model — CRUD 正向"

assert_pass "创建系统 SNP" \
    "$CLI" context-model -m "$WORKSPACE" add system SNP -n "SNP System" -d "SNP 核心系统"

assert_pass "添加 Actor USER" \
    "$CLI" context-model -m "$WORKSPACE" add actor USER -n "User" -t internal

assert_pass "添加外部系统 BD_SOFT" \
    "$CLI" context-model -m "$WORKSPACE" add external-system BD_SOFT -n "BD Soft" --tech "REST API"

assert_pass "添加外部系统 CHIP" \
    "$CLI" context-model -m "$WORKSPACE" add external-system CHIP -n "Chip" --tech "Hardware"

assert_pass "添加外部系统 CHIP_CTRL" \
    "$CLI" context-model -m "$WORKSPACE" add external-system CHIP_CTRL -n "Chip Controller" --tech "Hardware"

assert_pass "添加接口 ITF_OMU" \
    "$CLI" context-model -m "$WORKSPACE" add interface ITF_OMU -n "SNP OMU" -p rest

assert_pass "添加接口 ITF_SNP_CFG" \
    "$CLI" context-model -m "$WORKSPACE" add interface ITF_SNP_CFG -n "SNP Config Interface" -p rest

assert_pass "添加接口 ITF_DATA_PLANE" \
    "$CLI" context-model -m "$WORKSPACE" add interface ITF_DATA_PLANE -n "Data Plane Interface" -p grpc

assert_pass "添加接口 ITF_CHIP_CFG" \
    "$CLI" context-model -m "$WORKSPACE" add interface ITF_CHIP_CFG -n "Chip Config Interface" -p rest

assert_pass "添加提供关系 SNP → ITF_OMU" \
    "$CLI" context-model -m "$WORKSPACE" add provide-relation SNP ITF_OMU

assert_pass "添加提供关系 SNP → ITF_SNP_CFG" \
    "$CLI" context-model -m "$WORKSPACE" add provide-relation SNP ITF_SNP_CFG

assert_pass "添加提供关系 SNP → ITF_DATA_PLANE" \
    "$CLI" context-model -m "$WORKSPACE" add provide-relation SNP ITF_DATA_PLANE

assert_pass "添加提供关系 CHIP_CTRL → ITF_CHIP_CFG" \
    "$CLI" context-model -m "$WORKSPACE" add provide-relation CHIP_CTRL ITF_CHIP_CFG

assert_pass "添加使用关系 USER → ITF_OMU" \
    "$CLI" context-model -m "$WORKSPACE" add interface-usage USER ITF_OMU

assert_pass "添加使用关系 BD_SOFT → ITF_SNP_CFG" \
    "$CLI" context-model -m "$WORKSPACE" add interface-usage BD_SOFT ITF_SNP_CFG

assert_pass "添加使用关系 CHIP → ITF_DATA_PLANE" \
    "$CLI" context-model -m "$WORKSPACE" add interface-usage CHIP ITF_DATA_PLANE

assert_pass "添加使用关系 SNP → ITF_CHIP_CFG" \
    "$CLI" context-model -m "$WORKSPACE" add interface-usage SNP ITF_CHIP_CFG

# ————— CRUD 负向 —————

section "A. Context Model — CRUD 负向"

assert_fail "重复创建系统（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add system SNP -n "SNP System" -d "SNP"

assert_fail "重复添加 Actor（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add actor USER -n "User" -t internal

assert_fail "重复添加外部系统（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add external-system BD_SOFT -n "BD Soft" --tech "REST"

assert_fail "重复添加接口（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add interface ITF_OMU -n "SNP OMU" -p rest

assert_fail "重复添加提供关系（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add provide-relation SNP ITF_OMU

assert_fail "重复添加使用关系（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add interface-usage USER ITF_OMU

assert_fail "provide-relation 引用不存在的 system（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add provide-relation NONEXIST ITF_OMU

assert_fail "provide-relation 引用不存在的 interface（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add provide-relation SNP ITF_NONEXIST

assert_fail "interface-usage 引用不存在的 actor（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add interface-usage NONEXIST ITF_OMU

assert_fail "interface-usage 引用不存在的 interface（报错）" \
    "$CLI" context-model -m "$WORKSPACE" add interface-usage USER ITF_NONEXIST

# ————— List & Show —————

section "A. Context Model — List & Show"

assert_contains "list system 包含 SNP" "SNP" \
    "$CLI" context-model -m "$WORKSPACE" list system

assert_contains "list actors 包含 USER" "USER" \
    "$CLI" context-model -m "$WORKSPACE" list actors

assert_contains "list external-systems 包含 BD_SOFT" "BD_SOFT" \
    "$CLI" context-model -m "$WORKSPACE" list external-systems

assert_contains "list interfaces 包含 ITF_OMU" "ITF_OMU" \
    "$CLI" context-model -m "$WORKSPACE" list interfaces

assert_contains "list relations 包含 USER" "USER" \
    "$CLI" context-model -m "$WORKSPACE" list relations

assert_contains "show SNP 包含系统名称" "SNP System" \
    "$CLI" context-model -m "$WORKSPACE" show SNP

# ————— 验证 —————

section "A. Context Model — 验证"

assert_pass "validate -t context 通过" \
    "$CLI" validate -m "$WORKSPACE" -t context

# ————— 图生成 —————

section "A. Context Model — 图生成"

assert_output_equals "context-model-diagram 输出匹配" \
'@startuml

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

@enduml' \
    "$CLI" generate -m "$WORKSPACE" context-model-diagram

# ————— Remove —————
# 添加一次性元素用于 remove 测试，不影响后续测试

section "A. Context Model — Remove"

assert_pass "添加一次性 Actor TEMP_ACTOR" \
    "$CLI" context-model -m "$WORKSPACE" add actor TEMP_ACTOR -n "Temp Actor" -t external

assert_pass "添加一次性接口 ITF_TEMP" \
    "$CLI" context-model -m "$WORKSPACE" add interface ITF_TEMP -n "Temp Interface" -p rest

assert_pass "添加一次性 provide-relation SNP → ITF_TEMP" \
    "$CLI" context-model -m "$WORKSPACE" add provide-relation SNP ITF_TEMP

assert_pass "添加一次性 usage TEMP_ACTOR → ITF_TEMP" \
    "$CLI" context-model -m "$WORKSPACE" add interface-usage TEMP_ACTOR ITF_TEMP

# 正向 remove
assert_pass "remove interface-usage TEMP_ACTOR ITF_TEMP" \
    "$CLI" context-model -m "$WORKSPACE" remove interface-usage TEMP_ACTOR ITF_TEMP

assert_pass "remove provide-relation SNP ITF_TEMP" \
    "$CLI" context-model -m "$WORKSPACE" remove provide-relation SNP ITF_TEMP

assert_pass "remove interface ITF_TEMP（级联删除关联关系）" \
    "$CLI" context-model -m "$WORKSPACE" remove interface ITF_TEMP

assert_pass "remove actor TEMP_ACTOR" \
    "$CLI" context-model -m "$WORKSPACE" remove actor TEMP_ACTOR

# 负向 remove
assert_fail "remove 不存在的 actor（报错）" \
    "$CLI" context-model -m "$WORKSPACE" remove actor NONEXIST

assert_fail "remove 不存在的 external-system（报错）" \
    "$CLI" context-model -m "$WORKSPACE" remove external-system NONEXIST
