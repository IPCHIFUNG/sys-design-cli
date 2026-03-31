#!/bin/bash
# test_runtime_model.sh — Runtime Model 集成测试

# 前提：前面三个测试文件已创建完整的 workspace

# ————— 添加场景和参与者 —————

section "D. Runtime Model — 添加场景和参与者"

assert_pass "添加场景 MOTOR_INIT" \
    "$CLI" runtime-model -m "$WORKSPACE" add scenario MOTOR_INIT -n "Motor Init Flow" -d "Motor initialization sequence"

assert_pass "添加参与者 USER (actor)" \
    "$CLI" runtime-model -m "$WORKSPACE" add participant MOTOR_INIT USER -t actor

assert_pass "添加参与者 CTRL (control)" \
    "$CLI" runtime-model -m "$WORKSPACE" add participant MOTOR_INIT CTRL -t control

assert_pass "添加参与者 MOTOR_CTRL (entity)" \
    "$CLI" runtime-model -m "$WORKSPACE" add participant MOTOR_INIT MOTOR_CTRL -t entity

# 负向
assert_fail "participant 引用不存在的 element（报错 R007）" \
    "$CLI" runtime-model -m "$WORKSPACE" add participant MOTOR_INIT NONEXIST_ELEMENT -t participant

assert_fail "重复添加 scenario MOTOR_INIT（报错）" \
    "$CLI" runtime-model -m "$WORKSPACE" add scenario MOTOR_INIT -n "dup" -d "dup"

# ————— 添加步骤（sync / return） —————

section "D. Runtime Model — 添加步骤"

assert_pass "step USER → CTRL 'Init motor' (sync)" \
    "$CLI" runtime-model -m "$WORKSPACE" add step MOTOR_INIT USER CTRL "Init motor"

assert_pass "step CTRL → MOTOR_CTRL 'Configure' (sync)" \
    "$CLI" runtime-model -m "$WORKSPACE" add step MOTOR_INIT CTRL MOTOR_CTRL "Configure"

assert_pass "step MOTOR_CTRL → CTRL 'Config done' (return)" \
    "$CLI" runtime-model -m "$WORKSPACE" add step MOTOR_INIT MOTOR_CTRL CTRL "Config done" -t return

assert_pass "step CTRL → USER 'Motor ready' (return)" \
    "$CLI" runtime-model -m "$WORKSPACE" add step MOTOR_INIT CTRL USER "Motor ready" -t return

# ————— 添加 alt 分组 + 分支内步骤 —————

section "D. Runtime Model — 添加分组"

assert_pass "group alt 'Result' with branches success,failure" \
    "$CLI" runtime-model -m "$WORKSPACE" add group MOTOR_INIT alt "Result" --branches success,failure

assert_pass "step CTRL → USER 'OK' (return) in success branch" \
    "$CLI" runtime-model -m "$WORKSPACE" add step MOTOR_INIT CTRL USER "OK" -t return --group "Result" --branch success

assert_pass "step CTRL → USER 'Error' (return) in failure branch" \
    "$CLI" runtime-model -m "$WORKSPACE" add step MOTOR_INIT CTRL USER "Error" -t return --group "Result" --branch failure

# ————— 更多类型：第二个场景 —————

section "D. Runtime Model — 更多分组类型和步骤类型"

assert_pass "添加场景 TYPE_TEST" \
    "$CLI" runtime-model -m "$WORKSPACE" add scenario TYPE_TEST -n "Type Test"

assert_pass "添加参与者 TYPE_TEST" \
    "$CLI" runtime-model -m "$WORKSPACE" add participant TYPE_TEST USER -t actor

assert_pass "添加参与者 CTRL 到 TYPE_TEST" \
    "$CLI" runtime-model -m "$WORKSPACE" add participant TYPE_TEST CTRL -t control

assert_pass "step async 类型" \
    "$CLI" runtime-model -m "$WORKSPACE" add step TYPE_TEST USER CTRL "Async msg" -t async

assert_pass "step lost 类型" \
    "$CLI" runtime-model -m "$WORKSPACE" add step TYPE_TEST CTRL USER "Lost msg" -t lost

assert_pass "group loop 类型" \
    "$CLI" runtime-model -m "$WORKSPACE" add group TYPE_TEST loop "Retry"

assert_pass "step in loop group" \
    "$CLI" runtime-model -m "$WORKSPACE" add step TYPE_TEST USER CTRL "Retry cmd" --group "Retry"

# ————— Note & Divider —————

section "D. Runtime Model — Note & Divider"

assert_pass "note left of USER 'Initiates motor setup'" \
    "$CLI" runtime-model -m "$WORKSPACE" add note MOTOR_INIT left USER "Initiates motor setup"

assert_pass "divider 'Config Phase' after-order 2" \
    "$CLI" runtime-model -m "$WORKSPACE" add divider MOTOR_INIT "Config Phase" --after-order 2

# ————— List & Show —————

section "D. Runtime Model — List & Show"

assert_contains "list scenarios 包含 MOTOR_INIT" "MOTOR_INIT" \
    "$CLI" runtime-model -m "$WORKSPACE" list scenarios

assert_contains "list participants 包含 USER" "USER" \
    "$CLI" runtime-model -m "$WORKSPACE" list participants --scenario MOTOR_INIT

