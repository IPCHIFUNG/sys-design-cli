use crate::model::c4::context::ContextDiagram;
use crate::validator::result::{ValidationError, ValidationResult, Severity};
use regex::Regex;

/// Naming convention validation rules
pub fn validate(diagram: &ContextDiagram) -> ValidationResult {
    let mut result = ValidationResult::new();

    // ID pattern: kebab-case (lowercase letters, numbers, hyphens)
    let id_pattern = Regex::new(r"^[a-z][a-z0-9-]*$").unwrap();

    // N001: System ID naming convention
    if !id_pattern.is_match(&diagram.system.id) && !diagram.system.id.is_empty() {
        result.add_error(ValidationError {
            code: "N001".to_string(),
            rule: "SystemIdNaming".to_string(),
            message: format!(
                "System ID '{}' does not follow kebab-case convention (lowercase letters, numbers, hyphens)",
                diagram.system.id
            ),
            severity: Severity::Warning,
            location: Some("system.id".to_string()),
        });
    }

    // N001: Actor ID naming convention
    for actor in &diagram.actors {
        if !id_pattern.is_match(&actor.id) && !actor.id.is_empty() {
            result.add_error(ValidationError {
                code: "N001".to_string(),
                rule: "ActorIdNaming".to_string(),
                message: format!(
                    "Actor ID '{}' does not follow kebab-case convention",
                    actor.id
                ),
                severity: Severity::Warning,
                location: Some(format!("actors.id={}", actor.id)),
            });
        }
    }

    // N001: External system ID naming convention
    for ext in &diagram.external_systems {
        if !id_pattern.is_match(&ext.id) && !ext.id.is_empty() {
            result.add_error(ValidationError {
                code: "N001".to_string(),
                rule: "ExternalSystemIdNaming".to_string(),
                message: format!(
                    "External system ID '{}' does not follow kebab-case convention",
                    ext.id
                ),
                severity: Severity::Warning,
                location: Some(format!("external_systems.id={}", ext.id)),
            });
        }
    }

    // N001: Interface ID naming convention
    for iface in &diagram.interfaces {
        if !id_pattern.is_match(&iface.id) && !iface.id.is_empty() {
            result.add_error(ValidationError {
                code: "N001".to_string(),
                rule: "InterfaceIdNaming".to_string(),
                message: format!(
                    "Interface ID '{}' does not follow kebab-case convention",
                    iface.id
                ),
                severity: Severity::Warning,
                location: Some(format!("interfaces.id={}", iface.id)),
            });
        }
    }

    // N002: Name length check (max 50 characters)
    const MAX_NAME_LENGTH: usize = 50;

    if diagram.system.name.len() > MAX_NAME_LENGTH {
        result.add_error(ValidationError {
            code: "N002".to_string(),
            rule: "NameLength".to_string(),
            message: format!(
                "System name is too long ({} chars). Consider keeping it under {} chars",
                diagram.system.name.len(),
                MAX_NAME_LENGTH
            ),
            severity: Severity::Info,
            location: Some("system.name".to_string()),
        });
    }

    for actor in &diagram.actors {
        if actor.name.len() > MAX_NAME_LENGTH {
            result.add_error(ValidationError {
                code: "N002".to_string(),
                rule: "NameLength".to_string(),
                message: format!(
                    "Actor '{}' name is too long ({} chars)",
                    actor.id,
                    actor.name.len()
                ),
                severity: Severity::Info,
                location: Some(format!("actors.id={}", actor.id)),
            });
        }
    }

    for ext in &diagram.external_systems {
        if ext.name.len() > MAX_NAME_LENGTH {
            result.add_error(ValidationError {
                code: "N002".to_string(),
                rule: "NameLength".to_string(),
                message: format!(
                    "External system '{}' name is too long ({} chars)",
                    ext.id,
                    ext.name.len()
                ),
                severity: Severity::Info,
                location: Some(format!("external_systems.id={}", ext.id)),
            });
        }
    }

    // N003: Reserved word check
    const RESERVED_WORDS: &[&str] = &[
        "system", "actor", "interface", "external", "container", "component",
        "null", "undefined", "true", "false", "default",
    ];

    for &word in RESERVED_WORDS {
        if diagram.system.id == word {
            result.add_error(ValidationError {
                code: "N003".to_string(),
                rule: "ReservedWord".to_string(),
                message: format!("System ID '{}' is a reserved word", word),
                severity: Severity::Error,
                location: Some("system.id".to_string()),
            });
        }

        for actor in &diagram.actors {
            if actor.id == word {
                result.add_error(ValidationError {
                    code: "N003".to_string(),
                    rule: "ReservedWord".to_string(),
                    message: format!("Actor ID '{}' is a reserved word", word),
                    severity: Severity::Error,
                    location: Some(format!("actors.id={}", actor.id)),
                });
            }
        }

        for ext in &diagram.external_systems {
            if ext.id == word {
                result.add_error(ValidationError {
                    code: "N003".to_string(),
                    rule: "ReservedWord".to_string(),
                    message: format!("External system ID '{}' is a reserved word", word),
                    severity: Severity::Error,
                    location: Some(format!("external_systems.id={}", ext.id)),
                });
            }
        }
    }

    result
}
