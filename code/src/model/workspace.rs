use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::c4::context::ContextDiagram;
use crate::model::logic::concept::LogicConceptDiagram;

/// Workspace containing multiple diagrams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub version: String,
    pub metadata: WorkspaceMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_diagram: Option<ContextDiagram>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logic_concept_diagram: Option<LogicConceptDiagram>,
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
            logic_concept_diagram: None,
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

    /// Check if logic concept diagram exists
    pub fn has_logic_concept_diagram(&self) -> bool {
        self.logic_concept_diagram.is_some()
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
        assert!(!workspace.has_logic_concept_diagram());
    }

    #[test]
    fn test_workspace_with_diagrams() {
        let mut workspace = Workspace::new("My Project");
        workspace.context_diagram = Some(ContextDiagram::new("MY_SYSTEM", "My System"));
        workspace.logic_concept_diagram = Some(LogicConceptDiagram::new("MY_SYSTEM", "My System"));

        assert!(workspace.has_context_diagram());
        assert!(workspace.has_logic_concept_diagram());
    }
}
