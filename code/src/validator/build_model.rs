use crate::model::build::BuildModel;
use crate::validator::result::{Severity, ValidationError, ValidationResult};
use regex::Regex;
use std::collections::HashSet;

/// Validate a BuildModel
pub fn validate(model: &BuildModel) -> ValidationResult {
    let mut result = ValidationResult::new();
    let id_pattern = Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap();

    let mut seen_ids = HashSet::new();

    for artifact in &model.artifacts {
        let location = format!("artifacts.{}", artifact.id);

        // BM001: Artifact must have non-empty id and name
        if artifact.id.is_empty() {
            result.add_error(ValidationError {
                code: "BM001".to_string(),
                rule: "ArtifactRequiredFields".to_string(),
                message: "Artifact ID cannot be empty".to_string(),
                severity: Severity::Error,
                location: Some(location.clone()),
            });
        }
        if artifact.name.is_empty() {
            result.add_error(ValidationError {
                code: "BM001".to_string(),
                rule: "ArtifactRequiredFields".to_string(),
                message: format!("Artifact '{}' name cannot be empty", artifact.id),
                severity: Severity::Error,
                location: Some(format!("{}.name", location)),
            });
        }

        // BM002: Artifact ID must be unique
        if !artifact.id.is_empty() {
            if !seen_ids.insert(artifact.id.clone()) {
                result.add_error(ValidationError {
                    code: "BM002".to_string(),
                    rule: "ArtifactIdUniqueness".to_string(),
                    message: format!("Duplicate artifact ID: {}", artifact.id),
                    severity: Severity::Error,
                    location: Some(location.clone()),
                });
            }
        }

        // BM003: Artifact ID should follow UPPER_SNAKE_CASE
        if !artifact.id.is_empty() && !id_pattern.is_match(&artifact.id) {
            result.add_error(ValidationError {
                code: "BM003".to_string(),
                rule: "ArtifactIdNaming".to_string(),
                message: format!(
                    "Artifact ID '{}' does not follow UPPER_SNAKE_CASE convention",
                    artifact.id
                ),
                severity: Severity::Warning,
                location: Some(format!("{}.id", location)),
            });
        }
    }

    // BM004: Dependency from/to must reference existing artifacts
    for dep in &model.dependencies {
        if !seen_ids.contains(&dep.from) {
            result.add_error(ValidationError {
                code: "BM004".to_string(),
                rule: "DependencyReferencesValid".to_string(),
                message: format!(
                    "Dependency 'from' references non-existent artifact: {}",
                    dep.from
                ),
                severity: Severity::Error,
                location: Some(format!("dependencies.{}", dep.from)),
            });
        }
        if !seen_ids.contains(&dep.to) {
            result.add_error(ValidationError {
                code: "BM004".to_string(),
                rule: "DependencyReferencesValid".to_string(),
                message: format!(
                    "Dependency 'to' references non-existent artifact: {}",
                    dep.to
                ),
                severity: Severity::Error,
                location: Some(format!("dependencies.{}", dep.to)),
            });
        }
    }

    // BM005: Minimum artifacts warning
    if model.artifacts.is_empty() {
        result.add_error(ValidationError {
            code: "BM005".to_string(),
            rule: "MinArtifacts".to_string(),
            message: "Build model should have at least 1 artifact".to_string(),
            severity: Severity::Warning,
            location: None,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::build::*;

    fn create_test_model() -> BuildModel {
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
        model.artifacts.push(BuildArtifact {
            id: "ART_B".to_string(),
            name: "Artifact B".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Library,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        model.dependencies.push(ArtifactDependency {
            from: "ART_A".to_string(),
            to: "ART_B".to_string(),
        });
        model
    }

    #[test]
    fn test_valid_build_model() {
        let model = create_test_model();
        let result = validate(&model);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_bm001_empty_id() {
        let mut model = BuildModel::new("Test");
        model.artifacts.push(BuildArtifact {
            id: "".to_string(),
            name: "No ID".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "BM001" && e.severity == Severity::Error));
    }

    #[test]
    fn test_bm001_empty_name() {
        let mut model = BuildModel::new("Test");
        model.artifacts.push(BuildArtifact {
            id: "ART".to_string(),
            name: "".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "BM001" && e.severity == Severity::Error));
    }

    #[test]
    fn test_bm002_duplicate_id() {
        let mut model = BuildModel::new("Test");
        model.artifacts.push(BuildArtifact {
            id: "ART".to_string(),
            name: "First".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        model.artifacts.push(BuildArtifact {
            id: "ART".to_string(),
            name: "Second".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "BM002"));
    }

    #[test]
    fn test_bm003_naming() {
        let mut model = BuildModel::new("Test");
        model.artifacts.push(BuildArtifact {
            id: "bad-name".to_string(),
            name: "Bad Name".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        let result = validate(&model);
        assert!(result.is_valid); // Warning only
        assert!(result.errors.iter().any(|e| e.code == "BM003" && e.severity == Severity::Warning));
    }

    #[test]
    fn test_bm004_invalid_dep() {
        let mut model = BuildModel::new("Test");
        model.artifacts.push(BuildArtifact {
            id: "ART_A".to_string(),
            name: "A".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        model.dependencies.push(ArtifactDependency {
            from: "ART_A".to_string(),
            to: "ART_NONEXISTENT".to_string(),
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "BM004"));
    }

    #[test]
    fn test_bm005_min_artifacts() {
        let model = BuildModel::new("Test");
        let result = validate(&model);
        assert!(result.is_valid); // Warning only
        assert!(result.errors.iter().any(|e| e.code == "BM005" && e.severity == Severity::Warning));
    }
}
