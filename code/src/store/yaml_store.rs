use crate::model::c4::context::ContextDiagram;
use crate::utils::error::{AppError, Result};
use std::path::Path;

/// YAML file store for ContextDiagram
pub struct YamlStore;

impl YamlStore {
    /// Load a ContextDiagram from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<ContextDiagram> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let diagram: ContextDiagram = serde_yaml::from_str(&content)?;
        Ok(diagram)
    }

    /// Save a ContextDiagram to a YAML file
    pub fn save<P: AsRef<Path>>(path: P, diagram: &ContextDiagram) -> Result<()> {
        let content = serde_yaml::to_string(diagram)?;
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    /// Check if a file exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Create a new file with the given diagram
    pub fn create<P: AsRef<Path>>(path: P, diagram: &ContextDiagram) -> Result<()> {
        if Self::exists(&path) {
            return Err(AppError::ElementAlreadyExists(
                path.as_ref().display().to_string()
            ));
        }
        Self::save(path, diagram)
    }
}
