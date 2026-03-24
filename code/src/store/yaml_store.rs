use crate::model::c4::context::ContextDiagram;
use crate::model::logic::concept::LogicConceptDiagram;
use crate::model::logic::concept_model::LogicArchitectureConceptModel;
use crate::model::workspace::Workspace;
use crate::utils::error::{AppError, Result};
use std::path::Path;

/// YAML file store for diagrams
pub struct YamlStore;

/// Result of loading a file that could be either a workspace or individual diagram
pub enum LoadedContext {
    /// Loaded as workspace, contains context_diagram
    Workspace { workspace: Workspace, has_context: bool },
    /// Loaded as individual context diagram
    Diagram(ContextDiagram),
}

/// Result of loading a file that could be either a workspace or individual logic diagram
pub enum LoadedLogic {
    /// Loaded as workspace, contains logic_view
    Workspace { workspace: Workspace, has_logic_view: bool },
    /// Loaded as individual logic diagram
    Diagram(LogicConceptDiagram),
}

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

    /// Load a LogicConceptDiagram (Logic View) from a YAML file
    pub fn load_logic_concept<P: AsRef<Path>>(path: P) -> Result<LogicConceptDiagram> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let diagram: LogicConceptDiagram = serde_yaml::from_str(&content)?;
        Ok(diagram)
    }

    /// Alias for load_logic_concept (Logic View)
    pub fn load_logic_view<P: AsRef<Path>>(path: P) -> Result<LogicConceptDiagram> {
        Self::load_logic_concept(path)
    }

    /// Save a LogicConceptDiagram (Logic View) to a YAML file
    pub fn save_logic_concept<P: AsRef<Path>>(path: P, diagram: &LogicConceptDiagram) -> Result<()> {
        let content = serde_yaml::to_string(diagram)?;
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    /// Load a LogicArchitectureConceptModel from a YAML file
    pub fn load_concept_model<P: AsRef<Path>>(path: P) -> Result<LogicArchitectureConceptModel> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let model: LogicArchitectureConceptModel = serde_yaml::from_str(&content)?;
        Ok(model)
    }

    /// Save a LogicArchitectureConceptModel to a YAML file
    pub fn save_concept_model<P: AsRef<Path>>(path: P, model: &LogicArchitectureConceptModel) -> Result<()> {
        let content = serde_yaml::to_string(model)?;
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

    /// Create a new file with the given concept model
    pub fn create_concept_model<P: AsRef<Path>>(path: P, model: &LogicArchitectureConceptModel) -> Result<()> {
        if Self::exists(&path) {
            return Err(AppError::ElementAlreadyExists(
                path.as_ref().display().to_string()
            ));
        }
        Self::save_concept_model(path, model)
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

// Workspace-aware load/save for context diagrams
impl YamlStore {
    /// Load context diagram from either workspace or standalone file
    /// Returns LoadedContext enum indicating what was loaded
    pub fn load_context_any<P: AsRef<Path>>(path: P) -> Result<LoadedContext> {
        let content = std::fs::read_to_string(path.as_ref())?;

        // Check if this looks like a workspace (has workspace-specific keys)
        let looks_like_workspace = content.contains("context_diagram:")
            || content.contains("logic_view:")
            || content.contains("logic_architecture_concept_model:");

        if looks_like_workspace {
            if let Ok(workspace) = serde_yaml::from_str::<Workspace>(&content) {
                let has_context = workspace.context_diagram.is_some();
                return Ok(LoadedContext::Workspace { workspace, has_context });
            }
        }

        // Try to parse as individual context diagram
        if let Ok(diagram) = serde_yaml::from_str::<ContextDiagram>(&content) {
            return Ok(LoadedContext::Diagram(diagram));
        }

        // Try to parse as logic diagram and convert to workspace
        if let Ok(logic_diagram) = serde_yaml::from_str::<LogicConceptDiagram>(&content) {
            let mut workspace = Workspace::new(&logic_diagram.metadata.title);
            workspace.logic_view = Some(logic_diagram);
            return Ok(LoadedContext::Workspace { workspace, has_context: false });
        }

        // Try to parse as workspace (may have been misidentified)
        if let Ok(workspace) = serde_yaml::from_str::<Workspace>(&content) {
            let has_context = workspace.context_diagram.is_some();
            return Ok(LoadedContext::Workspace { workspace, has_context });
        }

        Err(AppError::InvalidOperation("Unable to parse YAML file as context diagram, logic diagram, or workspace".to_string()))
    }

    /// Save context diagram, preserving workspace format if applicable
    pub fn save_context_any<P: AsRef<Path>>(path: P, loaded: &LoadedContext, diagram: &ContextDiagram) -> Result<()> {
        match loaded {
            LoadedContext::Workspace { workspace, .. } => {
                let mut ws = workspace.clone();
                ws.context_diagram = Some(diagram.clone());
                ws.touch();
                Self::save_workspace(path, &ws)
            }
            LoadedContext::Diagram(_) => {
                Self::save_context(path, diagram)
            }
        }
    }
}

// Workspace-aware load/save for logic diagrams
impl YamlStore {
    /// Load logic diagram from either workspace or standalone file
    /// Returns LoadedLogic enum indicating what was loaded
    pub fn load_logic_any<P: AsRef<Path>>(path: P) -> Result<LoadedLogic> {
        let content = std::fs::read_to_string(path.as_ref())?;

        // Check if this looks like a workspace (has workspace-specific keys)
        let looks_like_workspace = content.contains("context_diagram:")
            || content.contains("logic_view:")
            || content.contains("logic_architecture_concept_model:");

        if looks_like_workspace {
            if let Ok(workspace) = serde_yaml::from_str::<Workspace>(&content) {
                let has_logic_view = workspace.logic_view.is_some();
                return Ok(LoadedLogic::Workspace { workspace, has_logic_view });
            }
        }

        // Try to parse as individual logic diagram
        if let Ok(diagram) = serde_yaml::from_str::<LogicConceptDiagram>(&content) {
            return Ok(LoadedLogic::Diagram(diagram));
        }

        // Try to parse as context diagram and convert to workspace
        if let Ok(context_diagram) = serde_yaml::from_str::<ContextDiagram>(&content) {
            let mut workspace = Workspace::new(&context_diagram.metadata.title);
            workspace.context_diagram = Some(context_diagram);
            return Ok(LoadedLogic::Workspace { workspace, has_logic_view: false });
        }

        // Try to parse as workspace (may have been misidentified)
        if let Ok(workspace) = serde_yaml::from_str::<Workspace>(&content) {
            let has_logic_view = workspace.logic_view.is_some();
            return Ok(LoadedLogic::Workspace { workspace, has_logic_view });
        }

        Err(AppError::InvalidOperation("Unable to parse YAML file as logic diagram, context diagram, or workspace".to_string()))
    }

    /// Save logic diagram, preserving workspace format if applicable
    pub fn save_logic_any<P: AsRef<Path>>(path: P, loaded: &LoadedLogic, diagram: &LogicConceptDiagram) -> Result<()> {
        match loaded {
            LoadedLogic::Workspace { workspace, .. } => {
                let mut ws = workspace.clone();
                ws.logic_view = Some(diagram.clone());
                ws.touch();
                Self::save_workspace(path, &ws)
            }
            LoadedLogic::Diagram(_) => {
                Self::save_logic_concept(path, diagram)
            }
        }
    }
}
