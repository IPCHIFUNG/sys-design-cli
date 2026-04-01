#!/bin/bash
# test_delivery_model.sh — Delivery Model 集成测试（H节）

# 前提：G 节已保留 CORE_BIN 和 UTIL_LIB_A

# ————— CRUD 正向 —————

section "H. Delivery Model — CRUD 正向"

assert_pass "add package CORE_IMG" \
    "$CLI" delivery-model -m "$WORKSPACE" add package CORE_IMG -n "Core Image" \
        -v 1.0.0 --delivery-type container-image \
        --artifacts CORE_BIN,UTIL_LIB_A --registry registry.example.com/core

# ————— CRUD 负向 —————

section "H. Delivery Model — CRUD 负向"

assert_fail "重复添加 package CORE_IMG（报错）" \
    "$CLI" delivery-model -m "$WORKSPACE" add package CORE_IMG -n "dup"

# ————— List & Show —————

section "H. Delivery Model — List & Show"

assert_contains "list packages 包含 CORE_IMG" "CORE_IMG" \
    "$CLI" delivery-model -m "$WORKSPACE" list packages

assert_contains "show CORE_IMG 包含 Core Image" "Core Image" \
    "$CLI" delivery-model -m "$WORKSPACE" show CORE_IMG

# ————— Validate & Generate —————

section "H. Delivery Model — Validate & Generate"

assert_pass "validate -t delivery-model 通过" \
    "$CLI" validate -m "$WORKSPACE" -t delivery-model

assert_contains "generate delivery-model-diagram 包含 @startuml" "@startuml" \
    "$CLI" generate -m "$WORKSPACE" delivery-model-diagram

assert_contains "generate delivery-model-diagram 包含 CORE_IMG" "CORE_IMG" \
    "$CLI" generate -m "$WORKSPACE" delivery-model-diagram

# ————— Remove（临时元素，保留 CORE_IMG 给后续 I 节） —————

section "H. Delivery Model — Remove"

assert_pass "添加临时 package TEMP_PKG_D" \
    "$CLI" delivery-model -m "$WORKSPACE" add package TEMP_PKG_D -n "Temp Delivery"

assert_pass "remove package TEMP_PKG_D" \
    "$CLI" delivery-model -m "$WORKSPACE" remove package TEMP_PKG_D

assert_fail "remove 不存在的 package（报错）" \
    "$CLI" delivery-model -m "$WORKSPACE" remove package NONEXIST
