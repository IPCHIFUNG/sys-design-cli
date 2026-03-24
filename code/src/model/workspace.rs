use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::c4::context::ContextDiagram;
use crate::model::logic::concept::LogicConceptDiagram;
use crate::model::logic::concept_model::LogicArchitectureConceptModel;

/// Workspace containing multiple diagrams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub version: String,
    pub metadata: WorkspaceMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_diagram: Option<ContextDiagram>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logic_architecture_concept_model: Option<LogicArchitectureConceptModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logic_view: Option<LogicConceptDiagram>,
}

impl Workspace {
    /// Create a new empty workspace
    pub fn new(title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            metadata: WorkspaceMetadata {
                title: title.to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            context_diagram: None,
            logic_architecture_concept_model: None,
            logic_view: None,
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    /// Check if context diagram exists
    pub fn has_context_diagram(&self) -> bool {
        self.context_diagram.is_some()
    }

    /// Check if logic architecture concept model exists
    pub fn has_logic_architecture_concept_model(&self) -> bool {
        self.logic_architecture_concept_model.is_some()
    }

    /// Check if logic view exists
    pub fn has_logic_view(&self) -> bool {
        self.logic_view.is_some()
    }

    /// Get the concept model for validation
    pub fn get_concept_model(&self) -> Option<&LogicArchitectureConceptModel> {
        self.logic_architecture_concept_model.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_workspace() {
        let workspace = Workspace::new("My Project");
        assert_eq!(workspace.metadata.title, "My Project");
        assert!(!workspace.has_context_diagram());
        assert!(!workspace.has_logic_architecture_concept_model());
        assert!(!workspace.has_logic_view());
    }

    #[test]
    fn test_workspace_with_diagrams() {
        let mut workspace = Workspace::new("My Project");
        workspace.context_diagram = Some(ContextDiagram::new("MY_SYSTEM", "My System"));
        workspace.logic_architecture_concept_model = Some(LogicArchitectureConceptModel::new("My Model"));
        workspace.logic_view = Some(LogicConceptDiagram::new("MY_SYSTEM", "My System"));

        assert!(workspace.has_context_diagram());
        assert!(workspace.has_logic_architecture_concept_model());
        assert!(workspace.has_logic_view());
    }
}
