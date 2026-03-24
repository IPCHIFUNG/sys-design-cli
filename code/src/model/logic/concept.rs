use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Root structure for Logic View Diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicConceptDiagram {
    pub version: String,
    pub kind: DiagramKind,
    pub metadata: Metadata,
    pub system: System,
    /// Provide relations: element provides interface
    #[serde(default)]
    pub provide_relations: Vec<ProvideRelation>,
    /// Element containment relationships: parent contains child
    #[serde(default)]
    pub containments: Vec<ElementContainment>,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    /// IDs of elements that are explicitly defined as submodules
    #[serde(default)]
    pub submodule_ids: Vec<String>,
}

impl LogicConceptDiagram {
    /// Create a new LogicConceptDiagram with the given system
    pub fn new(system_id: &str, title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            kind: DiagramKind::LogicView,
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
                modules: Vec::new(),
                interfaces: Vec::new(),
            },
            provide_relations: Vec::new(),
            containments: Vec::new(),
            dependencies: Vec::new(),
            submodule_ids: Vec::new(),
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
        for dep in &self.dependencies {
            ids.push(&dep.from);
            ids.push(&dep.to);
        }
        ids
    }

    /// Find element name by ID
    pub fn get_element_name(&self, id: &str) -> Option<&str> {
        if self.system.id == id {
            return Some(&self.system.name);
        }
        self.system.find_name(id)
    }

    /// Get all interface IDs
    pub fn all_interface_ids(&self) -> Vec<&str> {
        self.system.collect_interface_ids()
    }

    /// Find interface by ID
    pub fn find_interface(&self, id: &str) -> Option<&Interface> {
        self.system.find_interface(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiagramKind {
    LogicView,
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
    Interface,
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
    /// Direct modules under system (for flexible hierarchy)
    #[serde(default)]
    pub modules: Vec<Module>,
    /// Standalone interfaces at system level
    #[serde(default)]
    pub interfaces: Vec<Interface>,
}

impl System {
    fn collect_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        for sub in &self.subsystems {
            sub.collect_ids(ids);
        }
        for comp in &self.components {
            comp.collect_ids(ids);
        }
        for module in &self.modules {
            module.collect_ids(ids);
        }
        for iface in &self.interfaces {
            ids.push(&iface.id);
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
        for module in &self.modules {
            if let Some(name) = module.find_name(id) {
                return Some(name);
            }
        }
        // Check standalone interfaces
        for iface in &self.interfaces {
            if iface.id == id {
                return Some(&iface.name);
            }
        }
        None
    }

    fn collect_interface_ids<'a>(&'a self) -> Vec<&'a str> {
        let mut ids: Vec<&'a str> = Vec::new();
        // Standalone interfaces at system level
        for iface in &self.interfaces {
            ids.push(&iface.id);
        }
        for comp in &self.components {
            comp.collect_interface_ids(&mut ids);
        }
        for sub in &self.subsystems {
            for comp in &sub.components {
                comp.collect_interface_ids(&mut ids);
            }
        }
        for module in &self.modules {
            module.collect_interface_ids(&mut ids);
        }
        ids
    }

    fn find_interface(&self, id: &str) -> Option<&Interface> {
        // Check standalone interfaces first
        for iface in &self.interfaces {
            if iface.id == id {
                return Some(iface);
            }
        }
        for comp in &self.components {
            if let Some(iface) = comp.find_interface(id) {
                return Some(iface);
            }
        }
        for sub in &self.subsystems {
            for comp in &sub.components {
                if let Some(iface) = comp.find_interface(id) {
                    return Some(iface);
                }
            }
        }
        for module in &self.modules {
            if let Some(iface) = module.find_interface(id) {
                return Some(iface);
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

/// Component element (contains modules)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub modules: Vec<Module>,
    /// Exposed interfaces (interfaces that this component exposes to external)
    #[serde(default)]
    pub exposed_interfaces: Vec<String>,
}

impl Component {
    fn collect_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        ids.push(&self.id);
        for module in &self.modules {
            module.collect_ids(ids);
        }
        for iface_id in &self.exposed_interfaces {
            ids.push(iface_id);
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

    fn collect_interface_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        for module in &self.modules {
            module.collect_interface_ids(ids);
        }
        for iface_id in &self.exposed_interfaces {
            ids.push(iface_id);
        }
    }

    fn find_interface(&self, id: &str) -> Option<&Interface> {
        for module in &self.modules {
            if let Some(iface) = module.find_interface(id) {
                return Some(iface);
            }
        }
        None
    }
}

/// Module element (recursive, can contain sub-modules and interfaces)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub interfaces: Vec<Interface>,
    /// IDs of interfaces this module depends on
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Nested modules (recursive)
    #[serde(default)]
    pub modules: Vec<Module>,
}

impl Module {
    fn collect_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        ids.push(&self.id);
        for iface in &self.interfaces {
            ids.push(&iface.id);
        }
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

    fn collect_interface_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        for iface in &self.interfaces {
            ids.push(&iface.id);
        }
        for module in &self.modules {
            module.collect_interface_ids(ids);
        }
    }

    fn find_interface(&self, id: &str) -> Option<&Interface> {
        for iface in &self.interfaces {
            if iface.id == id {
                return Some(iface);
            }
        }
        for module in &self.modules {
            if let Some(iface) = module.find_interface(id) {
                return Some(iface);
            }
        }
        None
    }
}

/// Interface definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Dependency between elements (module uses interface)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Module ID that has the dependency
    pub from: String,
    /// Interface ID that is being used
    pub to: String,
}

/// Provide relation between element and interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvideRelation {
    /// Element ID that provides the interface
    pub element_id: String,
    /// Interface ID being provided
    pub interface_id: String,
}

/// Element containment relationship (parent contains child)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementContainment {
    /// Parent element ID
    pub parent_id: String,
    /// Child element ID
    pub child_id: String,
}

// Backward compatibility aliases
pub type Layer = Module;
pub type Submodule = Module;

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
    fn test_component_with_modules() {
        let mut diagram = LogicConceptDiagram::new("MY_SYSTEM", "My System");

        diagram.system.components.push(Component {
            id: "MOTOR".to_string(),
            name: "Motor Module".to_string(),
            description: None,
            modules: vec![
                Module {
                    id: "APP_LAYER".to_string(),
                    name: "Application Layer".to_string(),
                    description: None,
                    interfaces: vec![],
                    dependencies: vec![],
                    modules: vec![
                        Module {
                            id: "POSITION_LOOP".to_string(),
                            name: "Position Loop".to_string(),
                            description: None,
                            interfaces: vec![Interface {
                                id: "ITF_POSITION_LOOP".to_string(),
                                name: "Position Loop Interface".to_string(),
                                description: None,
                            }],
                            dependencies: vec!["ITF_SPEED_LOOP".to_string()],
                            modules: vec![],
                        },
                    ],
                },
            ],
            exposed_interfaces: vec!["ITF_POSITION_LOOP".to_string()],
        });

        let ids = diagram.all_element_ids();
        assert!(ids.contains(&"MY_SYSTEM"));
        assert!(ids.contains(&"MOTOR"));
        assert!(ids.contains(&"APP_LAYER"));
        assert!(ids.contains(&"POSITION_LOOP"));
        assert!(ids.contains(&"ITF_POSITION_LOOP"));
    }
}
