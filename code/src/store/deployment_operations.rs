use crate::model::deployment::{
    DeploymentModel, DeploymentNode, DeployedService, Environment, NetworkLink, NodeType,
};
use crate::utils::error::{AppError, Result};

/// Operations for modifying DeploymentModel
pub struct DeploymentOperations;

impl DeploymentOperations {
    /// Add a new environment (rejects duplicate ID)
    pub fn add_environment(
        model: &mut DeploymentModel,
        id: &str,
        name: Option<&str>,
        desc: Option<&str>,
    ) -> Result<()> {
        if model.environments.iter().any(|e| e.id == id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "environment: {}",
                id
            )));
        }

        model.environments.push(Environment {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: desc.map(|s| s.to_string()),
        });
        model.touch();
        Ok(())
    }

    /// Add a new node (rejects duplicate ID)
    pub fn add_node(
        model: &mut DeploymentModel,
        id: &str,
        name: Option<&str>,
        desc: Option<&str>,
        node_type: Option<NodeType>,
        environment: Option<&str>,
        technology: Option<&str>,
    ) -> Result<()> {
        if model.nodes.iter().any(|n| n.id == id) {
            return Err(AppError::ElementAlreadyExists(format!("node: {}", id)));
        }

        // Validate environment reference if specified
        if let Some(env_id) = environment {
            if model.find_environment(env_id).is_none() {
                return Err(AppError::ElementNotFound(format!(
                    "environment: {}",
                    env_id
                )));
            }
        }

        model.nodes.push(DeploymentNode {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: desc.map(|s| s.to_string()),
            node_type: node_type.unwrap_or_default(),
            environment: environment.map(|s| s.to_string()),
            technology: technology.map(|s| s.to_string()),
        });
        model.touch();
        Ok(())
    }

    /// Add a new service (rejects duplicate ID, validates target_node)
    pub fn add_service(
        model: &mut DeploymentModel,
        id: &str,
        name: Option<&str>,
        desc: Option<&str>,
        delivery_package: &str,
        target_node: &str,
        replicas: Option<u32>,
        port: Option<u16>,
    ) -> Result<()> {
        if model.services.iter().any(|s| s.id == id) {
            return Err(AppError::ElementAlreadyExists(format!("service: {}", id)));
        }

        // Validate target_node exists
        if model.find_node(target_node).is_none() {
            return Err(AppError::ElementNotFound(format!(
                "target_node: {}",
                target_node
            )));
        }

        model.services.push(DeployedService {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: desc.map(|s| s.to_string()),
            delivery_package: delivery_package.to_string(),
            target_node: target_node.to_string(),
            replicas,
            port,
        });
        model.touch();
        Ok(())
    }

    /// Add a new network link (rejects duplicate ID, validates from/to services)
    pub fn add_network_link(
        model: &mut DeploymentModel,
        id: &str,
        from_service: &str,
        to_service: &str,
        protocol: Option<&str>,
        port: Option<u16>,
        desc: Option<&str>,
    ) -> Result<()> {
        if model.network_links.iter().any(|l| l.id == id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "network_link: {}",
                id
            )));
        }

        // Validate from_service exists
        if model.find_service(from_service).is_none() {
            return Err(AppError::ElementNotFound(format!(
                "from_service: {}",
                from_service
            )));
        }

        // Validate to_service exists
        if model.find_service(to_service).is_none() {
            return Err(AppError::ElementNotFound(format!(
                "to_service: {}",
                to_service
            )));
        }

        model.network_links.push(NetworkLink {
            id: id.to_string(),
            from_service: from_service.to_string(),
            to_service: to_service.to_string(),
            protocol: protocol.map(|s| s.to_string()),
            port,
            description: desc.map(|s| s.to_string()),
        });
        model.touch();
        Ok(())
    }

    /// Remove an environment (blocked if nodes reference it)
    pub fn remove_environment(model: &mut DeploymentModel, id: &str) -> Result<()> {
        let idx = model
            .environments
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("environment: {}", id)))?;

        // Check if any node references this environment
        let referencing_nodes: Vec<&str> = model
            .nodes
            .iter()
            .filter(|n| n.environment.as_deref() == Some(id))
            .map(|n| n.id.as_str())
            .collect();

        if !referencing_nodes.is_empty() {
            return Err(AppError::InvalidOperation(format!(
                "无法删除环境 '{}'，以下节点引用了该环境: {}",
                id,
                referencing_nodes.join(", ")
            )));
        }

        model.environments.remove(idx);
        model.touch();
        Ok(())
    }

    /// Remove a node (blocked if services reference it)
    pub fn remove_node(model: &mut DeploymentModel, id: &str) -> Result<()> {
        let idx = model
            .nodes
            .iter()
            .position(|n| n.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("node: {}", id)))?;

        // Check if any service references this node
        let referencing_services: Vec<&str> = model
            .services
            .iter()
            .filter(|s| s.target_node == id)
            .map(|s| s.id.as_str())
            .collect();

        if !referencing_services.is_empty() {
            return Err(AppError::InvalidOperation(format!(
                "无法删除节点 '{}'，以下服务部署在该节点上: {}",
                id,
                referencing_services.join(", ")
            )));
        }

        model.nodes.remove(idx);
        model.touch();
        Ok(())
    }

    /// Remove a service
    pub fn remove_service(model: &mut DeploymentModel, id: &str) -> Result<()> {
        let idx = model
            .services
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("service: {}", id)))?;

        // Also remove network links referencing this service
        model
            .network_links
            .retain(|l| l.from_service != id && l.to_service != id);

        model.services.remove(idx);
        model.touch();
        Ok(())
    }

    /// Remove a network link
    pub fn remove_network_link(model: &mut DeploymentModel, id: &str) -> Result<()> {
        let idx = model
            .network_links
            .iter()
            .position(|l| l.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("network_link: {}", id)))?;

        model.network_links.remove(idx);
        model.touch();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> DeploymentModel {
        DeploymentModel::new("Test Deployment Model")
    }

    #[test]
    fn test_add_environment() {
        let mut model = create_test_model();
        DeploymentOperations::add_environment(&mut model, "PROD", Some("Production"), None).unwrap();
        assert_eq!(model.environments.len(), 1);
        assert_eq!(model.environments[0].id, "PROD");
    }

    #[test]
    fn test_add_duplicate_environment() {
        let mut model = create_test_model();
        DeploymentOperations::add_environment(&mut model, "PROD", None, None).unwrap();
        let result = DeploymentOperations::add_environment(&mut model, "PROD", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_node() {
        let mut model = create_test_model();
        DeploymentOperations::add_environment(&mut model, "PROD", None, None).unwrap();
        DeploymentOperations::add_node(
            &mut model,
            "K8S",
            Some("K8s Cluster"),
            None,
            Some(NodeType::Kubernetes),
            Some("PROD"),
            Some("k3s"),
        )
        .unwrap();
        assert_eq!(model.nodes.len(), 1);
        assert_eq!(model.nodes[0].environment, Some("PROD".to_string()));
    }

    #[test]
    fn test_add_node_invalid_environment() {
        let mut model = create_test_model();
        let result = DeploymentOperations::add_node(
            &mut model, "K8S", None, None, None, Some("NONEXIST"), None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_add_service() {
        let mut model = create_test_model();
        DeploymentOperations::add_node(&mut model, "WEB", None, None, None, None, None).unwrap();
        DeploymentOperations::add_service(
            &mut model,
            "API_SVC",
            Some("API"),
            None,
            "CORE_IMG",
            "WEB",
            Some(3),
            Some(8080),
        )
        .unwrap();
        assert_eq!(model.services.len(), 1);
        assert_eq!(model.services[0].replicas, Some(3));
    }

    #[test]
    fn test_add_service_invalid_node() {
        let mut model = create_test_model();
        let result = DeploymentOperations::add_service(
            &mut model, "SVC", None, None, "PKG", "NONEXIST", None, None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_add_network_link() {
        let mut model = create_test_model();
        DeploymentOperations::add_node(&mut model, "WEB", None, None, None, None, None).unwrap();
        DeploymentOperations::add_service(&mut model, "SVC_A", None, None, "PKG_A", "WEB", None, None).unwrap();
        DeploymentOperations::add_service(&mut model, "SVC_B", None, None, "PKG_B", "WEB", None, None).unwrap();
        DeploymentOperations::add_network_link(
            &mut model, "LINK1", "SVC_A", "SVC_B", Some("http"), Some(80), None,
        )
        .unwrap();
        assert_eq!(model.network_links.len(), 1);
    }

    #[test]
    fn test_add_network_link_invalid_service() {
        let mut model = create_test_model();
        DeploymentOperations::add_node(&mut model, "WEB", None, None, None, None, None).unwrap();
        DeploymentOperations::add_service(&mut model, "SVC_A", None, None, "PKG_A", "WEB", None, None).unwrap();
        let result = DeploymentOperations::add_network_link(
            &mut model, "LINK1", "SVC_A", "NONEXIST", None, None, None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_environment_with_nodes_blocked() {
        let mut model = create_test_model();
        DeploymentOperations::add_environment(&mut model, "PROD", None, None).unwrap();
        DeploymentOperations::add_node(&mut model, "K8S", None, None, None, Some("PROD"), None).unwrap();
        let result = DeploymentOperations::remove_environment(&mut model, "PROD");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_node_with_services_blocked() {
        let mut model = create_test_model();
        DeploymentOperations::add_node(&mut model, "WEB", None, None, None, None, None).unwrap();
        DeploymentOperations::add_service(&mut model, "SVC", None, None, "PKG", "WEB", None, None).unwrap();
        let result = DeploymentOperations::remove_node(&mut model, "WEB");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_service_cascades_links() {
        let mut model = create_test_model();
        DeploymentOperations::add_node(&mut model, "WEB", None, None, None, None, None).unwrap();
        DeploymentOperations::add_service(&mut model, "SVC_A", None, None, "PKG_A", "WEB", None, None).unwrap();
        DeploymentOperations::add_service(&mut model, "SVC_B", None, None, "PKG_B", "WEB", None, None).unwrap();
        DeploymentOperations::add_network_link(&mut model, "LINK", "SVC_A", "SVC_B", None, None, None).unwrap();

        DeploymentOperations::remove_service(&mut model, "SVC_A").unwrap();
        assert_eq!(model.services.len(), 1);
        assert_eq!(model.network_links.len(), 0); // link cascaded
    }

    #[test]
    fn test_remove_environment() {
        let mut model = create_test_model();
        DeploymentOperations::add_environment(&mut model, "PROD", None, None).unwrap();
        DeploymentOperations::add_environment(&mut model, "DEV", None, None).unwrap();
        DeploymentOperations::remove_environment(&mut model, "PROD").unwrap();
        assert_eq!(model.environments.len(), 1);
        assert_eq!(model.environments[0].id, "DEV");
    }

    #[test]
    fn test_remove_network_link() {
        let mut model = create_test_model();
        DeploymentOperations::add_node(&mut model, "WEB", None, None, None, None, None).unwrap();
        DeploymentOperations::add_service(&mut model, "SVC_A", None, None, "PKG_A", "WEB", None, None).unwrap();
        DeploymentOperations::add_service(&mut model, "SVC_B", None, None, "PKG_B", "WEB", None, None).unwrap();
        DeploymentOperations::add_network_link(&mut model, "LINK", "SVC_A", "SVC_B", None, None, None).unwrap();
        DeploymentOperations::remove_network_link(&mut model, "LINK").unwrap();
        assert!(model.network_links.is_empty());
    }
}
