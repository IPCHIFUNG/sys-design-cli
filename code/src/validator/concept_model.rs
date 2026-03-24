use crate::model::logic::concept_model::LogicArchitectureConceptModel;
use crate::validator::result::{ValidationError, ValidationResult, Severity};
use regex::Regex;

/// Validate a LogicArchitectureConceptModel
pub fn validate(model: &LogicArchitectureConceptModel) -> ValidationResult {
    let mut result = ValidationResult::new();

    // ID pattern: UPPER_SNAKE_CASE
    let id_pattern = Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap();

    // Validate hierarchy levels
    for level in &model.hierarchy.levels {
        validate_level(&mut result, level, &id_pattern);
    }

    // Validate containment rules
    validate_containment_rules(&mut result, &model);

    result
}

fn validate_level(
    result: &mut ValidationResult,
    level: &crate::model::logic::concept_model::LevelDefinition,
    pattern: &Regex,
) {
    // C001: Non-empty ID
    if level.id.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "NonEmptyId".to_string(),
            message: "Level ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some("hierarchy.levels".to_string()),
        });
    }

    // N001: UPPER_SNAKE_CASE naming
    if !pattern.is_match(&level.id) && !level.id.is_empty() {
        result.add_error(ValidationError {
            code: "N001".to_string(),
            rule: "LevelIdNaming".to_string(),
            message: format!(
                "Level ID '{}' does not follow UPPER_SNAKE_CASE convention",
                level.id
            ),
            severity: Severity::Warning,
            location: Some(format!("hierarchy.levels.{}", level.id)),
        });
    }

    // Validate can_contain list
    for child_type in &level.can_contain {
        if child_type.is_empty() {
            result.add_error(ValidationError {
                code: "C002".to_string(),
                rule: "NonEmptyContainType".to_string(),
                message: format!(
                    "Level '{}' has empty containment type",
                    level.id
                ),
                severity: Severity::Error,
                location: Some(format!("hierarchy.levels.{}.can_contain", level.id)),
            });
        }
    }
}

fn validate_containment_rules(
    result: &mut ValidationResult,
    model: &LogicArchitectureConceptModel,
) {
    // Get all valid level IDs
    let valid_levels: std::collections::HashSet<&String> = model
        .hierarchy
        .levels
        .iter()
        .map(|l| &l.id)
        .collect();

    // Check that all can_contain references point to valid levels
    for level in &model.hierarchy.levels {
        for child_type in &level.can_contain {
            // Allow wildcard "*"
            if child_type != "*" && !valid_levels.contains(child_type) {
                result.add_error(ValidationError {
                    code: "S001".to_string(),
                    rule: "ValidContainmentReference".to_string(),
                    message: format!(
                        "Level '{}' contains unknown type '{}'",
                        level.id, child_type
                    ),
                    severity: Severity::Error,
                    location: Some(format!("hierarchy.levels.{}.can_contain", level.id)),
                });
            }
        }
    }

    // Check that there's at least one root level
    if model.hierarchy.levels.is_empty() {
        result.add_error(ValidationError {
            code: "C003".to_string(),
            rule: "NonEmptyHierarchy".to_string(),
            message: "Hierarchy must have at least one level".to_string(),
            severity: Severity::Error,
            location: Some("hierarchy.levels".to_string()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_default_model() {
        let model = LogicArchitectureConceptModel::new("Test Model");
        let result = validate(&model);
        assert!(result.is_valid);
    }
}
