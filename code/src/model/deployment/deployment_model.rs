use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Root structure for Deployment Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentModel {
    pub version: String,
    pub kind: DeploymentDiagramKind,
    pub metadata: DeploymentMetadata,
    #[serde(default)]
    pub environments: Vec<Environment>,
    #[serde(default)]
    pub nodes: Vec<DeploymentNode>,
    #[serde(default)]
    pub services: Vec<DeployedService>,
    #[serde(default)]
    pub network_links: Vec<NetworkLink>,
}

impl DeploymentModel {
    pub fn new(title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            kind: DeploymentDiagramKind::DeploymentModel,
            metadata: DeploymentMetadata {
                title: title.to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            environments: Vec::new(),
            nodes: Vec::new(),
            services: Vec::new(),
            network_links: Vec::new(),
        }
    }

    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    pub fn all_environment_ids(&self) -> Vec<&str> {
        self.environments.iter().map(|e| e.id.as_str()).collect()
    }

    pub fn all_node_ids(&self) -> Vec<&str> {
        self.nodes.iter().map(|n| n.id.as_str()).collect()
    }

    pub fn all_service_ids(&self) -> Vec<&str> {
        self.services.iter().map(|s| s.id.as_str()).collect()
    }

    pub fn find_environment(&self, id: &str) -> Option<&Environment> {
        self.environments.iter().find(|e| e.id == id)
    }

    pub fn find_node(&self, id: &str) -> Option<&DeploymentNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn find_node_mut(&mut self, id: &str) -> Option<&mut DeploymentNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn find_service(&self, id: &str) -> Option<&DeployedService> {
        self.services.iter().find(|s| s.id == id)
    }

    pub fn find_service_mut(&mut self, id: &str) -> Option<&mut DeployedService> {
        self.services.iter_mut().find(|s| s.id == id)
    }

    pub fn find_network_link(&self, id: &str) -> Option<&NetworkLink> {
        self.network_links.iter().find(|l| l.id == id)
    }

