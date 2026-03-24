use crate::model::c4::context::ContextDiagram;
use crate::validator::result::{ValidationError, ValidationResult, Severity};
use std::collections::HashSet;

/// Completeness validation rules
pub fn validate(diagram: &ContextDiagram) -> ValidationResult {
    let mut result = ValidationResult::new();

    // C001: System must have non-empty ID and name
    if diagram.system.id.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "SystemRequiredFields".to_string(),
            message: "System ID cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some("system.id".to_string()),
        });
    }
    if diagram.system.name.is_empty() {
        result.add_error(ValidationError {
            code: "C001".to_string(),
            rule: "SystemRequiredFields".to_string(),
            message: "System name cannot be empty".to_string(),
            severity: Severity::Error,
            location: Some("system.name".to_string()),
        });
    }

    // C002: All actors must have non-empty ID and name
    for (idx, actor) in diagram.actors.iter().enumerate() {
        if actor.id.is_empty() {
            result.add_error(ValidationError {
                code: "C002".to_string(),
                rule: "ActorRequiredFields".to_string(),
                message: format!("Actor at index {} has empty ID", idx),
                severity: Severity::Error,
                location: Some(format!("actors[{}].id", idx)),
            });
        }
        if actor.name.is_empty() {
            result.add_error(ValidationError {
                code: "C002".to_string(),
                rule: "ActorRequiredFields".to_string(),
                message: format!("Actor '{}' has empty name", actor.id),
                severity: Severity::Error,
                location: Some(format!("actors[{}].name", idx)),
            });
        }
    }

    // C003: All external systems must have non-empty ID and name
    for (idx, ext) in diagram.external_systems.iter().enumerate() {
        if ext.id.is_empty() {
            result.add_error(ValidationError {
                code: "C003".to_string(),
                rule: "ExternalSystemRequiredFields".to_string(),
                message: format!("External system at index {} has empty ID", idx),
                severity: Severity::Error,
                location: Some(format!("external_systems[{}].id", idx)),
            });
        }
        if ext.name.is_empty() {
            result.add_error(ValidationError {
                code: "C003".to_string(),
                rule: "ExternalSystemRequiredFields".to_string(),
                message: format!("External system '{}' has empty name", ext.id),
                severity: Severity::Error,
                location: Some(format!("external_systems[{}].name", idx)),
            });
        }
    }

    // C004: All interfaces must have non-empty ID and name
    for (idx, iface) in diagram.interfaces.iter().enumerate() {
        if iface.id.is_empty() {
            result.add_error(ValidationError {
                code: "C004".to_string(),
                rule: "InterfaceRequiredFields".to_string(),
                message: format!("Interface at index {} has empty ID", idx),
                severity: Severity::Error,
                location: Some(format!("interfaces[{}].id", idx)),
            });
        }
        if iface.name.is_empty() {
            result.add_error(ValidationError {
                code: "C004".to_string(),
                rule: "InterfaceRequiredFields".to_string(),
                message: format!("Interface '{}' has empty name", iface.id),
                severity: Severity::Error,
                location: Some(format!("interfaces[{}].name", idx)),
            });
        }
    }

    // C005: All interfaces must have a provider
    let provided_interfaces: HashSet<&str> = diagram
        .interface_providers
        .iter()
        .flat_map(|p| p.interfaces.iter().map(|s| s.as_str()))
        .collect();

    for iface in &diagram.interfaces {
        if !provided_interfaces.contains(iface.id.as_str()) {
            result.add_error(ValidationError {
                code: "C005".to_string(),
                rule: "InterfaceMustHaveProvider".to_string(),
                message: format!("Interface '{}' has no provider defined", iface.id),
                severity: Severity::Warning,
                location: Some(format!("interfaces.id={}", iface.id)),
            });
        }
    }

    // C006: Interface usages must reference existing interfaces and actors
    let all_interface_ids: HashSet<&str> = diagram.interfaces.iter().map(|i| i.id.as_str()).collect();
    let all_actor_ids: HashSet<&str> = diagram.all_element_ids().into_iter().collect();

    for (idx, usage) in diagram.interface_usages.iter().enumerate() {
        // Check actor exists
        if !all_actor_ids.contains(usage.actor.as_str()) {
            result.add_error(ValidationError {
                code: "C006".to_string(),
                rule: "InterfaceUsageActorExists".to_string(),
                message: format!("Interface usage references non-existent actor: {}", usage.actor),
                severity: Severity::Error,
                location: Some(format!("interface_usages[{}].actor", idx)),
            });
        }

        // Check interfaces exist
        for iface_id in &usage.interfaces {
            if !all_interface_ids.contains(iface_id.as_str()) {
                result.add_error(ValidationError {
                    code: "C006".to_string(),
                    rule: "InterfaceUsageInterfaceExists".to_string(),
                    message: format!("Interface usage references non-existent interface: {}", iface_id),
                    severity: Severity::Error,
                    location: Some(format!("interface_usages[{}].interfaces", idx)),
                });
            }
        }
    }

    // C007: Interface providers must reference existing systems and interfaces
    for (idx, provider) in diagram.interface_providers.iter().enumerate() {
        // Check system exists
        if provider.system != diagram.system.id
            && !diagram.external_systems.iter().any(|e| e.id == provider.system)
        {
            result.add_error(ValidationError {
                code: "C007".to_string(),
                rule: "InterfaceProviderSystemExists".to_string(),
                message: format!("Interface provider references non-existent system: {}", provider.system),
                severity: Severity::Error,
                location: Some(format!("interface_providers[{}].system", idx)),
            });
        }

        // Check interfaces exist
        for iface_id in &provider.interfaces {
            if !all_interface_ids.contains(iface_id.as_str()) {
                result.add_error(ValidationError {
                    code: "C007".to_string(),
                    rule: "InterfaceProviderInterfaceExists".to_string(),
                    message: format!("Interface provider references non-existent interface: {}", iface_id),
                    severity: Severity::Error,
                    location: Some(format!("interface_providers[{}].interfaces", idx)),
                });
            }
        }
    }

    result
}
