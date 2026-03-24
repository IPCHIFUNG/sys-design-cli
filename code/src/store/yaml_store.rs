use crate::model::c4::context::ContextDiagram;
use crate::model::logic::concept::LogicConceptDiagram;
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

    /// Load a ContextDiagram from a YAML file
    pub fn load_context<P: AsRef<Path>>(path: P) -> Result<ContextDiagram> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let diagram: ContextDiagram = serde_yaml::from_str(&content)?;
        Ok(diagram)
    }

    /// Save a ContextDiagram to a YAML file
    pub fn save_context<P: AsRef<Path>>(path: P, diagram: &ContextDiagram) -> Result<()> {
        let content = serde_yaml::to_string(diagram)?;
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    /// Load a LogicConceptDiagram from a YAML file
    pub fn load_logic_concept<P: AsRef<Path>>(path: P) -> Result<LogicConceptDiagram> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let diagram: LogicConceptDiagram = serde_yaml::from_str(&content)?;
        Ok(diagram)
    }

    /// Save a LogicConceptDiagram to a YAML file
    pub fn save_logic_concept<P: AsRef<Path>>(path: P, diagram: &LogicConceptDiagram) -> Result<()> {
        let content = serde_yaml::to_string(diagram)?;
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    /// Check if a file exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Create a new file with the given context diagram
    pub fn create_context<P: AsRef<Path>>(path: P, diagram: &ContextDiagram) -> Result<()> {
        if Self::exists(&path) {
            return Err(AppError::ElementAlreadyExists(
                path.as_ref().display().to_string()
            ));
        }
        Self::save_context(path, diagram)
    }

    /// Create a new file with the given logic concept diagram
    pub fn create_logic_concept<P: AsRef<Path>>(path: P, diagram: &LogicConceptDiagram) -> Result<()> {
        if Self::exists(&path) {
            return Err(AppError::ElementAlreadyExists(
                path.as_ref().display().to_string()
            ));
        }
        Self::save_logic_concept(path, diagram)
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
}

// Backward compatibility
impl YamlStore {
    /// Load a ContextDiagram from a YAML file (alias for load_context)
    pub fn load<P: AsRef<Path>>(path: P) -> Result<ContextDiagram> {
        Self::load_context(path)
    }

    /// Save a ContextDiagram to a YAML file (alias for save_context)
    pub fn save<P: AsRef<Path>>(path: P, diagram: &ContextDiagram) -> Result<()> {
        Self::save_context(path, diagram)
    }

    /// Create a new file with the given diagram (alias for create_context)
    pub fn create<P: AsRef<Path>>(path: P, diagram: &ContextDiagram) -> Result<()> {
        Self::create_context(path, diagram)
    }
}
