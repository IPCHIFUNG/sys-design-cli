use crate::model::runtime::{Block, GroupType, RuntimeView, Scenario};
use crate::validator::result::{Severity, ValidationError, ValidationResult};
use regex::Regex;

/// Validate a RuntimeView
pub fn validate(view: &RuntimeView) -> ValidationResult {
    let mut result = ValidationResult::new();
    let id_pattern = Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap();

    for scenario in &view.scenarios {
        validate_scenario(&mut result, scenario, &id_pattern);
    }

    result
}

fn validate_scenario(result: &mut ValidationResult, scenario: &Scenario, pattern: &Regex) {
    let location = format!("scenarios.{}", scenario.id);

    // R001: Scenario must have non-empty ID and name
    if scenario.id.is_empty() {
        result.add_error(ValidationError {
            code: "R001".to_string(),
            rule: "ScenarioRequiredFields".to_string(),
            message: "Scenario ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some(format!("{}.id", location)),
        });
    }

    if scenario.name.is_empty() {
        result.add_error(ValidationError {
            code: "R001".to_string(),
            rule: "ScenarioRequiredFields".to_string(),
            message: format!("Scenario '{}' name cannot be empty", scenario.id),
            severity: Severity::Error,
            location: Some(format!("{}.name", location)),
        });
    }

    // R002: Participant must have non-empty element_id
    for participant in &scenario.participants {
        if participant.element_id.is_empty() {
            result.add_error(ValidationError {
                code: "R002".to_string(),
                rule: "ParticipantRequiredFields".to_string(),
                message: format!(
                    "Participant in scenario '{}' has empty element_id",
                    scenario.id
                ),
                severity: Severity::Error,
                location: Some(format!(
                    "{}.participants.{}",
                    location, participant.element_id
                )),
            });
        }
    }

    // R004: At least 2 participants (warning)
    if scenario.participants.len() < 2 {
        result.add_error(ValidationError {
            code: "R004".to_string(),
            rule: "MinParticipants".to_string(),
            message: format!(
                "Scenario '{}' should have at least 2 participants (has {})",
                scenario.id,
                scenario.participants.len()
            ),
            severity: Severity::Warning,
            location: Some(location.clone()),
        });
    }

    // Collect participant IDs for step validation
    let participant_ids = scenario.participant_id_set();

    // Validate blocks tree
    let mut all_orders: Vec<u32> = Vec::new();
    let mut has_steps = false;
    validate_blocks(
        result,
        &scenario.blocks,
        &participant_ids,
        &location,
        &mut all_orders,
        &mut has_steps,
    );

    // R005: At least 1 step (warning)
    if !has_steps {
        result.add_error(ValidationError {
            code: "R005".to_string(),
            rule: "MinSteps".to_string(),
            message: format!("Scenario '{}' should have at least 1 step", scenario.id),
            severity: Severity::Warning,
            location: Some(location.clone()),
        });
    }

    // R008: Step order must be unique
    all_orders.sort();
    for window in all_orders.windows(2) {
        if window[0] == window[1] {
            result.add_error(ValidationError {
                code: "R008".to_string(),
                rule: "StepOrderUnique".to_string(),
                message: format!(
                    "Duplicate step order {} in scenario '{}'",
                    window[0], scenario.id
                ),
                severity: Severity::Error,
                location: Some(location.clone()),
            });
            break; // Only report once
        }
    }

    // R011: Scenario ID should follow UPPER_SNAKE_CASE
    if !pattern.is_match(&scenario.id) && !scenario.id.is_empty() {
        result.add_error(ValidationError {
            code: "R011".to_string(),
            rule: "ScenarioIdNaming".to_string(),
            message: format!(
                "Scenario ID '{}' does not follow UPPER_SNAKE_CASE convention",
                scenario.id
            ),
            severity: Severity::Warning,
            location: Some(format!("{}.id", location)),
        });
    }

    // R012: Scenario name length
    if scenario.name.len() > 50 {
        result.add_error(ValidationError {
            code: "R012".to_string(),
            rule: "ScenarioNameLength".to_string(),
            message: format!(
                "Scenario '{}' name exceeds 50 characters (has {})",
                scenario.id,
                scenario.name.len()
            ),
            severity: Severity::Info,
            location: Some(format!("{}.name", location)),
        });
    }
}

