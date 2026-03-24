use crate::model::logic::concept::{Component, Interface, LogicConceptDiagram, Module, Subsystem, System};
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

    // Validate exposed interfaces
    for iface_id in &component.exposed_interfaces {
        validate_interface_id(result, iface_id, pattern, &format!("{}.exposed_interfaces", path));
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

    // Validate interfaces
    for iface in &module.interfaces {
        validate_interface(result, iface, pattern, &path);
    }

    // Validate dependencies (interface IDs)
    for dep_id in &module.dependencies {
        validate_interface_id(result, dep_id, pattern, &format!("{}.dependencies", path));
    }

    // Recursively validate nested modules
    for m in &module.modules {
        validate_module(result, m, pattern, &path);
    }
}

fn validate_interface(
    result: &mut ValidationResult,
    interface: &Interface,
    pattern: &Regex,
    parent_path: &str,
) {
    let path = format!("{}.interfaces.{}", parent_path, interface.id);

    // C001: Non-empty ID
    if interface.id.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "NonEmptyId".to_string(),
            message: "Interface ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some(path.clone()),
        });
    }

    // N001: UPPER_SNAKE_CASE naming (interfaces typically use ITF_ prefix)
    if !pattern.is_match(&interface.id) && !interface.id.is_empty() {
        result.add_error(ValidationError {
            code: "N001".to_string(),
            rule: "InterfaceIdNaming".to_string(),
            message: format!(
                "Interface ID '{}' does not follow UPPER_SNAKE_CASE convention",
                interface.id
            ),
            severity: Severity::Warning,
            location: Some(path),
        });
    }
}

fn validate_interface_id(
    result: &mut ValidationResult,
    interface_id: &str,
    pattern: &Regex,
    location: &str,
) {
    // C001: Non-empty ID
    if interface_id.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "NonEmptyId".to_string(),
            message: "Interface ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some(location.to_string()),
        });
    }

    // N001: UPPER_SNAKE_CASE naming
    if !pattern.is_match(interface_id) && !interface_id.is_empty() {
        result.add_error(ValidationError {
            code: "N001".to_string(),
            rule: "InterfaceIdNaming".to_string(),
            message: format!(
                "Interface ID '{}' does not follow UPPER_SNAKE_CASE convention",
                interface_id
            ),
            severity: Severity::Warning,
            location: Some(location.to_string()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_diagram() {
        let diagram = LogicConceptDiagram::new("MY_SYSTEM", "My System");
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
