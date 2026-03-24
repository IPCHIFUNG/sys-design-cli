use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Root structure for Logical Architecture Concept Model Diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicConceptDiagram {
    pub version: String,
    pub kind: DiagramKind,
    pub metadata: Metadata,
    pub system: System,
}

impl LogicConceptDiagram {
    /// Create a new LogicConceptDiagram with the given system
    pub fn new(system_id: &str, title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            kind: DiagramKind::LogicConceptDiagram,
            metadata: Metadata {
                title: title.to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            system: System {
                id: system_id.to_string(),
                name: system_id.to_string(),
                description: None,
                subsystems: Vec::new(),
                components: Vec::new(),
            },
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    /// Get all element IDs (for validation)
    pub fn all_element_ids(&self) -> Vec<&str> {
        let mut ids = vec![self.system.id.as_str()];
        self.system.collect_ids(&mut ids);
        ids
    }

    /// Find element name by ID
    pub fn get_element_name(&self, id: &str) -> Option<&str> {
        if self.system.id == id {
            return Some(&self.system.name);
        }
        self.system.find_name(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiagramKind {
    LogicConceptDiagram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

/// Element type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ElementType {
    System,
    Subsystem,
    Component,
    Module,
    Submodule,
}

/// System element (root)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub subsystems: Vec<Subsystem>,
    #[serde(default)]
    pub components: Vec<Component>,
}

impl System {
    fn collect_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        for sub in &self.subsystems {
            sub.collect_ids(ids);
        }
        for comp in &self.components {
            comp.collect_ids(ids);
        }
    }

    fn find_name<'a>(&'a self, id: &str) -> Option<&'a str> {
        for sub in &self.subsystems {
            if let Some(name) = sub.find_name(id) {
                return Some(name);
            }
        }
        for comp in &self.components {
            if let Some(name) = comp.find_name(id) {
                return Some(name);
            }
        }
        None
    }
}

/// Subsystem element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subsystem {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub components: Vec<Component>,
}

impl Subsystem {
    fn collect_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        ids.push(&self.id);
        for comp in &self.components {
            comp.collect_ids(ids);
        }
    }

    fn find_name<'a>(&'a self, id: &str) -> Option<&'a str> {
        if self.id == id {
            return Some(&self.name);
        }
        for comp in &self.components {
            if let Some(name) = comp.find_name(id) {
                return Some(name);
            }
        }
        None
    }
}

/// Component element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub modules: Vec<Module>,
}

impl Component {
    fn collect_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        ids.push(&self.id);
        for module in &self.modules {
            module.collect_ids(ids);
        }
    }

    fn find_name<'a>(&'a self, id: &str) -> Option<&'a str> {
        if self.id == id {
            return Some(&self.name);
        }
        for module in &self.modules {
            if let Some(name) = module.find_name(id) {
                return Some(name);
            }
        }
        None
    }
}

/// Module element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub submodules: Vec<Submodule>,
}

impl Module {
    fn collect_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        ids.push(&self.id);
        for sub in &self.submodules {
            sub.collect_ids(ids);
        }
    }

    fn find_name<'a>(&'a self, id: &str) -> Option<&'a str> {
        if self.id == id {
            return Some(&self.name);
        }
        for sub in &self.submodules {
            if let Some(name) = sub.find_name(id) {
                return Some(name);
            }
        }
        None
    }
}

/// Submodule element (leaf)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submodule {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub submodules: Vec<Submodule>,
}

impl Submodule {
    fn collect_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        ids.push(&self.id);
        for sub in &self.submodules {
            sub.collect_ids(ids);
        }
    }

    fn find_name<'a>(&'a self, id: &str) -> Option<&'a str> {
        if self.id == id {
            return Some(&self.name);
        }
        for sub in &self.submodules {
            if let Some(name) = sub.find_name(id) {
                return Some(name);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_diagram() {
        let diagram = LogicConceptDiagram::new("MY_SYSTEM", "My System");
        assert_eq!(diagram.system.id, "MY_SYSTEM");
        assert_eq!(diagram.metadata.title, "My System");
        assert!(diagram.system.subsystems.is_empty());
        assert!(diagram.system.components.is_empty());
    }

    #[test]
    fn test_nested_structure() {
        let mut diagram = LogicConceptDiagram::new("MY_SYSTEM", "My System");

        // Add a subsystem with component
        diagram.system.subsystems.push(Subsystem {
            id: "SUB_SYS".to_string(),
            name: "Sub System".to_string(),
            description: None,
            components: vec![Component {
                id: "COMP".to_string(),
                name: "Component".to_string(),
                description: None,
                modules: vec![],
            }],
        });

        // Add a direct component
        diagram.system.components.push(Component {
            id: "DIRECT_COMP".to_string(),
            name: "Direct Component".to_string(),
            description: None,
            modules: vec![],
        });

        let ids = diagram.all_element_ids();
        assert!(ids.contains(&"MY_SYSTEM"));
        assert!(ids.contains(&"SUB_SYS"));
        assert!(ids.contains(&"COMP"));
        assert!(ids.contains(&"DIRECT_COMP"));
    }
}