fn validate_blocks(
    result: &mut ValidationResult,
    blocks: &[Block],
    participant_ids: &std::collections::HashSet<&str>,
    location: &str,
    all_orders: &mut Vec<u32>,
    has_steps: &mut bool,
) {
    for block in blocks {
        match block {
            Block::Step(step) => {
                *has_steps = true;
                let step_loc = format!("{}.step.{}", location, step.order);

                // R003: Step must have non-empty from, to, message
                if step.from.is_empty() {
                    result.add_error(ValidationError {
                        code: "R003".to_string(),
                        rule: "StepRequiredFields".to_string(),
                        message: format!("Step {} has empty 'from'", step.order),
                        severity: Severity::Error,
                        location: Some(format!("{}.from", step_loc)),
                    });
                }
                if step.to.is_empty() {
                    result.add_error(ValidationError {
                        code: "R003".to_string(),
                        rule: "StepRequiredFields".to_string(),
                        message: format!("Step {} has empty 'to'", step.order),
                        severity: Severity::Error,
                        location: Some(format!("{}.to", step_loc)),
                    });
                }
                if step.message.is_empty() {
                    result.add_error(ValidationError {
                        code: "R003".to_string(),
                        rule: "StepRequiredFields".to_string(),
                        message: format!("Step {} has empty 'message'", step.order),
                        severity: Severity::Error,
                        location: Some(format!("{}.message", step_loc)),
                    });
                }

                // R006: Step from/to must reference defined participants
                if !step.from.is_empty() && !participant_ids.contains(step.from.as_str()) {
                    result.add_error(ValidationError {
                        code: "R006".to_string(),
                        rule: "StepReferencesValid".to_string(),
                        message: format!(
                            "Step {} 'from' references '{}' which is not a participant in this scenario",
                            step.order, step.from
                        ),
                        severity: Severity::Error,
                        location: Some(format!("{}.from", step_loc)),
                    });
                }
                if !step.to.is_empty() && !participant_ids.contains(step.to.as_str()) {
                    result.add_error(ValidationError {
                        code: "R006".to_string(),
                        rule: "StepReferencesValid".to_string(),
                        message: format!(
                            "Step {} 'to' references '{}' which is not a participant in this scenario",
                            step.order, step.to
                        ),
                        severity: Severity::Error,
                        location: Some(format!("{}.to", step_loc)),
                    });
                }

                all_orders.push(step.order);
            }
            Block::Group(group) => {
                let group_loc = format!("{}.group.{}", location, group.label);

                // R009: Alt must have branches
                if group.group_type == GroupType::Alt && group.branches.is_empty() {
                    result.add_error(ValidationError {
                        code: "R009".to_string(),
                        rule: "AltRequiresBranches".to_string(),
                        message: format!("Alt group '{}' must have at least one branch", group.label),
                        severity: Severity::Error,
                        location: Some(group_loc.clone()),
                    });
                }

                // R010: Non-alt should not have branches
                if group.group_type != GroupType::Alt && !group.branches.is_empty() {
                    result.add_error(ValidationError {
                        code: "R010".to_string(),
                        rule: "NonAltShouldNotHaveBranches".to_string(),
                        message: format!(
                            "Non-alt group '{}' (type: {:?}) should not have branches",
                            group.label, group.group_type
                        ),
                        severity: Severity::Warning,
                        location: Some(group_loc.clone()),
                    });
                }

                // Validate inner blocks
                if group.branches.is_empty() {
                    validate_blocks(
                        result,
                        &group.blocks,
                        participant_ids,
                        &group_loc,
                        all_orders,
                        has_steps,
                    );
                } else {
                    for branch in &group.branches {
                        let branch_loc = format!("{}.{}", group_loc, branch.label);
                        validate_blocks(
                            result,
                            &branch.blocks,
                            participant_ids,
                            &branch_loc,
                            all_orders,
                            has_steps,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::runtime::*;

    fn create_test_scenario() -> Scenario {
        Scenario {
            id: "TEST_SCENARIO".to_string(),
            name: "Test Scenario".to_string(),
            description: None,
            participants: vec![
                Participant {
                    element_id: "A".to_string(),
                    participant_type: ParticipantType::Participant,
                    alias: None,
                    color: None,
                },
                Participant {
                    element_id: "B".to_string(),
                    participant_type: ParticipantType::Participant,
                    alias: None,
                    color: None,
                },
            ],
            blocks: vec![Block::Step(Step {
                order: 1,
                from: "A".to_string(),
                to: "B".to_string(),
                message: "hello".to_string(),
                step_type: StepType::Sync,
                protocol: None,
                color: None,
                activate_target: None,
            })],
            notes: Vec::new(),
            dividers: Vec::new(),
        }
    }

    #[test]
    fn test_validate_valid_scenario() {
        let mut view = RuntimeView::new("Test");
        view.scenarios.push(create_test_scenario());
        let result = validate(&view);
        assert!(result.is_valid);
    }

    #[test]
    fn test_r001_empty_id() {
        let mut view = RuntimeView::new("Test");
        let mut scenario = create_test_scenario();
        scenario.id = "".to_string();
        view.scenarios.push(scenario);
        let result = validate(&view);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "R001"));
    }

    #[test]
    fn test_r004_min_participants() {
        let mut view = RuntimeView::new("Test");
        let mut scenario = create_test_scenario();
        scenario.participants.truncate(1);
        view.scenarios.push(scenario);
        let result = validate(&view);
        assert!(result.errors.iter().any(|e| e.code == "R004" && e.severity == Severity::Warning));
    }

    #[test]
    fn test_r006_invalid_reference() {
        let mut view = RuntimeView::new("Test");
        let mut scenario = create_test_scenario();
        if let Block::Step(s) = &mut scenario.blocks[0] {
            s.from = "NONEXISTENT".to_string();
        }
        view.scenarios.push(scenario);
        let result = validate(&view);
        assert!(result.errors.iter().any(|e| e.code == "R006"));
    }

    #[test]
    fn test_r009_alt_without_branches() {
        let mut view = RuntimeView::new("Test");
        let mut scenario = create_test_scenario();
        scenario.blocks.push(Block::Group(Group {
            group_type: GroupType::Alt,
            label: "result".to_string(),
            blocks: Vec::new(),
            branches: Vec::new(), // Empty!
        }));
        view.scenarios.push(scenario);
        let result = validate(&view);
        assert!(result.errors.iter().any(|e| e.code == "R009"));
    }

    #[test]
    fn test_r010_non_alt_with_branches() {
        let mut view = RuntimeView::new("Test");
        let mut scenario = create_test_scenario();
        scenario.blocks.push(Block::Group(Group {
            group_type: GroupType::Loop,
            label: "retry".to_string(),
            blocks: Vec::new(),
            branches: vec![GroupBranch {
                label: "body".to_string(),
                blocks: Vec::new(),
            }],
        }));
        view.scenarios.push(scenario);
        let result = validate(&view);
        assert!(result.errors.iter().any(|e| e.code == "R010"));
    }

    #[test]
    fn test_r011_naming_convention() {
        let mut view = RuntimeView::new("Test");
        let mut scenario = create_test_scenario();
        scenario.id = "bad-name".to_string();
        view.scenarios.push(scenario);
        let result = validate(&view);
        assert!(result.errors.iter().any(|e| e.code == "R011"));
    }

    #[test]
    fn test_r012_name_length() {
        let mut view = RuntimeView::new("Test");
        let mut scenario = create_test_scenario();
        scenario.name = "x".repeat(51);
        view.scenarios.push(scenario);
        let result = validate(&view);
        assert!(result.errors.iter().any(|e| e.code == "R012"));
    }
}
