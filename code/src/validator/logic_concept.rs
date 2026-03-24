use crate::model::logic::concept::{Component, LogicConceptDiagram, Module, Submodule, Subsystem, System};
use crate::validator::result::{ValidationError, ValidationResult, Severity};
use regex::Regex;

/// Validate a LogicConceptDiagram
pub fn validate(diagram: &LogicConceptDiagram) -> ValidationResult {
    let mut result = ValidationResult::new();

    // ID pattern: UPPER_SNAKE_CASE
    let id_pattern = Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap();

    // Validate system
    validate_system(&mut result, &diagram.system, &id_pattern);

    result
}

fn validate_system(result: &mut ValidationResult, system: &System, pattern: &Regex) {
    // C001: Non-empty ID
    if system.id.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "NonEmptyId".to_string(),
            message: "System ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some("system.id".to_string()),
        });
    }

    // N001: UPPER_SNAKE_CASE naming
    if !pattern.is_match(&system.id) && !system.id.is_empty() {
        result.add_error(ValidationError {
            code: "N001".to_string(),
            rule: "SystemIdNaming".to_string(),
            message: format!(
                "System ID '{}' does not follow UPPER_SNAKE_CASE convention",
                system.id
            ),
            severity: Severity::Warning,
            location: Some("system.id".to_string()),
        });
    }

    // Validate subsystems
    for sub in &system.subsystems {
        validate_subsystem(result, sub, pattern, "system");
    }

    // Validate direct components
    for comp in &system.components {
        validate_component(result, comp, pattern, "system");
    }
}

fn validate_subsystem(
    result: &mut ValidationResult,
    subsystem: &Subsystem,
    pattern: &Regex,
    parent_path: &str,
) {
    let path = format!("{}.subsystems.{}", parent_path, subsystem.id);

    // C001: Non-empty ID
    if subsystem.id.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "NonEmptyId".to_string(),
            message: "Subsystem ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some(path.clone()),
        });
    }

    // N001: UPPER_SNAKE_CASE naming
    if !pattern.is_match(&subsystem.id) && !subsystem.id.is_empty() {
        result.add_error(ValidationError {
            code: "N001".to_string(),
            rule: "SubsystemIdNaming".to_string(),
            message: format!(
                "Subsystem ID '{}' does not follow UPPER_SNAKE_CASE convention",
                subsystem.id
            ),
            severity: Severity::Warning,
            location: Some(path.clone()),
        });
    }

    // Validate components
    for comp in &subsystem.components {
        validate_component(result, comp, pattern, &path);
    }
}

fn validate_component(
    result: &mut ValidationResult,
    component: &Component,
    pattern: &Regex,
    parent_path: &str,
) {
    let path = format!("{}.components.{}", parent_path, component.id);

    // C001: Non-empty ID
    if component.id.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "NonEmptyId".to_string(),
            message: "Component ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some(path.clone()),
        });
    }

    // N001: UPPER_SNAKE_CASE naming
    if !pattern.is_match(&component.id) && !component.id.is_empty() {
        result.add_error(ValidationError {
            code: "N001".to_string(),
            rule: "ComponentIdNaming".to_string(),
            message: format!(
                "Component ID '{}' does not follow UPPER_SNAKE_CASE convention",
                component.id
            ),
            severity: Severity::Warning,
            location: Some(path.clone()),
        });
    }

    // Validate modules
    for module in &component.modules {
        validate_module(result, module, pattern, &path);
    }
}

fn validate_module(
    result: &mut ValidationResult,
    module: &Module,
    pattern: &Regex,
    parent_path: &str,
) {
    let path = format!("{}.modules.{}", parent_path, module.id);

    // C001: Non-empty ID
    if module.id.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "NonEmptyId".to_string(),
            message: "Module ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some(path.clone()),
        });
    }

    // N001: UPPER_SNAKE_CASE naming
    if !pattern.is_match(&module.id) && !module.id.is_empty() {
        result.add_error(ValidationError {
            code: "N001".to_string(),
            rule: "ModuleIdNaming".to_string(),
            message: format!(
                "Module ID '{}' does not follow UPPER_SNAKE_CASE convention",
                module.id
            ),
            severity: Severity::Warning,
            location: Some(path.clone()),
        });
    }

    // Validate submodules
    for sub in &module.submodules {
        validate_submodule(result, sub, pattern, &path);
    }
}

fn validate_submodule(
    result: &mut ValidationResult,
    submodule: &Submodule,
    pattern: &Regex,
    parent_path: &str,
) {
    let path = format!("{}.submodules.{}", parent_path, submodule.id);

    // C001: Non-empty ID
    if submodule.id.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "NonEmptyId".to_string(),
            message: "Submodule ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some(path.clone()),
        });
    }

    // N001: UPPER_SNAKE_CASE naming
    if !pattern.is_match(&submodule.id) && !submodule.id.is_empty() {
        result.add_error(ValidationError {
            code: "N001".to_string(),
            rule: "SubmoduleIdNaming".to_string(),
            message: format!(
                "Submodule ID '{}' does not follow UPPER_SNAKE_CASE convention",
                submodule.id
            ),
            severity: Severity::Warning,
            location: Some(path.clone()),
        });
    }

    // Recursively validate nested submodules
    for sub in &submodule.submodules {
        validate_submodule(result, sub, pattern, &path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::logic::concept::LogicConceptDiagram;

    #[test]
    fn test_validate_valid_diagram() {
        let mut diagram = LogicConceptDiagram::new("MY_SYSTEM", "My System");
        diagram.system.subsystems.push(Subsystem {
            id: "SUB_SYS".to_string(),
            name: "Sub System".to_string(),
            description: None,
            components: vec![],
        });

        let result = validate(&diagram);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_empty_id() {
        let diagram = LogicConceptDiagram::new("", "Empty ID");
        let result = validate(&diagram);

        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "C001"));
    }

    #[test]
    fn test_validate_naming_convention() {
        let diagram = LogicConceptDiagram::new("my-system", "Bad Name");
        let result = validate(&diagram);

        assert!(result.errors.iter().any(|e| e.code == "N001"));
    }
}
