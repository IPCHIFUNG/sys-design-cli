use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Root structure for Build Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildModel {
    pub version: String,
    pub kind: BuildDiagramKind,
    pub metadata: BuildMetadata,
    #[serde(default)]
    pub artifacts: Vec<BuildArtifact>,
    #[serde(default)]
    pub dependencies: Vec<ArtifactDependency>,
}

impl BuildModel {
    pub fn new(title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            kind: BuildDiagramKind::BuildModel,
            metadata: BuildMetadata {
                title: title.to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            artifacts: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    pub fn all_artifact_ids(&self) -> Vec<&str> {
        self.artifacts.iter().map(|a| a.id.as_str()).collect()
    }

    pub fn find_artifact(&self, id: &str) -> Option<&BuildArtifact> {
        self.artifacts.iter().find(|a| a.id == id)
    }

    pub fn find_artifact_mut(&mut self, id: &str) -> Option<&mut BuildArtifact> {
        self.artifacts.iter_mut().find(|a| a.id == id)
    }

    pub fn get_artifact_name(&self, id: &str) -> Option<&str> {
        self.find_artifact(id).map(|a| a.name.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BuildDiagramKind {
    BuildModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_tool: Option<BuildTool>,
    #[serde(default)]
    pub output_type: OutputType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_packages: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<BuildProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_args: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    #[default]
    Binary,
    Library,
    DockerImage,
    Archive,
    Bundle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BuildTool {
    Cargo,
    Maven,
    Gradle,
    Npm,
    Pip,
    Make,
    Cmake,
    GoBuild,
    Dotnet,
    Bazel,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BuildProfile {
    #[default]
    Debug,
    Release,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactDependency {
    pub from: String,
    pub to: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_build_model() {
        let model = BuildModel::new("Test Build");
        assert_eq!(model.version, "1.0");
        assert_eq!(model.kind, BuildDiagramKind::BuildModel);
        assert!(model.artifacts.is_empty());
        assert!(model.dependencies.is_empty());
    }

    #[test]
    fn test_all_artifact_ids() {
        let mut model = BuildModel::new("Test");
        model.artifacts.push(BuildArtifact {
            id: "ART_A".to_string(),
            name: "Artifact A".to_string(),
            description: None,
            build_tool: Some(BuildTool::Cargo),
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        assert_eq!(model.all_artifact_ids(), vec!["ART_A"]);
    }

    #[test]
    fn test_find_artifact() {
        let mut model = BuildModel::new("Test");
        model.artifacts.push(BuildArtifact {
            id: "ART_A".to_string(),
            name: "Artifact A".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        assert!(model.find_artifact("ART_A").is_some());
        assert!(model.find_artifact("ART_B").is_none());
    }

    #[test]
    fn test_output_type_default() {
        let artifact = BuildArtifact {
            id: "A".to_string(),
            name: "A".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::default(),
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        };
        assert_eq!(artifact.output_type, OutputType::Binary);
    }

    #[test]
    fn test_build_tool_custom() {
        let tool = BuildTool::Custom("bazel".to_string());
        let yaml = serde_yaml::to_string(&tool).unwrap();
        assert!(yaml.contains("bazel"));
        let parsed: BuildTool = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, BuildTool::Custom("bazel".to_string()));
    }

    #[test]
    fn test_yaml_round_trip() {
        let mut model = BuildModel::new("Test Build");
        model.artifacts.push(BuildArtifact {
            id: "CORE_BIN".to_string(),
            name: "Core Binary".to_string(),
            description: Some("Main executable".to_string()),
            build_tool: Some(BuildTool::Cargo),
            output_type: OutputType::Binary,
            source_packages: vec!["CORE_LIB".to_string()],
            build_file: Some("Cargo.toml".to_string()),
            profile: Some(BuildProfile::Release),
            build_args: None,
        });
        model.dependencies.push(ArtifactDependency {
            from: "CORE_BIN".to_string(),
            to: "UTIL_BIN".to_string(),
        });

        let yaml = serde_yaml::to_string(&model).unwrap();
        let parsed: BuildModel = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.artifacts.len(), 1);
        assert_eq!(parsed.artifacts[0].id, "CORE_BIN");
        assert_eq!(parsed.artifacts[0].source_packages, vec!["CORE_LIB"]);
        assert_eq!(parsed.dependencies.len(), 1);
    }
}