    /// Cross-type lookup: search in order env -> node -> service
    pub fn get_element_name(&self, id: &str) -> Option<&str> {
        self.find_environment(id)
            .map(|e| e.name.as_str())
            .or_else(|| self.find_node(id).map(|n| n.name.as_str()))
            .or_else(|| self.find_service(id).map(|s| s.name.as_str()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentDiagramKind {
    DeploymentModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentNode {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub node_type: NodeType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    #[default]
    Server,
    Vm,
    Container,
    Kubernetes,
    Serverless,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedService {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub delivery_package: String,
    pub target_node: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replicas: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLink {
    pub id: String,
    pub from_service: String,
    pub to_service: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_deployment_model() {
        let model = DeploymentModel::new("Test Deployment");
        assert_eq!(model.version, "1.0");
        assert_eq!(model.kind, DeploymentDiagramKind::DeploymentModel);
        assert!(model.environments.is_empty());
        assert!(model.nodes.is_empty());
        assert!(model.services.is_empty());
        assert!(model.network_links.is_empty());
    }

    #[test]
    fn test_all_environment_ids() {
        let mut model = DeploymentModel::new("Test");
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
        assert_eq!(model.all_environment_ids(), vec!["PROD", "STAGING"]);
    }

    #[test]
    fn test_all_node_ids() {
        let mut model = DeploymentModel::new("Test");
        model.nodes.push(DeploymentNode {
            id: "WEB_SRV".to_string(),
            name: "Web Server".to_string(),
            description: None,
            node_type: NodeType::Server,
            environment: None,
            technology: None,
        });
        assert_eq!(model.all_node_ids(), vec!["WEB_SRV"]);
    }

    #[test]
    fn test_all_service_ids() {
        let mut model = DeploymentModel::new("Test");
        model.services.push(DeployedService {
            id: "API_SVC".to_string(),
            name: "API Service".to_string(),
            description: None,
            delivery_package: "CORE_IMG".to_string(),
            target_node: "WEB_SRV".to_string(),
            replicas: Some(3),
            port: Some(8080),
        });
        assert_eq!(model.all_service_ids(), vec!["API_SVC"]);
    }

    #[test]
    fn test_find_environment() {
        let mut model = DeploymentModel::new("Test");
        model.environments.push(Environment {
            id: "PROD".to_string(),
            name: "Production".to_string(),
            description: None,
        });
        assert!(model.find_environment("PROD").is_some());
        assert!(model.find_environment("DEV").is_none());
    }

    #[test]
    fn test_find_node() {
        let mut model = DeploymentModel::new("Test");
        model.nodes.push(DeploymentNode {
            id: "WEB_SRV".to_string(),
            name: "Web Server".to_string(),
            description: None,
            node_type: NodeType::Server,
            environment: None,
            technology: None,
        });
        assert!(model.find_node("WEB_SRV").is_some());
        assert!(model.find_node("DB_SRV").is_none());
        assert!(model.find_node_mut("WEB_SRV").is_some());
    }

    #[test]
    fn test_find_service() {
        let mut model = DeploymentModel::new("Test");
        model.services.push(DeployedService {
            id: "API_SVC".to_string(),
            name: "API".to_string(),
            description: None,
            delivery_package: "IMG".to_string(),
            target_node: "NODE".to_string(),
            replicas: None,
            port: None,
        });
        assert!(model.find_service("API_SVC").is_some());
        assert!(model.find_service("OTHER").is_none());
    }

    #[test]
    fn test_get_element_name_cross_type() {
        let mut model = DeploymentModel::new("Test");
        model.environments.push(Environment {
            id: "PROD".to_string(),
            name: "Production".to_string(),
            description: None,
        });
        model.nodes.push(DeploymentNode {
            id: "WEB".to_string(),
            name: "Web Node".to_string(),
            description: None,
            node_type: NodeType::Server,
            environment: None,
            technology: None,
        });
        assert_eq!(model.get_element_name("PROD"), Some("Production"));
        assert_eq!(model.get_element_name("WEB"), Some("Web Node"));
        assert_eq!(model.get_element_name("MISSING"), None);
    }

    #[test]
    fn test_node_type_default() {
        let node = DeploymentNode {
            id: "N".to_string(),
            name: "N".to_string(),
            description: None,
            node_type: NodeType::default(),
            environment: None,
            technology: None,
        };
        assert_eq!(node.node_type, NodeType::Server);
    }

    #[test]
    fn test_yaml_round_trip() {
        let mut model = DeploymentModel::new("Test Deployment");
        model.environments.push(Environment {
            id: "PROD".to_string(),
            name: "Production".to_string(),
            description: Some("Production environment".to_string()),
        });
        model.nodes.push(DeploymentNode {
            id: "K8S_CLUSTER".to_string(),
            name: "K8s Cluster".to_string(),
            description: None,
            node_type: NodeType::Kubernetes,
            environment: Some("PROD".to_string()),
            technology: Some("k3s".to_string()),
        });
        model.services.push(DeployedService {
            id: "API_SVC".to_string(),
            name: "API Service".to_string(),
            description: None,
            delivery_package: "CORE_IMG".to_string(),
            target_node: "K8S_CLUSTER".to_string(),
            replicas: Some(3),
            port: Some(8080),
        });
        model.network_links.push(NetworkLink {
            id: "API_TO_DB".to_string(),
            from_service: "API_SVC".to_string(),
            to_service: "DB_SVC".to_string(),
            protocol: Some("tcp".to_string()),
            port: Some(5432),
            description: None,
        });

        let yaml = serde_yaml::to_string(&model).unwrap();
        let parsed: DeploymentModel = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.environments.len(), 1);
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.services.len(), 1);
        assert_eq!(parsed.network_links.len(), 1);
        assert_eq!(parsed.nodes[0].node_type, NodeType::Kubernetes);
        assert_eq!(parsed.services[0].replicas, Some(3));
        assert_eq!(parsed.network_links[0].port, Some(5432));
    }
}
