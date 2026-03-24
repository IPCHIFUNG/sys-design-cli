use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Root structure for Logic Architecture Concept Model Diagram
/// 定义逻辑架构的层次结构规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicArchitectureConceptModel {
    pub version: String,
    pub kind: ConceptModelKind,
    pub metadata: ConceptMetadata,
    pub hierarchy: HierarchyDefinition,
    /// Allowed element types in this concept model
    #[serde(default)]
    pub element_types: Vec<ElementTypeDefinition>,
}

impl LogicArchitectureConceptModel {
    /// Create a new LogicArchitectureConceptModel with empty hierarchy
    pub fn new(title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            kind: ConceptModelKind::LogicArchitectureConceptModel,
            metadata: ConceptMetadata {
                title: title.to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            hierarchy: HierarchyDefinition::empty(),
            element_types: Vec::new(),
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    /// Get a level definition by id
    pub fn get_level(&self, level_id: &str) -> Option<&LevelDefinition> {
        self.hierarchy.levels.iter().find(|l| l.id == level_id)
    }

    /// Check if a parent can contain a child type
    pub fn can_contain(&self, parent_type: &str, child_type: &str) -> bool {
        if let Some(level) = self.get_level(parent_type) {
            level.can_contain.iter().any(|t| t == child_type || t == "*")
        } else {
            false
        }
    }

    /// Get the root level (first level in hierarchy)
    pub fn root_level(&self) -> Option<&LevelDefinition> {
        self.hierarchy.levels.first()
    }

    /// Check if an element type is allowed
    pub fn has_element_type(&self, type_name: &str) -> bool {
        let upper = type_name.to_uppercase();
        self.element_types.iter().any(|et| et.id.to_uppercase() == upper)
    }

    /// Add an element type
    pub fn add_element_type(&mut self, type_name: &str) -> bool {
        let upper = type_name.to_uppercase();
        if self.has_element_type(&upper) {
            return false;
        }
        self.element_types.push(ElementTypeDefinition {
            id: upper,
            name: type_name.to_string(),
            description: None,
        });
        self.touch();
        true
    }

    /// Remove an element type
    pub fn remove_element_type(&mut self, type_name: &str) -> bool {
        let upper = type_name.to_uppercase();
        let initial_len = self.element_types.len();
        self.element_types.retain(|et| et.id != upper);
        if self.element_types.len() != initial_len {
            self.touch();
            true
        } else {
            false
        }
    }

    /// Get unused element types (types defined but not used in logic view)
    pub fn get_unused_element_types(&self, used_types: &[&str]) -> Vec<&str> {
        self.element_types
            .iter()
            .filter(|et| !used_types.iter().any(|ut| ut.to_uppercase() == et.id))
            .map(|et| et.id.as_str())
            .collect()
    }

    /// Check if a hierarchy level exists
    pub fn has_level(&self, level_id: &str) -> bool {
        let upper = level_id.to_uppercase();
        self.hierarchy.levels.iter().any(|l| l.id == upper)
    }

    /// Check if a containment relationship exists
    pub fn has_containment(&self, parent_type: &str, child_type: &str) -> bool {
        let parent_upper = parent_type.to_uppercase();
        let child_upper = child_type.to_uppercase();
        if let Some(level) = self.get_level(&parent_upper) {
            level.can_contain.iter().any(|c| c == &child_upper)
        } else {
            false
        }
    }

    /// Add a containment relationship (parent can contain child)
    /// Returns true if added, false if already exists
    pub fn add_containment(&mut self, parent_type: &str, child_type: &str) -> bool {
        let parent_upper = parent_type.to_uppercase();
        let child_upper = child_type.to_uppercase();

        // Find or create the parent level
        let level = self.hierarchy.levels.iter_mut().find(|l| l.id == parent_upper);
        if let Some(level) = level {
            // Check if containment already exists
            if level.can_contain.iter().any(|c| c == &child_upper) {
                return false;
            }
            level.can_contain.push(child_upper);
        } else {
            // Create new level
            self.hierarchy.levels.push(LevelDefinition {
                id: parent_upper,
                name: parent_type.to_string(),
                description: None,
                can_contain: vec![child_upper],
            });
        }
        self.touch();
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConceptModelKind {
    LogicArchitectureConceptModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMetadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

/// Hierarchy definition containing all levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyDefinition {
    pub levels: Vec<LevelDefinition>,
}

impl HierarchyDefinition {
    /// Create empty hierarchy
    pub fn empty() -> Self {
        Self {
            levels: Vec::new(),
        }
    }
}

/// Definition of a single level in the hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelDefinition {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Types that this level can contain
    pub can_contain: Vec<String>,
}

/// Definition of an element type that can be used in logic view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementTypeDefinition {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_hierarchy() {
        let model = LogicArchitectureConceptModel::new("Test Model");
        assert!(!model.has_containment("SYSTEM", "SUBSYSTEM"));
    }

    #[test]
    fn test_add_containment() {
        let mut model = LogicArchitectureConceptModel::new("Test Model");

        // Add containment
        assert!(model.add_containment("system", "subsystem"));
        assert!(model.has_containment("SYSTEM", "SUBSYSTEM"));
        assert!(model.can_contain("SYSTEM", "SUBSYSTEM"));

        // Duplicate should fail
        assert!(!model.add_containment("SYSTEM", "SUBSYSTEM"));

        // Add more
        assert!(model.add_containment("subsystem", "component"));
        assert!(model.add_containment("component", "module"));

        assert!(model.can_contain("SUBSYSTEM", "COMPONENT"));
        assert!(model.can_contain("COMPONENT", "MODULE"));
    }

    #[test]
    fn test_get_level() {
        let mut model = LogicArchitectureConceptModel::new("Test Model");
        model.add_containment("system", "subsystem");

        let system_level = model.get_level("SYSTEM");
        assert!(system_level.is_some());
        assert_eq!(system_level.unwrap().name, "system");
    }

    #[test]
    fn test_element_types() {
        let mut model = LogicArchitectureConceptModel::new("Test Model");

        // Initially empty
        assert!(!model.has_element_type("SUBSYSTEM"));

        // Add element type
        assert!(model.add_element_type("subsystem"));
        assert!(model.has_element_type("SUBSYSTEM"));
        assert!(model.has_element_type("subsystem")); // case insensitive

        // Duplicate should fail
        assert!(!model.add_element_type("SUBSYSTEM"));

        // Add another
        assert!(model.add_element_type("component"));

        // Check unused
        let unused = model.get_unused_element_types(&["SUBSYSTEM"]);
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0], "COMPONENT");

        // Remove
        assert!(model.remove_element_type("component"));
        assert!(!model.has_element_type("COMPONENT"));
    }
}
