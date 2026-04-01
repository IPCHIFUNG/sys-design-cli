use crate::model::deployment::*;
use crate::model::workspace::Workspace;

/// Generate PlantUML deployment diagram
pub fn generate_deployment_model_plantuml(
    workspace: &Workspace,
    model: &DeploymentModel,
    environment_id: Option<&str>,
) -> Result<String, String> {
    // Determine which environment to render
    let selected_env_id: Option<&str> = match environment_id {
        Some(id) => Some(id),
        None => {
            match model.environments.len() {
                0 => None,
                1 => Some(&model.environments[0].id),
                _ => {
                    return Err(
                        "Multiple environments exist. Please specify environment_id.".to_string()
                    );
                }
            }
        }
    };

    // Filter nodes by selected environment
    let filtered_nodes: Vec<&DeploymentNode> = if let Some(env_id) = selected_env_id {
        model.nodes.iter().filter(|n| n.environment.as_deref() == Some(env_id)).collect()
    } else {
        model.nodes.iter().collect()
    };

    // Build set of filtered node IDs
    let filtered_node_ids: std::collections::HashSet<&str> =
        filtered_nodes.iter().map(|n| n.id.as_str()).collect();

    // Filter services whose target_node is in the filtered nodes
    let filtered_services: Vec<&DeployedService> = model
        .services
        .iter()
        .filter(|s| filtered_node_ids.contains(s.target_node.as_str()))
        .collect();

    // Build set of filtered service IDs
    let filtered_service_ids: std::collections::HashSet<&str> =
        filtered_services.iter().map(|s| s.id.as_str()).collect();

    // Filter network_links where both from_service and to_service are in filtered services
    let filtered_links: Vec<&NetworkLink> = model
        .network_links
        .iter()
        .filter(|l| {
            filtered_service_ids.contains(l.from_service.as_str())
                && filtered_service_ids.contains(l.to_service.as_str())
        })
        .collect();

    let mut output = String::new();
    output.push_str("@startuml\n\n");

    // Get environment name for the rectangle label
    let env_name = if let Some(env_id) = selected_env_id {
        model.find_environment(env_id).map(|e| e.name.as_str()).unwrap_or(env_id)
    } else {
        "Deployment"
    };

    output.push_str(&format!("rectangle \"{}\" {{\n", env_name));

    // Group services by target node
    for node in &filtered_nodes {
        let node_type_str = format!("{:?}", node.node_type);
        output.push_str(&format!(
            "  node \"{} <<{}>>\" as {} {{\n",
            node.name, node_type_str, node.id
        ));

        let node_services: Vec<&&DeployedService> =
            filtered_services.iter().filter(|s| s.target_node == node.id).collect();

        for svc in node_services {
            output.push_str(&format!("    component \"{}\" as {}\n", svc.name, svc.id));

            // Build note with delivery_package, replicas, port
            let mut note_lines: Vec<String> = Vec::new();

            // Resolve delivery_package name from workspace delivery_model
            let pkg_display = resolve_delivery_package_name(workspace, &svc.delivery_package)
                .map(|name| format!("{} ({})", svc.delivery_package, name))
                .unwrap_or_else(|| svc.delivery_package.clone());
            note_lines.push(format!("delivery_package: {}", pkg_display));

            if let Some(replicas) = svc.replicas {
                note_lines.push(format!("replicas: {}", replicas));
            }
            if let Some(port) = svc.port {
                note_lines.push(format!("port: {}", port));
            }

            output.push_str(&format!(
                "    note right of {}\n      {}\n    end note\n",
                svc.id,
                note_lines.join("\\n")
            ));
        }

        output.push_str("  }\n\n");
    }

    output.push_str("}\n\n");

    // Render network links
    for link in &filtered_links {
        let label = match (&link.protocol, &link.port) {
            (Some(proto), Some(port)) => format!("{}:{}", proto, port),
            (Some(proto), None) => proto.clone(),
            (None, Some(port)) => format!(":{}", port),
            (None, None) => String::new(),
        };
        output.push_str(&format!("{} --> {} : {}\n", link.from_service, link.to_service, label));
    }

    output.push_str("\n@enduml\n");
    Ok(output)
}

