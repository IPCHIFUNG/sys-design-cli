#!/bin/bash
# test_deployment_model.sh — Deployment Model 集成测试（I节）

# 前提：H 节已保留 CORE_IMG

# ————— CRUD 正向 —————

section "I. Deployment Model — CRUD 正向"

assert_pass "add environment PROD" \
    "$CLI" deployment-model -m "$WORKSPACE" add environment PROD -n "Production"

assert_pass "add environment STAGING" \
    "$CLI" deployment-model -m "$WORKSPACE" add environment STAGING -n "Staging"

assert_pass "add node K8S_CLUSTER" \
    "$CLI" deployment-model -m "$WORKSPACE" add node K8S_CLUSTER -n "K8s Cluster" \
        -t kubernetes --environment PROD --technology k3s

assert_pass "add node STAGING_NODE" \
    "$CLI" deployment-model -m "$WORKSPACE" add node STAGING_NODE -n "Staging Node" \
        -t vm --environment STAGING

assert_pass "add service API_SVC" \
    "$CLI" deployment-model -m "$WORKSPACE" add service API_SVC -n "API Service" \
        --delivery-package CORE_IMG --target-node K8S_CLUSTER --replicas 3 --port 8080

assert_pass "add service DB_SVC" \
    "$CLI" deployment-model -m "$WORKSPACE" add service DB_SVC -n "DB Service" \
        --delivery-package CORE_IMG --target-node K8S_CLUSTER --port 5432

assert_pass "add network-link API_TO_DB" \
    "$CLI" deployment-model -m "$WORKSPACE" add network-link API_TO_DB \
        --from API_SVC --to DB_SVC -p http --port 5432

# ————— CRUD 负向 —————

section "I. Deployment Model — CRUD 负向"

assert_fail "重复添加 environment PROD（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" add environment PROD -n "dup"

assert_fail "重复添加 node K8S_CLUSTER（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" add node K8S_CLUSTER -n "dup"

assert_fail "重复添加 service API_SVC（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" add service API_SVC -n "dup" \
        --delivery-package CORE_IMG --target-node K8S_CLUSTER

assert_fail "重复添加 network-link API_TO_DB（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" add network-link API_TO_DB \
        --from API_SVC --to DB_SVC

assert_fail "add node 引用不存在的 environment（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" add node BAD_NODE -n "bad" \
        --environment NONEXIST

assert_fail "add service 引用不存在的 target_node（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" add service BAD_SVC -n "bad" \
        --delivery-package CORE_IMG --target-node NONEXIST

assert_fail "add service 引用不存在的 delivery_package（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" add service BAD_SVC2 -n "bad" \
        --delivery-package NONEXIST --target-node K8S_CLUSTER

assert_fail "add network-link 引用不存在的 from（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" add network-link BAD_LINK \
        --from NONEXIST --to DB_SVC

assert_fail "add network-link 引用不存在的 to（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" add network-link BAD_LINK \
        --from API_SVC --to NONEXIST

# ————— List & Show —————

section "I. Deployment Model — List & Show"

assert_contains "list environments 包含 PROD" "PROD" \
    "$CLI" deployment-model -m "$WORKSPACE" list environments

assert_contains "list nodes 包含 K8S_CLUSTER" "K8S_CLUSTER" \
    "$CLI" deployment-model -m "$WORKSPACE" list nodes

assert_contains "list services 包含 API_SVC" "API_SVC" \
    "$CLI" deployment-model -m "$WORKSPACE" list services

assert_contains "list network-links 包含 API_TO_DB" "API_TO_DB" \
    "$CLI" deployment-model -m "$WORKSPACE" list network-links

assert_contains "show API_SVC 包含 API Service" "API Service" \
    "$CLI" deployment-model -m "$WORKSPACE" show API_SVC

assert_contains "show PROD 包含 Production" "Production" \
    "$CLI" deployment-model -m "$WORKSPACE" show PROD

assert_contains "show K8S_CLUSTER 包含 K8s Cluster" "K8s Cluster" \
    "$CLI" deployment-model -m "$WORKSPACE" show K8S_CLUSTER

# ————— Validate —————

section "I. Deployment Model — Validate"

assert_pass "validate -t deployment-model 通过" \
    "$CLI" validate -m "$WORKSPACE" -t deployment-model

# ————— Generate —————

section "I. Deployment Model — Generate"

assert_contains "generate deployment-model-diagram PROD 包含 @startuml" "@startuml" \
    "$CLI" generate -m "$WORKSPACE" deployment-model-diagram PROD

assert_contains "generate deployment-model-diagram PROD 包含 K8S_CLUSTER" "K8S_CLUSTER" \
    "$CLI" generate -m "$WORKSPACE" deployment-model-diagram PROD

assert_fail "多 environment 不指定 environment_id（报错）" \
    "$CLI" generate -m "$WORKSPACE" deployment-model-diagram

# ————— Remove —————

section "I. Deployment Model — Remove"

# 先验证阻塞删除
assert_fail "remove environment PROD（有 node 引用，报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" remove environment PROD

assert_fail "remove node K8S_CLUSTER（有 service 引用，报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" remove node K8S_CLUSTER

# 正向按序删除
assert_pass "remove network-link API_TO_DB" \
    "$CLI" deployment-model -m "$WORKSPACE" remove network-link API_TO_DB

# 删除 service 时验证级联：创建临时 link，删除 service 后 link 应消失
assert_pass "添加临时 network-link CASCADE_TEST" \
    "$CLI" deployment-model -m "$WORKSPACE" add network-link CASCADE_TEST \
        --from API_SVC --to DB_SVC -p tcp

assert_pass "remove service API_SVC（级联删除 CASCADE_TEST）" \
    "$CLI" deployment-model -m "$WORKSPACE" remove service API_SVC

# 验证级联：CASCADE_TEST 应已消失
TOTAL=$((TOTAL + 1))
LINKS=$("$CLI" deployment-model -m "$WORKSPACE" list network-links 2>&1)
if echo "$LINKS" | grep -q "CASCADE_TEST"; then
    FAIL=$((FAIL + 1))
    echo -e "  ${RED}FAIL${NC} 删除 service 后 network-link 未被级联删除"
else
    PASS=$((PASS + 1))
    echo -e "  ${GREEN}PASS${NC} 删除 service 后 network-link 被级联删除"
fi

assert_pass "remove service DB_SVC" \
    "$CLI" deployment-model -m "$WORKSPACE" remove service DB_SVC

assert_pass "remove node K8S_CLUSTER" \
    "$CLI" deployment-model -m "$WORKSPACE" remove node K8S_CLUSTER

assert_pass "remove node STAGING_NODE" \
    "$CLI" deployment-model -m "$WORKSPACE" remove node STAGING_NODE

assert_pass "remove environment PROD" \
    "$CLI" deployment-model -m "$WORKSPACE" remove environment PROD

assert_pass "remove environment STAGING" \
    "$CLI" deployment-model -m "$WORKSPACE" remove environment STAGING

# 负向 remove
assert_fail "remove 不存在的 environment（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" remove environment NONEXIST

assert_fail "remove 不存在的 node（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" remove node NONEXIST

assert_fail "remove 不存在的 service（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" remove service NONEXIST

assert_fail "remove 不存在的 network-link（报错）" \
    "$CLI" deployment-model -m "$WORKSPACE" remove network-link NONEXIST