assert_contains "list steps 包含 Init motor" "Init motor" \
    "$CLI" runtime-model -m "$WORKSPACE" list steps --scenario MOTOR_INIT

assert_contains "list groups 包含 Result" "Result" \
    "$CLI" runtime-model -m "$WORKSPACE" list groups --scenario MOTOR_INIT

assert_contains "show MOTOR_INIT 包含 Motor Init" "Motor Init" \
    "$CLI" runtime-model -m "$WORKSPACE" show MOTOR_INIT

# ————— 验证 —————

section "D. Runtime Model — 验证"

assert_pass "validate -t runtime-view 通过" \
    "$CLI" validate -m "$WORKSPACE" -t runtime-view

# ————— 图生成 —————

section "D. Runtime Model — 图生成"

assert_output_equals "runtime-model-diagram MOTOR_INIT 输出匹配" \
'@startuml

autonumber

actor "User" as USER
control "Controller" as CTRL
entity "Motor Controller" as MOTOR_CTRL

USER -> CTRL : Init motor
CTRL -> MOTOR_CTRL : Configure
MOTOR_CTRL --> CTRL : Config done
CTRL --> USER : Motor ready
alt Result
CTRL --> USER : OK
else failure
CTRL --> USER : Error
end
note left of USER
  Initiates motor setup
end note
== Config Phase ==

@enduml' \
    "$CLI" generate -m "$WORKSPACE" runtime-model-diagram MOTOR_INIT

# 不指定 scenario_id（单场景时自动选择）— 此 workspace 有多个场景，必须指定
assert_fail "多场景时不指定 scenario_id（报错）" \
    "$CLI" generate -m "$WORKSPACE" runtime-model-diagram

# 不存在的场景
assert_contains "runtime-model-diagram 不存在的场景报错" "not found" \
    "$CLI" generate -m "$WORKSPACE" runtime-model-diagram NONEXISTENT

# ————— Remove —————

section "D. Runtime Model — Remove"

# 创建临时场景用于 remove 测试
assert_pass "添加临时场景 DEL_TEST" \
    "$CLI" runtime-model -m "$WORKSPACE" add scenario DEL_TEST -n "Delete Test"

assert_pass "添加参与者到 DEL_TEST (USER, CTRL)" \
    "$CLI" runtime-model -m "$WORKSPACE" add participant DEL_TEST USER -t actor

assert_pass "添加参与者 CTRL 到 DEL_TEST" \
    "$CLI" runtime-model -m "$WORKSPACE" add participant DEL_TEST CTRL -t control

assert_pass "添加步骤到 DEL_TEST" \
    "$CLI" runtime-model -m "$WORKSPACE" add step DEL_TEST USER CTRL "Step1"

assert_pass "添加步骤到 DEL_TEST (return)" \
    "$CLI" runtime-model -m "$WORKSPACE" add step DEL_TEST CTRL USER "Step2" -t return

assert_pass "添加 note 到 DEL_TEST" \
    "$CLI" runtime-model -m "$WORKSPACE" add note DEL_TEST right CTRL "Test note"

assert_pass "添加 divider 到 DEL_TEST" \
    "$CLI" runtime-model -m "$WORKSPACE" add divider DEL_TEST "Phase 1" --after-order 1

# remove note
assert_pass "remove note DEL_TEST 0" \
    "$CLI" runtime-model -m "$WORKSPACE" remove note DEL_TEST 0

# remove divider
assert_pass "remove divider DEL_TEST 0" \
    "$CLI" runtime-model -m "$WORKSPACE" remove divider DEL_TEST 0

# remove step
assert_pass "remove step DEL_TEST 2" \
    "$CLI" runtime-model -m "$WORKSPACE" remove step DEL_TEST 2

# remove participant（级联删除相关步骤）
assert_pass "remove participant DEL_TEST CTRL（级联删除步骤）" \
    "$CLI" runtime-model -m "$WORKSPACE" remove participant DEL_TEST CTRL

# 验证级联删除：Step1 应已消失
TOTAL=$((TOTAL + 1))
STEPS=$("$CLI" runtime-model -m "$WORKSPACE" list steps --scenario DEL_TEST 2>&1)
if echo "$STEPS" | grep -q "Step1"; then
    FAIL=$((FAIL + 1))
    echo -e "  ${RED}FAIL${NC} 删除参与者后步骤未被级联删除"
else
    PASS=$((PASS + 1))
    echo -e "  ${GREEN}PASS${NC} 删除参与者后步骤被级联删除"
fi

# remove scenario
assert_pass "remove scenario DEL_TEST" \
    "$CLI" runtime-model -m "$WORKSPACE" remove scenario DEL_TEST

# 验证场景已删除
TOTAL=$((TOTAL + 1))
SCENARIOS=$("$CLI" runtime-model -m "$WORKSPACE" list scenarios 2>&1)
if echo "$SCENARIOS" | grep -q "DEL_TEST"; then
    FAIL=$((FAIL + 1))
    echo -e "  ${RED}FAIL${NC} 删除场景后场景仍存在"
else
    PASS=$((PASS + 1))
    echo -e "  ${GREEN}PASS${NC} 删除场景后场景不再存在"
fi
