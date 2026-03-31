use crate::model::c4::context::ContextDiagram;
use crate::model::logic::concept::LogicConceptDiagram;
use crate::model::logic::concept_model::LogicArchitectureConceptModel;
use crate::model::runtime::RuntimeView;
use crate::model::workspace::Workspace;
use crate::utils::error::{AppError, Result};
use std::path::Path;

/// YAML file store for diagrams
pub struct YamlStore;

impl YamlStore {
    /// Load a Workspace from a YAML file
    pub fn load_workspace<P: AsRef<Path>>(path: P) -> Result<Workspace> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let workspace: Workspace = serde_yaml::from_str(&content)?;
        Ok(workspace)
    }

    /// Save a Workspace to a YAML file
    pub fn save_workspace<P: AsRef<Path>>(path: P, workspace: &Workspace) -> Result<()> {
        let content = serde_yaml::to_string(workspace)?;
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    /// Check if a file exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Create a new file with the given workspace
    pub fn create_workspace<P: AsRef<Path>>(path: P, workspace: &Workspace) -> Result<()> {
        if Self::exists(&path) {
            return Err(AppError::ElementAlreadyExists(
                path.as_ref().display().to_string()
            ));
        }
        Self::save_workspace(path, workspace)
    }

    /// Load workspace from file, converting legacy formats if necessary
    /// Returns Workspace (always workspace format)
    pub fn load_workspace_any<P: AsRef<Path>>(path: P) -> Result<Workspace> {
        let content = std::fs::read_to_string(path.as_ref())?;

        // Try to parse as workspace first
        if let Ok(workspace) = serde_yaml::from_str::<Workspace>(&content) {
            return Ok(workspace);
        }

        // Try to parse as individual context diagram and convert to workspace
        if let Ok(context_diagram) = serde_yaml::from_str::<ContextDiagram>(&content) {
            let mut workspace = Workspace::new(&context_diagram.metadata.title);
            workspace.context_diagram = Some(context_diagram);
            return Ok(workspace);
        }

        // Try to parse as logic diagram and convert to workspace
        if let Ok(logic_diagram) = serde_yaml::from_str::<LogicConceptDiagram>(&content) {
            let mut workspace = Workspace::new(&logic_diagram.metadata.title);
            workspace.logic_view = Some(logic_diagram);
            return Ok(workspace);
        }

        // Try to parse as concept model and convert to workspace
        if let Ok(concept_model) = serde_yaml::from_str::<LogicArchitectureConceptModel>(&content) {
            let mut workspace = Workspace::new(&concept_model.metadata.title);
            workspace.logic_architecture_concept_model = Some(concept_model);
            return Ok(workspace);
        }

        // Try to parse as runtime view and convert to workspace
        if let Ok(runtime_view) = serde_yaml::from_str::<RuntimeView>(&content) {
            let mut workspace = Workspace::new(&runtime_view.metadata.title);
            workspace.runtime_view = Some(runtime_view);
            return Ok(workspace);
        }

        Err(AppError::InvalidOperation("Unable to parse YAML file as workspace or diagram".to_string()))
    }

    /// Save context diagram to workspace (always workspace format)
    pub fn save_context_to_workspace<P: AsRef<Path>>(path: P, workspace: &mut Workspace, diagram: &ContextDiagram) -> Result<()> {
        workspace.context_diagram = Some(diagram.clone());
        workspace.touch();
        Self::save_workspace(path, workspace)
    }

    /// Save logic view to workspace (always workspace format)
    pub fn save_logic_to_workspace<P: AsRef<Path>>(path: P, workspace: &mut Workspace, diagram: &LogicConceptDiagram) -> Result<()> {
        workspace.logic_view = Some(diagram.clone());
        workspace.touch();
        Self::save_workspace(path, workspace)
    }

    /// Save runtime view to workspace (always workspace format)
    pub fn save_runtime_to_workspace<P: AsRef<Path>>(path: P, workspace: &mut Workspace) -> Result<()> {
        workspace.touch();
        Self::save_workspace(path, workspace)
    }
}
