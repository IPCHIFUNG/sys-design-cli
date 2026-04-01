use crate::model::deployment::DeploymentModel;
use crate::validator::result::{Severity, ValidationError, ValidationResult};
use regex::Regex;
use std::collections::HashSet;

/// Validate a DeploymentModel
pub fn validate(model: &DeploymentModel) -> ValidationResult {
    let mut result = ValidationResult::new();
    let id_pattern = Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap();

    // Track all IDs per type for uniqueness checks
    let mut env_ids = HashSet::new();
    let mut node_ids = HashSet::new();
    let mut service_ids = HashSet::new();
    let mut link_ids = HashSet::new();

    // === Environment validation ===
    for env in &model.environments {
        let location = format!("environments.{}", env.id);

        // DP001: Environment must have non-empty id and name
        if env.id.is_empty() {
            result.add_error(ValidationError {
                code: "DP001".to_string(),
                rule: "EnvironmentRequiredFields".to_string(),
                message: "Environment ID cannot be empty".to_string(),
                severity: Severity::Error,
                location: Some(location.clone()),
            });
        }
        if env.name.is_empty() {
            result.add_error(ValidationError {
                code: "DP001".to_string(),
                rule: "EnvironmentRequiredFields".to_string(),
                message: format!("Environment '{}' name cannot be empty", env.id),
                severity: Severity::Error,
                location: Some(format!("{}.name", location)),
            });
        }

        // DP005: Environment ID must be unique
        if !env.id.is_empty() {
            if !env_ids.insert(env.id.clone()) {
                result.add_error(ValidationError {
                    code: "DP005".to_string(),
                    rule: "EnvironmentIdUniqueness".to_string(),
                    message: format!("Duplicate environment ID: {}", env.id),
                    severity: Severity::Error,
                    location: Some(location.clone()),
                });
            }
        }

        // DP012: ID should follow UPPER_SNAKE_CASE
        if !env.id.is_empty() && !id_pattern.is_match(&env.id) {
            result.add_error(ValidationError {
                code: "DP012".to_string(),
                rule: "IdNaming".to_string(),
                message: format!(
                    "Environment ID '{}' does not follow UPPER_SNAKE_CASE convention",
                    env.id
                ),
                severity: Severity::Warning,
                location: Some(format!("{}.id", location)),
            });
        }
    }

    // === Node validation ===
    for node in &model.nodes {
        let location = format!("nodes.{}", node.id);

        // DP002: Node must have non-empty id and name
        if node.id.is_empty() {
            result.add_error(ValidationError {
                code: "DP002".to_string(),
                rule: "NodeRequiredFields".to_string(),
                message: "Node ID cannot be empty".to_string(),
                severity: Severity::Error,
                location: Some(location.clone()),
            });
        }
        if node.name.is_empty() {
            result.add_error(ValidationError {
                code: "DP002".to_string(),
                rule: "NodeRequiredFields".to_string(),
                message: format!("Node '{}' name cannot be empty", node.id),
                severity: Severity::Error,
                location: Some(format!("{}.name", location)),
            });
        }

        // DP006: Node ID must be unique
        if !node.id.is_empty() {
            if !node_ids.insert(node.id.clone()) {
                result.add_error(ValidationError {
                    code: "DP006".to_string(),
                    rule: "NodeIdUniqueness".to_string(),
                    message: format!("Duplicate node ID: {}", node.id),
                    severity: Severity::Error,
                    location: Some(location.clone()),
                });
            }
        }

        // DP009: Node environment must reference existing environment
        if let Some(ref env_ref) = node.environment {
            if !env_ref.is_empty() && !env_ids.contains(env_ref) {
                result.add_error(ValidationError {
                    code: "DP009".to_string(),
                    rule: "NodeEnvironmentReference".to_string(),
                    message: format!(
                        "Node '{}' references non-existent environment '{}'",
                        node.id, env_ref
                    ),
                    severity: Severity::Error,
                    location: Some(format!("{}.environment", location)),
                });
            }
        }

        // DP012: ID should follow UPPER_SNAKE_CASE
        if !node.id.is_empty() && !id_pattern.is_match(&node.id) {
            result.add_error(ValidationError {
                code: "DP012".to_string(),
                rule: "IdNaming".to_string(),
                message: format!(
                    "Node ID '{}' does not follow UPPER_SNAKE_CASE convention",
                    node.id
                ),
                severity: Severity::Warning,
                location: Some(format!("{}.id", location)),
            });
        }
    }

    // === Service validation ===
    for svc in &model.services {
        let location = format!("services.{}", svc.id);

        // DP003: Service must have non-empty id, name, delivery_package, target_node
        if svc.id.is_empty() {
            result.add_error(ValidationError {
                code: "DP003".to_string(),
                rule: "ServiceRequiredFields".to_string(),
                message: "Service ID cannot be empty".to_string(),
                severity: Severity::Error,
                location: Some(location.clone()),
            });
        }
        if svc.name.is_empty() {
            result.add_error(ValidationError {
                code: "DP003".to_string(),
                rule: "ServiceRequiredFields".to_string(),
                message: format!("Service '{}' name cannot be empty", svc.id),
                severity: Severity::Error,
                location: Some(format!("{}.name", location)),
            });
        }
        if svc.delivery_package.is_empty() {
            result.add_error(ValidationError {
                code: "DP003".to_string(),
                rule: "ServiceRequiredFields".to_string(),
                message: format!("Service '{}' delivery_package cannot be empty", svc.id),
                severity: Severity::Error,
                location: Some(format!("{}.delivery_package", location)),
            });
        }
        if svc.target_node.is_empty() {
            result.add_error(ValidationError {
                code: "DP003".to_string(),
                rule: "ServiceRequiredFields".to_string(),
                message: format!("Service '{}' target_node cannot be empty", svc.id),
                severity: Severity::Error,
                location: Some(format!("{}.target_node", location)),
            });
        }

        // DP007: Service ID must be unique
        if !svc.id.is_empty() {
            if !service_ids.insert(svc.id.clone()) {
                result.add_error(ValidationError {
                    code: "DP007".to_string(),
                    rule: "ServiceIdUniqueness".to_string(),
                    message: format!("Duplicate service ID: {}", svc.id),
                    severity: Severity::Error,
                    location: Some(location.clone()),
                });
            }
        }

        // DP010: Service target_node must reference existing node
        if !svc.target_node.is_empty() && !node_ids.contains(&svc.target_node) {
            result.add_error(ValidationError {
                code: "DP010".to_string(),
                rule: "ServiceTargetNodeReference".to_string(),
                message: format!(
                    "Service '{}' references non-existent target_node '{}'",
                    svc.id, svc.target_node
                ),
                severity: Severity::Error,
                location: Some(format!("{}.target_node", location)),
            });
        }

        // DP012: ID should follow UPPER_SNAKE_CASE
        if !svc.id.is_empty() && !id_pattern.is_match(&svc.id) {
            result.add_error(ValidationError {
                code: "DP012".to_string(),
                rule: "IdNaming".to_string(),
                message: format!(
                    "Service ID '{}' does not follow UPPER_SNAKE_CASE convention",
                    svc.id
                ),
                severity: Severity::Warning,
                location: Some(format!("{}.id", location)),
            });
        }
    }

    // === NetworkLink validation ===
    for link in &model.network_links {
        let location = format!("network_links.{}", link.id);

        // DP004: NetworkLink must have non-empty id, from_service, to_service
        if link.id.is_empty() {
            result.add_error(ValidationError {
                code: "DP004".to_string(),
                rule: "NetworkLinkRequiredFields".to_string(),
                message: "NetworkLink ID cannot be empty".to_string(),
                severity: Severity::Error,
                location: Some(location.clone()),
            });
        }
        if link.from_service.is_empty() {
            result.add_error(ValidationError {
                code: "DP004".to_string(),
                rule: "NetworkLinkRequiredFields".to_string(),
                message: format!("NetworkLink '{}' from_service cannot be empty", link.id),
                severity: Severity::Error,
                location: Some(format!("{}.from_service", location)),
            });
        }
        if link.to_service.is_empty() {
            result.add_error(ValidationError {
                code: "DP004".to_string(),
                rule: "NetworkLinkRequiredFields".to_string(),
                message: format!("NetworkLink '{}' to_service cannot be empty", link.id),
                severity: Severity::Error,
                location: Some(format!("{}.to_service", location)),
            });
        }

        // DP008: NetworkLink ID must be unique
        if !link.id.is_empty() {
            if !link_ids.insert(link.id.clone()) {
                result.add_error(ValidationError {
                    code: "DP008".to_string(),
                    rule: "NetworkLinkIdUniqueness".to_string(),
                    message: format!("Duplicate network_link ID: {}", link.id),
                    severity: Severity::Error,
                    location: Some(location.clone()),
                });
            }
        }

        // DP011: NetworkLink from/to must reference existing services
        if !link.from_service.is_empty() && !service_ids.contains(&link.from_service) {
            result.add_error(ValidationError {
                code: "DP011".to_string(),
                rule: "NetworkLinkServiceReference".to_string(),
                message: format!(
                    "NetworkLink '{}' references non-existent from_service '{}'",
                    link.id, link.from_service
                ),
                severity: Severity::Error,
                location: Some(format!("{}.from_service", location)),
            });
        }
        if !link.to_service.is_empty() && !service_ids.contains(&link.to_service) {
            result.add_error(ValidationError {
                code: "DP011".to_string(),
                rule: "NetworkLinkServiceReference".to_string(),
                message: format!(
                    "NetworkLink '{}' references non-existent to_service '{}'",
                    link.id, link.to_service
                ),
                severity: Severity::Error,
                location: Some(format!("{}.to_service", location)),
            });
        }

        // DP012: ID should follow UPPER_SNAKE_CASE
        if !link.id.is_empty() && !id_pattern.is_match(&link.id) {
            result.add_error(ValidationError {
                code: "DP012".to_string(),
                rule: "IdNaming".to_string(),
                message: format!(
                    "NetworkLink ID '{}' does not follow UPPER_SNAKE_CASE convention",
                    link.id
                ),
                severity: Severity::Warning,
                location: Some(format!("{}.id", location)),
            });
        }
    }

    // DP013: Deployment model should have at least 1 node
    if model.nodes.is_empty() {
        result.add_error(ValidationError {
            code: "DP013".to_string(),
            rule: "MinNodes".to_string(),
            message: "Deployment model should have at least 1 node".to_string(),
            severity: Severity::Warning,
            location: None,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::deployment::*;

    fn create_test_model() -> DeploymentModel {
        let mut model = DeploymentModel::new("Test");
        model.environments.push(Environment {
            id: "PROD".to_string(),
            name: "Production".to_string(),
            description: None,
        });
        model.nodes.push(DeploymentNode {
            id: "K8S".to_string(),
            name: "K8s Cluster".to_string(),
            description: None,
            node_type: NodeType::Kubernetes,
            environment: Some("PROD".to_string()),
            technology: None,
        });
        model.services.push(DeployedService {
            id: "API_SVC".to_string(),
            name: "API".to_string(),
            description: None,
            delivery_package: "CORE_IMG".to_string(),
            target_node: "K8S".to_string(),
            replicas: Some(3),
            port: Some(8080),
        });
        model
    }

    #[test]
    fn test_valid_deployment_model() {
        let model = create_test_model();
        let result = validate(&model);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_dp001_empty_env_id() {
        let mut model = create_test_model();
        model.environments.push(Environment {
            id: "".to_string(),
            name: "No ID".to_string(),
            description: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP001" && e.severity == Severity::Error));
    }

    #[test]
    fn test_dp001_empty_env_name() {
        let mut model = create_test_model();
        model.environments.push(Environment {
            id: "DEV".to_string(),
            name: "".to_string(),
            description: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP001" && e.severity == Severity::Error));
    }

    #[test]
    fn test_dp002_empty_node_id() {
        let mut model = create_test_model();
        model.nodes.push(DeploymentNode {
            id: "".to_string(),
            name: "No ID".to_string(),
            description: None,
            node_type: NodeType::Server,
            environment: None,
            technology: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP002" && e.severity == Severity::Error));
    }

    #[test]
    fn test_dp002_empty_node_name() {
        let mut model = create_test_model();
        model.nodes.push(DeploymentNode {
            id: "SOME_NODE".to_string(),
            name: "".to_string(),
            description: None,
            node_type: NodeType::Server,
            environment: None,
            technology: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP002" && e.severity == Severity::Error));
    }

    #[test]
    fn test_dp003_empty_service_fields() {
        let mut model = create_test_model();
        model.services.push(DeployedService {
            id: "".to_string(),
            name: "".to_string(),
            description: None,
            delivery_package: "".to_string(),
            target_node: "".to_string(),
            replicas: None,
            port: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP003" && e.severity == Severity::Error));
    }

    #[test]
    fn test_dp004_empty_link_fields() {
        let mut model = create_test_model();
        model.network_links.push(NetworkLink {
            id: "".to_string(),
            from_service: "".to_string(),
            to_service: "".to_string(),
            protocol: None,
            port: None,
            description: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP004" && e.severity == Severity::Error));
    }

    #[test]
    fn test_dp005_duplicate_env_id() {
        let mut model = create_test_model();
        model.environments.push(Environment {
            id: "PROD".to_string(),
            name: "Another Prod".to_string(),
            description: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP005"));
    }

    #[test]
    fn test_dp006_duplicate_node_id() {
        let mut model = create_test_model();
        model.nodes.push(DeploymentNode {
            id: "K8S".to_string(),
            name: "Another K8s".to_string(),
            description: None,
            node_type: NodeType::Kubernetes,
            environment: None,
            technology: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP006"));
    }

    #[test]
    fn test_dp007_duplicate_service_id() {
        let mut model = create_test_model();
        model.services.push(DeployedService {
            id: "API_SVC".to_string(),
            name: "Another API".to_string(),
            description: None,
            delivery_package: "OTHER_IMG".to_string(),
            target_node: "K8S".to_string(),
            replicas: None,
            port: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP007"));
    }

    #[test]
    fn test_dp008_duplicate_link_id() {
        let mut model = create_test_model();
        model.services.push(DeployedService {
            id: "DB_SVC".to_string(),
            name: "DB".to_string(),
            description: None,
            delivery_package: "DB_IMG".to_string(),
            target_node: "K8S".to_string(),
            replicas: None,
            port: None,
        });
        model.network_links.push(NetworkLink {
            id: "LINK_A".to_string(),
            from_service: "API_SVC".to_string(),
            to_service: "DB_SVC".to_string(),
            protocol: None,
            port: None,
            description: None,
        });
        model.network_links.push(NetworkLink {
            id: "LINK_A".to_string(),
            from_service: "DB_SVC".to_string(),
            to_service: "API_SVC".to_string(),
            protocol: None,
            port: None,
            description: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP008"));
    }

    #[test]
    fn test_dp009_node_bad_env_reference() {
        let mut model = DeploymentModel::new("Test");
        model.nodes.push(DeploymentNode {
            id: "ORPHAN_NODE".to_string(),
            name: "Orphan".to_string(),
            description: None,
            node_type: NodeType::Server,
            environment: Some("NONEXISTENT".to_string()),
            technology: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP009"));
    }

    #[test]
    fn test_dp010_service_bad_node_reference() {
        let mut model = DeploymentModel::new("Test");
        model.nodes.push(DeploymentNode {
            id: "REAL_NODE".to_string(),
            name: "Real".to_string(),
            description: None,
            node_type: NodeType::Server,
            environment: None,
            technology: None,
        });
        model.services.push(DeployedService {
            id: "SVC_A".to_string(),
            name: "Svc A".to_string(),
            description: None,
            delivery_package: "PKG".to_string(),
            target_node: "NONEXISTENT_NODE".to_string(),
            replicas: None,
            port: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP010"));
    }

    #[test]
    fn test_dp011_link_bad_service_reference() {
        let mut model = create_test_model();
        model.network_links.push(NetworkLink {
            id: "BAD_LINK".to_string(),
            from_service: "NONEXISTENT_SVC".to_string(),
            to_service: "API_SVC".to_string(),
            protocol: None,
            port: None,
            description: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DP011"));
    }

    #[test]
    fn test_dp012_naming() {
        let mut model = DeploymentModel::new("Test");
        model.environments.push(Environment {
            id: "bad-env".to_string(),
            name: "Bad Env".to_string(),
            description: None,
        });
        model.nodes.push(DeploymentNode {
            id: "bad-node".to_string(),
            name: "Bad Node".to_string(),
            description: None,
            node_type: NodeType::Server,
            environment: None,
            technology: None,
        });
        model.services.push(DeployedService {
            id: "bad-svc".to_string(),
            name: "Bad Svc".to_string(),
            description: None,
            delivery_package: "PKG".to_string(),
            target_node: "bad-node".to_string(),
            replicas: None,
            port: None,
        });
        model.network_links.push(NetworkLink {
            id: "bad-link".to_string(),
            from_service: "bad-svc".to_string(),
            to_service: "bad-svc".to_string(),
            protocol: None,
            port: None,
            description: None,
        });
        let result = validate(&model);
        assert!(result.is_valid); // Warnings only
        assert!(result.errors.iter().any(|e| e.code == "DP012" && e.severity == Severity::Warning));
        // Should have DP012 warnings for all 4 element types
        assert_eq!(result.errors.iter().filter(|e| e.code == "DP012").count(), 4);
    }

    #[test]
    fn test_dp013_no_nodes() {
        let model = DeploymentModel::new("Test");
        let result = validate(&model);
        assert!(result.is_valid); // Warning only
        assert!(result.errors.iter().any(|e| e.code == "DP013" && e.severity == Severity::Warning));
    }
}
