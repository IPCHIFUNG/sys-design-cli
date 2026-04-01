use crate::model::build::{ArtifactDependency, BuildArtifact, BuildModel, BuildProfile, BuildTool, OutputType};
use crate::utils::error::{AppError, Result};

/// Operations for modifying BuildModel
pub struct BuildOperations;

impl BuildOperations {
    /// Add a new artifact (rejects duplicate ID)
    pub fn add_artifact(
        model: &mut BuildModel,
        id: &str,
        name: Option<&str>,
        desc: Option<&str>,
        build_tool: Option<BuildTool>,
        output_type: Option<OutputType>,
        source_packages: Vec<String>,
        build_file: Option<&str>,
        profile: Option<BuildProfile>,
        build_args: Option<&str>,
    ) -> Result<()> {
        if model.artifacts.iter().any(|a| a.id == id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "artifact: {}",
                id
            )));
        }

        model.artifacts.push(BuildArtifact {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: desc.map(|s| s.to_string()),
            build_tool,
            output_type: output_type.unwrap_or_default(),
            source_packages,
            build_file: build_file.map(|s| s.to_string()),
            profile,
            build_args: build_args.map(|s| s.to_string()),
        });
        model.touch();
        Ok(())
    }

    /// Remove an artifact by ID (cascade: also removes dependencies referencing this artifact)
    pub fn remove_artifact(model: &mut BuildModel, id: &str) -> Result<()> {
        let idx = model
            .artifacts
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("artifact: {}", id)))?;

        model.artifacts.remove(idx);

        // Remove dependencies referencing this artifact
        model
            .dependencies
            .retain(|d| d.from != id && d.to != id);

        model.touch();
        Ok(())
    }

    /// Add a dependency between artifacts
    pub fn add_dependency(model: &mut BuildModel, from: &str, to: &str) -> Result<()> {
        if !model.artifacts.iter().any(|a| a.id == from) {
            return Err(AppError::ElementNotFound(format!("artifact: {}", from)));
        }
        if !model.artifacts.iter().any(|a| a.id == to) {
            return Err(AppError::ElementNotFound(format!("artifact: {}", to)));
        }

        let dep = ArtifactDependency {
            from: from.to_string(),
            to: to.to_string(),
        };
        if model.dependencies.contains(&dep) {
            return Err(AppError::ElementAlreadyExists(format!(
                "dependency: {} -> {}",
                from, to
            )));
        }

        model.dependencies.push(dep);
        model.touch();
        Ok(())
    }

    /// Remove a dependency
    pub fn remove_dependency(model: &mut BuildModel, from: &str, to: &str) -> Result<()> {
        let idx = model
            .dependencies
            .iter()
            .position(|d| d.from == from && d.to == to)
            .ok_or_else(|| {
                AppError::ElementNotFound(format!("dependency: {} -> {}", from, to))
            })?;

        model.dependencies.remove(idx);
        model.touch();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> BuildModel {
        BuildModel::new("Test Build Model")
    }

    #[test]
    fn test_add_artifact() {
        let mut model = create_test_model();
        BuildOperations::add_artifact(
            &mut model,
            "CORE_BIN",
            Some("Core Binary"),
            Some("Main executable"),
            Some(BuildTool::Cargo),
            Some(OutputType::Binary),
            vec!["CORE_LIB".to_string()],
            Some("Cargo.toml"),
            Some(BuildProfile::Release),
            None,
        )
        .unwrap();
        assert_eq!(model.artifacts.len(), 1);
        assert_eq!(model.artifacts[0].id, "CORE_BIN");
        assert_eq!(model.artifacts[0].name, "Core Binary");
        assert_eq!(model.artifacts[0].source_packages, vec!["CORE_LIB"]);
    }

    #[test]
    fn test_add_duplicate_artifact() {
        let mut model = create_test_model();
        BuildOperations::add_artifact(&mut model, "ART_A", None, None, None, None, vec![], None, None, None).unwrap();
        let result = BuildOperations::add_artifact(&mut model, "ART_A", None, None, None, None, vec![], None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_artifact() {
        let mut model = create_test_model();
        BuildOperations::add_artifact(&mut model, "ART_A", None, None, None, None, vec![], None, None, None).unwrap();
        BuildOperations::add_artifact(&mut model, "ART_B", None, None, None, None, vec![], None, None, None).unwrap();
        BuildOperations::add_dependency(&mut model, "ART_A", "ART_B").unwrap();

        BuildOperations::remove_artifact(&mut model, "ART_A").unwrap();
        assert_eq!(model.artifacts.len(), 1);
        // Dependencies referencing ART_A should be removed
        assert!(model.dependencies.is_empty());
    }

    #[test]
    fn test_add_dependency() {
        let mut model = create_test_model();
        BuildOperations::add_artifact(&mut model, "ART_A", None, None, None, None, vec![], None, None, None).unwrap();
        BuildOperations::add_artifact(&mut model, "ART_B", None, None, None, None, vec![], None, None, None).unwrap();
        BuildOperations::add_dependency(&mut model, "ART_A", "ART_B").unwrap();
        assert_eq!(model.dependencies.len(), 1);
    }

    #[test]
    fn test_add_dependency_invalid_from() {
        let mut model = create_test_model();
        BuildOperations::add_artifact(&mut model, "ART_B", None, None, None, None, vec![], None, None, None).unwrap();
        let result = BuildOperations::add_dependency(&mut model, "ART_A", "ART_B");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_dependency() {
        let mut model = create_test_model();
        BuildOperations::add_artifact(&mut model, "ART_A", None, None, None, None, vec![], None, None, None).unwrap();
        BuildOperations::add_artifact(&mut model, "ART_B", None, None, None, None, vec![], None, None, None).unwrap();
        BuildOperations::add_dependency(&mut model, "ART_A", "ART_B").unwrap();
        BuildOperations::remove_dependency(&mut model, "ART_A", "ART_B").unwrap();
        assert!(model.dependencies.is_empty());
    }

    #[test]
    fn test_list_empty_artifacts() {
        let model = create_test_model();
        assert!(model.artifacts.is_empty());
        assert!(model.all_artifact_ids().is_empty());
    }
}
