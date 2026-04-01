use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Root structure for Delivery Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryModel {
    pub version: String,
    pub kind: DeliveryDiagramKind,
    pub metadata: DeliveryMetadata,
    #[serde(default)]
    pub packages: Vec<DeliveryPackage>,
}

impl DeliveryModel {
    pub fn new(title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            kind: DeliveryDiagramKind::DeliveryModel,
            metadata: DeliveryMetadata {
                title: title.to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            packages: Vec::new(),
        }
    }

    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    pub fn all_package_ids(&self) -> Vec<&str> {
        self.packages.iter().map(|p| p.id.as_str()).collect()
    }

    pub fn find_package(&self, id: &str) -> Option<&DeliveryPackage> {
        self.packages.iter().find(|p| p.id == id)
    }

    pub fn find_package_mut(&mut self, id: &str) -> Option<&mut DeliveryPackage> {
        self.packages.iter_mut().find(|p| p.id == id)
    }

    pub fn get_package_name(&self, id: &str) -> Option<&str> {
        self.find_package(id).map(|p| p.name.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryDiagramKind {
    DeliveryModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryMetadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryPackage {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub delivery_type: DeliveryType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryType {
    #[default]
    ContainerImage,
    Archive,
    Installer,
    HelmChart,
    NpmPackage,
    Crate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_delivery_model() {
        let model = DeliveryModel::new("Test Delivery");
        assert_eq!(model.version, "1.0");
        assert_eq!(model.kind, DeliveryDiagramKind::DeliveryModel);
        assert!(model.packages.is_empty());
    }

    #[test]
    fn test_all_package_ids() {
        let mut model = DeliveryModel::new("Test");
        model.packages.push(DeliveryPackage {
            id: "PKG_A".to_string(),
            name: "Package A".to_string(),
            description: None,
            version: None,
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        assert_eq!(model.all_package_ids(), vec!["PKG_A"]);
    }

    #[test]
    fn test_find_package() {
        let mut model = DeliveryModel::new("Test");
        model.packages.push(DeliveryPackage {
            id: "PKG_A".to_string(),
            name: "Package A".to_string(),
            description: None,
            version: None,
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        assert!(model.find_package("PKG_A").is_some());
        assert!(model.find_package("PKG_B").is_none());
        assert!(model.find_package_mut("PKG_A").is_some());
    }

    #[test]
    fn test_delivery_type_default() {
        let package = DeliveryPackage {
            id: "P".to_string(),
            name: "P".to_string(),
            description: None,
            version: None,
            delivery_type: DeliveryType::default(),
            artifacts: vec![],
            registry: None,
        };
        assert_eq!(package.delivery_type, DeliveryType::ContainerImage);
    }

    #[test]
    fn test_yaml_round_trip() {
        let mut model = DeliveryModel::new("Test Delivery");
        model.packages.push(DeliveryPackage {
            id: "CORE_IMG".to_string(),
            name: "Core Image".to_string(),
            description: Some("Main container image".to_string()),
            version: Some("1.0.0".to_string()),
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec!["CORE_BIN".to_string()],
            registry: Some("registry.example.com/core".to_string()),
        });
        model.packages.push(DeliveryPackage {
            id: "UTIL_CRATE".to_string(),
            name: "Utility Crate".to_string(),
            description: None,
            version: Some("0.1.0".to_string()),
            delivery_type: DeliveryType::Crate,
            artifacts: vec![],
            registry: Some("crates.io".to_string()),
        });

        let yaml = serde_yaml::to_string(&model).unwrap();
        let parsed: DeliveryModel = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.packages.len(), 2);
        assert_eq!(parsed.packages[0].id, "CORE_IMG");
        assert_eq!(parsed.packages[0].artifacts, vec!["CORE_BIN"]);
        assert_eq!(parsed.packages[1].delivery_type, DeliveryType::Crate);
    }
}