/// Resolve delivery package name from workspace delivery_model
fn resolve_delivery_package_name<'a>(workspace: &'a Workspace, package_id: &str) -> Option<&'a str> {
    if let Some(delivery_model) = &workspace.delivery_model {
        return delivery_model.get_package_name(package_id);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::delivery::*;

    fn create_test_workspace() -> Workspace {
        let mut ws = Workspace::new("Test");
        let mut delivery = DeliveryModel::new("Test Delivery");
        delivery.packages.push(DeliveryPackage {
            id: "CORE_IMG".to_string(),
            name: "Core Image".to_string(),
            description: None,
            version: Some("1.0.0".to_string()),
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        ws.delivery_model = Some(delivery);
        ws
    }

    fn create_test_deployment_model() -> DeploymentModel {
        let mut model = DeploymentModel::new("Test Deployment");
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
    fn test_generate_deployment_model() {
        let ws = create_test_workspace();
        let model = create_test_deployment_model();
        let result = generate_deployment_model_plantuml(&ws, &model, None).unwrap();

        assert!(result.contains("@startuml"));
        assert!(result.contains("@enduml"));
        assert!(result.contains("node \"K8s Cluster <<Kubernetes>>\" as K8S"));
        assert!(result.contains("component \"API\" as API_SVC"));
        assert!(result.contains("delivery_package:"));
        assert!(result.contains("replicas: 3"));
        assert!(result.contains("port: 8080"));
    }

    #[test]
    fn test_generate_deployment_model_with_env_filter() {
        let ws = create_test_workspace();
        let mut model = DeploymentModel::new("Test Deployment");
        model.environments.push(Environment {
            id: "PROD".to_string(),
            name: "Production".to_string(),
            description: None,
        });
        model.environments.push(Environment {
            id: "STAGING".to_string(),
            name: "Staging".to_string(),
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
        model.nodes.push(DeploymentNode {
            id: "STAGING_SRV".to_string(),
            name: "Staging Server".to_string(),
            description: None,
            node_type: NodeType::Server,
            environment: Some("STAGING".to_string()),
            technology: None,
        });
        model.services.push(DeployedService {
            id: "API_SVC".to_string(),
            name: "API".to_string(),
            description: None,
            delivery_package: "CORE_IMG".to_string(),
            target_node: "K8S".to_string(),
            replicas: None,
            port: None,
        });
        model.services.push(DeployedService {
            id: "STAGING_SVC".to_string(),
            name: "Staging API".to_string(),
            description: None,
            delivery_package: "CORE_IMG".to_string(),
            target_node: "STAGING_SRV".to_string(),
            replicas: None,
            port: None,
        });

        // Filter to PROD only
        let result = generate_deployment_model_plantuml(&ws, &model, Some("PROD")).unwrap();
        assert!(result.contains("K8s Cluster"));
        assert!(!result.contains("Staging Server"));
        assert!(result.contains("component \"API\""));
        assert!(!result.contains("Staging API"));
    }

    #[test]
    fn test_generate_deployment_auto_env() {
        let ws = create_test_workspace();
        let model = create_test_deployment_model();
        // Single env: auto-selected when environment_id is None
        let result = generate_deployment_model_plantuml(&ws, &model, None).unwrap();
        assert!(result.contains("Production"));
        assert!(result.contains("component \"API\""));
    }

    #[test]
    fn test_generate_deployment_multi_env_error() {
        let ws = create_test_workspace();
        let mut model = DeploymentModel::new("Test Deployment");
        model.environments.push(Environment {
            id: "PROD".to_string(),
            name: "Production".to_string(),
            description: None,
        });
        model.environments.push(Environment {
            id: "STAGING".to_string(),
            name: "Staging".to_string(),
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

        // Multiple envs without specifying should return error
        let result = generate_deployment_model_plantuml(&ws, &model, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Multiple environments exist"));
    }
}
