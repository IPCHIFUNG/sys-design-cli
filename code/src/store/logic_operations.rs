use crate::model::logic::concept::{Component, Interface, LogicConceptDiagram, Module, Subsystem};
use crate::utils::error::{AppError, Result};

/// Operations for modifying LogicConceptDiagram
pub struct LogicOperations;

impl LogicOperations {
    // ==================== Component Operations ====================

    /// Add a component to the system
    pub fn add_component(
        diagram: &mut LogicConceptDiagram,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        Self::check_id_uniqueness(diagram, id)?;

        diagram.system.components.push(Component {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: description.map(|s| s.to_string()),
            modules: Vec::new(),
            exposed_interfaces: Vec::new(),
        });
        diagram.touch();
        Ok(())
    }

    /// Remove a component from the system
    pub fn remove_component(diagram: &mut LogicConceptDiagram, id: &str) -> Result<()> {
        let idx = diagram
            .system
            .components
            .iter()
            .position(|c| c.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("component: {}", id)))?;

        diagram.system.components.remove(idx);
        diagram.touch();
        Ok(())
    }

    // ==================== Subsystem Operations ====================

    /// Add a subsystem to the system
    pub fn add_subsystem(
        diagram: &mut LogicConceptDiagram,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        Self::check_id_uniqueness(diagram, id)?;

        diagram.system.subsystems.push(Subsystem {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: description.map(|s| s.to_string()),
            components: Vec::new(),
        });
        diagram.touch();
        Ok(())
    }

    /// Remove a subsystem from the system
    pub fn remove_subsystem(diagram: &mut LogicConceptDiagram, id: &str) -> Result<()> {
        let idx = diagram
            .system
            .subsystems
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("subsystem: {}", id)))?;

        diagram.system.subsystems.remove(idx);
        diagram.touch();
        Ok(())
    }

    // ==================== Module Operations ====================

    /// Add a module to a component
    pub fn add_module(
        diagram: &mut LogicConceptDiagram,
        component_id: &str,
        module_id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        Self::check_id_uniqueness(diagram, module_id)?;

        let component = Self::find_component_mut(diagram, component_id)?;

        component.modules.push(Module {
            id: module_id.to_string(),
            name: name.unwrap_or(module_id).to_string(),
            description: description.map(|s| s.to_string()),
            interfaces: Vec::new(),
            dependencies: Vec::new(),
            modules: Vec::new(),
        });
        diagram.touch();
        Ok(())
    }

    /// Remove a module from a component
    pub fn remove_module(
        diagram: &mut LogicConceptDiagram,
        component_id: &str,
        module_id: &str,
    ) -> Result<()> {
        let component = Self::find_component_mut(diagram, component_id)?;

        let idx = component
            .modules
            .iter()
            .position(|m| m.id == module_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("module: {}", module_id)))?;

        component.modules.remove(idx);
        diagram.touch();
        Ok(())
    }

    /// Add a nested module to another module
    pub fn add_nested_module(
        diagram: &mut LogicConceptDiagram,
        component_id: &str,
        parent_module_id: &str,
        module_id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        Self::check_id_uniqueness(diagram, module_id)?;

        let component = Self::find_component_mut(diagram, component_id)?;
        let parent = Self::find_module_mut(&mut component.modules, parent_module_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("parent module: {}", parent_module_id)))?;

        parent.modules.push(Module {
            id: module_id.to_string(),
            name: name.unwrap_or(module_id).to_string(),
            description: description.map(|s| s.to_string()),
            interfaces: Vec::new(),
            dependencies: Vec::new(),
            modules: Vec::new(),
        });
        diagram.touch();
        Ok(())
    }

    // ==================== Interface Operations ====================

    /// Add an interface to a module
    pub fn add_interface(
        diagram: &mut LogicConceptDiagram,
        component_id: &str,
        module_id: &str,
        interface_id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        Self::check_interface_id_uniqueness(diagram, interface_id)?;

        let component = Self::find_component_mut(diagram, component_id)?;
        let module = Self::find_module_mut(&mut component.modules, module_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("module: {}", module_id)))?;

        module.interfaces.push(Interface {
            id: interface_id.to_string(),
            name: name.unwrap_or(interface_id).to_string(),
            description: description.map(|s| s.to_string()),
        });
        diagram.touch();
        Ok(())
    }

    /// Remove an interface from a module
    pub fn remove_interface(
        diagram: &mut LogicConceptDiagram,
        component_id: &str,
        module_id: &str,
        interface_id: &str,
    ) -> Result<()> {
        let component = Self::find_component_mut(diagram, component_id)?;
        let module = Self::find_module_mut(&mut component.modules, module_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("module: {}", module_id)))?;

        let idx = module
            .interfaces
            .iter()
            .position(|i| i.id == interface_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("interface: {}", interface_id)))?;

        module.interfaces.remove(idx);

        // Also remove from exposed_interfaces if present
        if let Some(comp) = diagram.system.components.iter_mut().find(|c| c.id == component_id) {
            comp.exposed_interfaces.retain(|id| id != interface_id);
        }

        diagram.touch();
        Ok(())
    }

    /// Expose an interface (add to component's exposed_interfaces)
    pub fn expose_interface(
        diagram: &mut LogicConceptDiagram,
        component_id: &str,
        interface_id: &str,
    ) -> Result<()> {
        let component = Self::find_component_mut(diagram, component_id)?;

        // Verify interface exists in the component
        let interface_exists = component
            .modules
            .iter()
            .any(|m| Self::module_has_interface(m, interface_id));

        if !interface_exists {
            return Err(AppError::ElementNotFound(format!(
                "interface {} in component {}",
                interface_id, component_id
            )));
        }

        if !component.exposed_interfaces.contains(&interface_id.to_string()) {
            component.exposed_interfaces.push(interface_id.to_string());
        }

        diagram.touch();
        Ok(())
    }

    fn module_has_interface(module: &Module, interface_id: &str) -> bool {
        module.interfaces.iter().any(|i| i.id == interface_id)
            || module.modules.iter().any(|m| Self::module_has_interface(m, interface_id))
    }

    // ==================== Dependency Operations ====================

    /// Add a dependency (module uses interface)
    pub fn add_dependency(
        diagram: &mut LogicConceptDiagram,
        component_id: &str,
        module_id: &str,
        interface_id: &str,
    ) -> Result<()> {
        // Verify interface exists
        if diagram.find_interface(interface_id).is_none() {
            return Err(AppError::ElementNotFound(format!(
                "interface: {}",
                interface_id
            )));
        }

        let component = Self::find_component_mut(diagram, component_id)?;
        let module = Self::find_module_mut(&mut component.modules, module_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("module: {}", module_id)))?;

        if !module.dependencies.contains(&interface_id.to_string()) {
            module.dependencies.push(interface_id.to_string());
        }

        diagram.touch();
        Ok(())
    }

    /// Remove a dependency
    pub fn remove_dependency(
        diagram: &mut LogicConceptDiagram,
        component_id: &str,
        module_id: &str,
        interface_id: &str,
    ) -> Result<()> {
        let component = Self::find_component_mut(diagram, component_id)?;
        let module = Self::find_module_mut(&mut component.modules, module_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("module: {}", module_id)))?;

        module
            .dependencies
            .retain(|id| id != interface_id);

        diagram.touch();
        Ok(())
    }

    // ==================== Helper Functions ====================

    fn check_id_uniqueness(diagram: &LogicConceptDiagram, id: &str) -> Result<()> {
        let ids = diagram.all_element_ids();
        if ids.contains(&id) {
            return Err(AppError::ElementAlreadyExists(format!("element with id: {}", id)));
        }
        Ok(())
    }

    fn check_interface_id_uniqueness(diagram: &LogicConceptDiagram, id: &str) -> Result<()> {
        let ids = diagram.all_interface_ids();
        if ids.contains(&id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "interface with id: {}",
                id
            )));
        }
        Ok(())
    }

    fn find_component_mut<'a>(
        diagram: &'a mut LogicConceptDiagram,
        component_id: &str,
    ) -> Result<&'a mut Component> {
        // Search in system.components
        if let Some(comp) = diagram.system.components.iter_mut().find(|c| c.id == component_id) {
            return Ok(comp);
        }
        // Search in subsystems
        for sub in &mut diagram.system.subsystems {
            if let Some(comp) = sub.components.iter_mut().find(|c| c.id == component_id) {
                return Ok(comp);
            }
        }
        Err(AppError::ElementNotFound(format!(
            "component: {}",
            component_id
        )))
    }

    fn find_module_mut<'a>(modules: &'a mut [Module], module_id: &str) -> Option<&'a mut Module> {
        Self::find_module_mut_recursive(modules, module_id)
    }

    fn find_module_mut_recursive<'a>(
        modules: &'a mut [Module],
        target_id: &str,
    ) -> Option<&'a mut Module> {
        // Use split_first_mut to avoid borrow checker issues
        let (first, rest) = modules.split_first_mut()?;

        // Check if first matches
        if first.id == target_id {
            return Some(first);
        }

        // Search nested modules of first
        if let Some(found) = Self::find_module_mut_recursive(&mut first.modules, target_id) {
            return Some(found);
        }

        // Search rest of the array
        Self::find_module_mut_recursive(rest, target_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_component() {
        let mut diagram = LogicConceptDiagram::new("TEST_SYS", "Test System");
        LogicOperations::add_component(&mut diagram, "COMP_1", Some("Component 1"), None).unwrap();
        assert_eq!(diagram.system.components.len(), 1);
        assert_eq!(diagram.system.components[0].id, "COMP_1");
    }

    #[test]
    fn test_add_duplicate_component() {
        let mut diagram = LogicConceptDiagram::new("TEST_SYS", "Test System");
        LogicOperations::add_component(&mut diagram, "COMP_1", None, None).unwrap();
        let result = LogicOperations::add_component(&mut diagram, "COMP_1", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_module_and_nested() {
        let mut diagram = LogicConceptDiagram::new("TEST_SYS", "Test System");
        LogicOperations::add_component(&mut diagram, "COMP", None, None).unwrap();
        LogicOperations::add_module(&mut diagram, "COMP", "MOD_1", None, None).unwrap();
        LogicOperations::add_nested_module(&mut diagram, "COMP", "MOD_1", "MOD_2", None, None).unwrap();

        assert_eq!(diagram.system.components[0].modules.len(), 1);
        assert_eq!(diagram.system.components[0].modules[0].modules.len(), 1);
    }

    #[test]
    fn test_add_interface_and_dependency() {
        let mut diagram = LogicConceptDiagram::new("TEST_SYS", "Test System");
        LogicOperations::add_component(&mut diagram, "COMP", None, None).unwrap();
        LogicOperations::add_module(&mut diagram, "COMP", "MOD_1", None, None).unwrap();
        LogicOperations::add_interface(
            &mut diagram,
            "COMP",
            "MOD_1",
            "ITF_API",
            Some("API Interface"),
            None,
        ).unwrap();

        assert_eq!(
            diagram.system.components[0].modules[0].interfaces.len(),
            1
        );
    }
}
