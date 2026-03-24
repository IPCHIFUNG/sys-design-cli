use crate::model::c4::context::ContextDiagram;
use crate::validator::result::{ValidationError, ValidationResult, Severity};
use std::collections::HashSet;

/// Consistency validation rules
pub fn validate(diagram: &ContextDiagram) -> ValidationResult {
    let mut result = ValidationResult::new();

    // S001: All element IDs must be unique
    check_id_uniqueness(diagram, &mut result);

    // S002: Check for orphan interfaces (defined but not used)
    check_orphan_interfaces(diagram, &mut result);

    // S003: Check for orphan actors (defined but have no relationships)
    check_orphan_actors(diagram, &mut result);

    result
}

fn check_id_uniqueness(diagram: &ContextDiagram, result: &mut ValidationResult) {
    let mut seen_ids: HashSet<&str> = HashSet::new();
    let mut duplicates: Vec<(&str, &str)> = Vec::new();

    // Check system
    if !seen_ids.insert(&diagram.system.id) {
        duplicates.push((&diagram.system.id, "system"));
    }

    // Check actors
    for actor in &diagram.actors {
        if !seen_ids.insert(&actor.id) {
            duplicates.push((&actor.id, "actor"));
        }
    }

    // Check external systems
    for ext in &diagram.external_systems {
        if !seen_ids.insert(&ext.id) {
            duplicates.push((&ext.id, "external_system"));
        }
    }

    // Check interfaces
    for iface in &diagram.interfaces {
        if !seen_ids.insert(&iface.id) {
            duplicates.push((&iface.id, "interface"));
        }
    }

    for (id, element_type) in duplicates {
        result.add_error(ValidationError {
            code: "S001".to_string(),
            rule: "IdUniqueness".to_string(),
            message: format!("Duplicate ID '{}' found in {}", id, element_type),
            severity: Severity::Error,
            location: Some(format!("elements.id={}", id)),
        });
    }
}

fn check_orphan_interfaces(diagram: &ContextDiagram, result: &mut ValidationResult) {
    // Collect all used interfaces
    let used_interfaces: HashSet<&str> = diagram
        .interface_usages
        .iter()
        .flat_map(|u| u.interfaces.iter().map(|s| s.as_str()))
        .collect();

    // Check each interface
    for iface in &diagram.interfaces {
        if !used_interfaces.contains(iface.id.as_str()) {
            result.add_error(ValidationError {
                code: "S002".to_string(),
                rule: "OrphanInterface".to_string(),
                message: format!("Interface '{}' is defined but not used by any actor", iface.id),
                severity: Severity::Warning,
                location: Some(format!("interfaces.id={}", iface.id)),
            });
        }
    }
}

fn check_orphan_actors(diagram: &ContextDiagram, result: &mut ValidationResult) {
    // Collect all actors with interface usages
    let actors_with_usages: HashSet<&str> = diagram
        .interface_usages
        .iter()
        .map(|u| u.actor.as_str())
        .collect();

    // Check each actor
    for actor in &diagram.actors {
        if !actors_with_usages.contains(actor.id.as_str()) {
            result.add_error(ValidationError {
                code: "S003".to_string(),
                rule: "OrphanActor".to_string(),
                message: format!("Actor '{}' has no interface usage defined", actor.id),
                severity: Severity::Warning,
                location: Some(format!("actors.id={}", actor.id)),
            });
        }
    }

    // Also check external systems
    for ext in &diagram.external_systems {
        // Check if it's a provider or a user
        let is_provider = diagram
            .interface_providers
            .iter()
            .any(|p| p.system == ext.id);
        let is_user = actors_with_usages.contains(ext.id.as_str());

        if !is_provider && !is_user {
            result.add_error(ValidationError {
                code: "S003".to_string(),
                rule: "OrphanExternalSystem".to_string(),
                message: format!(
                    "External system '{}' neither provides nor uses any interfaces",
                    ext.id
                ),
                severity: Severity::Warning,
                location: Some(format!("external_systems.id={}", ext.id)),
            });
        }
    }
}
