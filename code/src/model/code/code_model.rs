use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Root structure for Code Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeModel {
    pub version: String,
    pub kind: CodeDiagramKind,
    pub metadata: CodeMetadata,
    #[serde(default)]
    pub packages: Vec<CodePackage>,
    #[serde(default)]
    pub dependencies: Vec<PackageDependency>,
}

impl CodeModel {
    /// Create a new empty CodeModel
    pub fn new(title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            kind: CodeDiagramKind::CodeModel,
            metadata: CodeMetadata {
                title: title.to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            packages: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    /// Get all package IDs
    pub fn all_package_ids(&self) -> Vec<&str> {
        self.packages.iter().map(|p| p.id.as_str()).collect()
    }

    /// Find package by ID
    pub fn find_package(&self, id: &str) -> Option<&CodePackage> {
        self.packages.iter().find(|p| p.id == id)
    }

    /// Find package by ID (mutable)
    pub fn find_package_mut(&mut self, id: &str) -> Option<&mut CodePackage> {
        self.packages.iter_mut().find(|p| p.id == id)
    }

    /// Get package name by ID
    pub fn get_package_name(&self, id: &str) -> Option<&str> {
        self.find_package(id).map(|p| p.name.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CodeDiagramKind {
    CodeModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

/// A code package (module, library, service, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePackage {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
}

/// Programming language enum with Custom support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Rust,
    Java,
    Python,
    Go,
    TypeScript,
    Cpp,
    CSharp,
    JavaScript,
    Kotlin,
    Swift,
    Custom(String),
}

/// Package-to-package dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageDependency {
    pub from: String,
    pub to: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_code_model() {
        let model = CodeModel::new("Test Code Model");
        assert_eq!(model.version, "1.0");
        assert_eq!(model.kind, CodeDiagramKind::CodeModel);
        assert_eq!(model.metadata.title, "Test Code Model");
        assert!(model.packages.is_empty());
        assert!(model.dependencies.is_empty());
    }

    #[test]
    fn test_all_package_ids() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG_A".to_string(),
            name: "Package A".to_string(),
            description: None,
            language: Some(Language::Rust),
            framework: None,
            path: None,
            element_id: None,
        });
        model.packages.push(CodePackage {
            id: "PKG_B".to_string(),
            name: "Package B".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        assert_eq!(model.all_package_ids(), vec!["PKG_A", "PKG_B"]);
    }

    #[test]
    fn test_find_package() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG_A".to_string(),
            name: "Package A".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });

        assert!(model.find_package("PKG_A").is_some());
        assert!(model.find_package("PKG_B").is_none());
    }

    #[test]
    fn test_get_package_name() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG_A".to_string(),
            name: "Package A".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });

        assert_eq!(model.get_package_name("PKG_A"), Some("Package A"));
        assert_eq!(model.get_package_name("PKG_B"), None);
    }

    #[test]
    fn test_touch() {
        let mut model = CodeModel::new("Test");
        let before = model.metadata.updated_at;
        model.touch();
        assert!(model.metadata.updated_at >= before);
    }

    #[test]
    fn test_yaml_round_trip() {
        let mut model = CodeModel::new("Test Code Model");
        model.packages.push(CodePackage {
            id: "CORE_LIB".to_string(),
            name: "Core Library".to_string(),
            description: Some("Core functionality".to_string()),
            language: Some(Language::Rust),
            framework: None,
            path: Some("src/core".to_string()),
            element_id: Some("CTRL".to_string()),
        });
        model.dependencies.push(PackageDependency {
            from: "CORE_LIB".to_string(),
            to: "UTIL_LIB".to_string(),
        });

        let yaml = serde_yaml::to_string(&model).unwrap();
        let parsed: CodeModel = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.packages.len(), 1);
        assert_eq!(parsed.packages[0].id, "CORE_LIB");
        assert_eq!(parsed.packages[0].language, Some(Language::Rust));
        assert_eq!(parsed.dependencies.len(), 1);
        assert_eq!(parsed.dependencies[0].from, "CORE_LIB");
    }

    #[test]
    fn test_language_custom() {
        let pkg = CodePackage {
            id: "PKG".to_string(),
            name: "Pkg".to_string(),
            description: None,
            language: Some(Language::Custom("elixir".to_string())),
            framework: None,
            path: None,
            element_id: None,
        };
        let yaml = serde_yaml::to_string(&pkg).unwrap();
        assert!(yaml.contains("elixir"));
        let parsed: CodePackage = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.language, Some(Language::Custom("elixir".to_string())));
    }

    #[test]
    fn test_optional_fields_omitted() {
        let pkg = CodePackage {
            id: "PKG".to_string(),
            name: "Pkg".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        };
        let yaml = serde_yaml::to_string(&pkg).unwrap();
        assert!(!yaml.contains("description"));
        assert!(!yaml.contains("language"));
        assert!(!yaml.contains("framework"));
        assert!(!yaml.contains("path"));
        assert!(!yaml.contains("element_id"));
    }

    #[test]
    fn test_find_package_mut() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG_A".to_string(),
            name: "Old Name".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });

        model.find_package_mut("PKG_A").unwrap().name = "New Name".to_string();
        assert_eq!(model.find_package("PKG_A").unwrap().name, "New Name");
        assert!(model.find_package_mut("PKG_B").is_none());
    }
}
