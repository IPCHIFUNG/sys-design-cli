use crate::model::logic::concept::{Component, LogicConceptDiagram, Module, Interface};
use crate::model::workspace::Workspace;
use std::collections::{HashMap, HashSet};

/// Generate PlantUML for Logical Architecture Concept Model Diagram
pub fn generate_logic_concept_plantuml(diagram: &LogicConceptDiagram) -> String {
    generate_logic_concept_plantuml_with_root(diagram, None)
}

/// Generate PlantUML with optional root element filter
pub fn generate_logic_concept_plantuml_with_root(
    diagram: &LogicConceptDiagram,
    root: Option<&str>,
) -> String {
    generate_logic_concept_plantuml_with_workspace(None, diagram, root)
}

/// Generate PlantUML with workspace access (for context diagram interfaces)
pub fn generate_logic_concept_plantuml_with_workspace(
    workspace: Option<&Workspace>,
    diagram: &LogicConceptDiagram,
    root: Option<&str>,
) -> String {
    let mut output = String::new();

    // Header
    output.push_str("@startuml\n\n");
    output.push_str("top to bottom direction\n");
    output.push_str("skinparam defaultTextAlignment center\n\n");

    // Build containment tree
    let containment_tree = build_containment_tree(diagram);

    match root {
        Some(root_id) => {
            // Check if element exists
            if element_exists(diagram, root_id) {
                generate_element_subtree(
                    &mut output,
                    workspace,
                    diagram,
                    root_id,
                    &containment_tree,
                );
            } else {
                // Element not found - output error message
                output.push_str(&format!(
                    "' ERROR: Element '{}' not found in diagram\n",
                    root_id
                ));
                output.push_str(&format!(
                    "' Available elements: {}\n",
                    get_all_element_ids(diagram).join(", ")
                ));
            }
        }
        None => {
            // Generate full diagram
            generate_full_diagram(&mut output, diagram, &containment_tree);
        }
    }

    // Footer
    output.push_str("\n@enduml\n");

    output
}

/// Check if an element exists in the diagram
fn element_exists(diagram: &LogicConceptDiagram, id: &str) -> bool {
    // Check modules
    if find_module_in_diagram(diagram, id).is_some() {
        return true;
    }
    // Check interfaces
    if find_interface_in_diagram(diagram, id).is_some() {
        return true;
    }
    // Check components
    if find_component_in_diagram(diagram, id).is_some() {
        return true;
    }
    // Check subsystems
    if find_subsystem_in_diagram(diagram, id).is_some() {
        return true;
    }
    // Check system
    if diagram.system.id == id {
        return true;
    }
    false
}

/// Get all element IDs for error messages
fn get_all_element_ids(diagram: &LogicConceptDiagram) -> Vec<&str> {
    let mut ids = Vec::new();

    // System level modules
    for module in &diagram.system.modules {
        ids.push(module.id.as_str());
    }

    // Components
    for comp in &diagram.system.components {
        ids.push(comp.id.as_str());
    }

    // Subsystems
    for sub in &diagram.system.subsystems {
        ids.push(sub.id.as_str());
    }

    // Interfaces
    for iface in &diagram.system.interfaces {
        ids.push(iface.id.as_str());
    }

    ids
}

/// Build a tree of parent -> children relationships from containments
fn build_containment_tree(diagram: &LogicConceptDiagram) -> HashMap<String, Vec<String>> {
    let mut tree: HashMap<String, Vec<String>> = HashMap::new();
    for containment in &diagram.containments {
        tree.entry(containment.parent_id.clone())
            .or_default()
            .push(containment.child_id.clone());
    }
    tree
}

/// Generate element subtree with only one level of children, interfaces, and dependencies
fn generate_element_subtree(
    output: &mut String,
    workspace: Option<&Workspace>,
    diagram: &LogicConceptDiagram,
    root_id: &str,
    containment_tree: &HashMap<String, Vec<String>>,
) {
    // Check if root is the system element
    let is_system_root = diagram.system.id == root_id;

    // Collect only immediate children (one level only)
    let mut subtree_elements = HashSet::new();
    subtree_elements.insert(root_id.to_string());

    // Add only immediate children (one level)
    if let Some(children) = containment_tree.get(root_id) {
        for child_id in children {
            subtree_elements.insert(child_id.clone());
        }
    }

    // Get root interfaces
    let root_interfaces: Vec<String> = if is_system_root {
        // For SYSTEM root, get interfaces from context diagram
        get_context_diagram_interfaces(workspace, root_id)
    } else {
        // For other elements, get from logic view provide relations
        find_provided_interface(diagram, root_id).into_iter().collect()
    };

    // Add root interfaces to subtree_elements
    for iface_id in &root_interfaces {
        subtree_elements.insert(iface_id.clone());
    }

    // Add provided interfaces for immediate child elements only
    for element_id in subtree_elements.clone().iter() {
        if let Some(iface_id) = find_provided_interface(diagram, element_id) {
            subtree_elements.insert(iface_id);
        }
    }

    // For one-level expansion, only add immediate child interfaces through containment
    // (not recursive - just one level)
    for iface_id in &root_interfaces {
        // Find direct child interfaces of this root interface
        if let Some(iface_children) = containment_tree.get(iface_id) {
            for child_iface_id in iface_children {
                if find_interface_in_diagram(diagram, child_iface_id).is_some() {
                    subtree_elements.insert(child_iface_id.clone());
                }
            }
        }
    }

    // Generate root interfaces outside the rectangle
    for iface_id in &root_interfaces {
        output.push_str(&format!("interface {}\n", iface_id));
    }
    if !root_interfaces.is_empty() {
        output.push('\n');
    }

    // Generate the root element rectangle with one level of children
    if is_system_root {
        generate_system_rectangle(output, diagram, root_id, containment_tree, &subtree_elements);
    } else if let Some(module) = find_module_in_diagram(diagram, root_id) {
        generate_module_rectangle(output, diagram, module, containment_tree, &subtree_elements);
    } else if let Some(component) = find_component_in_diagram(diagram, root_id) {
        generate_component_rectangle(output, diagram, component, containment_tree, &subtree_elements);
    } else if let Some(subsystem) = find_subsystem_in_diagram(diagram, root_id) {
        generate_subsystem_rectangle(output, diagram, subsystem, containment_tree, &subtree_elements);
    }

    // Generate interface containment relationships
    generate_interface_containments(output, diagram, &subtree_elements, containment_tree, &root_interfaces);

    // Generate provide relations (interface --- module)
    generate_provide_relations(output, diagram, &subtree_elements, &root_interfaces, is_system_root);

    // Generate dependencies (only for immediate children)
    generate_dependencies(output, diagram, &subtree_elements);
}

/// Get interfaces from context diagram for a system element
fn get_context_diagram_interfaces(workspace: Option<&Workspace>, system_id: &str) -> Vec<String> {
    let mut interfaces = Vec::new();

    if let Some(ws) = workspace {
        if let Some(context) = &ws.context_diagram {
            // Get interfaces that the system provides
            for provider in &context.interface_providers {
                if provider.system == system_id {
                    // Add all interfaces from this provider
                    for iface_id in &provider.interfaces {
                        interfaces.push(iface_id.clone());
                    }
                }
            }
        }
    }

    interfaces
}

/// Generate a system rectangle with one level of children
fn generate_system_rectangle(
    output: &mut String,
    diagram: &LogicConceptDiagram,
    system_id: &str,
    containment_tree: &HashMap<String, Vec<String>>,
    subtree_elements: &HashSet<String>,
) {
    // Start system rectangle
    output.push_str(&format!(
        "rectangle \"<<SYSTEM>>\\n{}\" as {} {{\n\n",
        system_id, system_id
    ));

    // Get immediate children only
    if let Some(children) = containment_tree.get(system_id) {
        // Generate interfaces for child elements first
        for child_id in children {
            if subtree_elements.contains(child_id) {
                if let Some(child_interface) = find_provided_interface(diagram, child_id) {
                    if subtree_elements.contains(&child_interface) {
                        output.push_str(&format!("    interface {}\n", child_interface));
                    }
                }
            }
        }

        output.push('\n');

        // Generate child elements
        for child_id in children {
            if subtree_elements.contains(child_id) {
                // Determine element type
                if let Some(subsystem) = find_subsystem_in_diagram(diagram, child_id) {
                    output.push_str(&format!(
                        "    rectangle \"<<SUBSYSTEM>>\\n{}\" as {}\n",
                        subsystem.id, subsystem.id
                    ));
                } else if let Some(component) = find_component_in_diagram(diagram, child_id) {
                    output.push_str(&format!(
                        "    rectangle \"<<COMPONENT>>\\n{}\" as {}\n",
                        component.id, component.id
                    ));
                } else if let Some(module) = find_module_in_diagram(diagram, child_id) {
                    let stereotype = if diagram.submodule_ids.contains(&module.id) {
                        "SUBMODULE"
                    } else {
                        "MODULE"
                    };
                    output.push_str(&format!(
                        "    rectangle \"<<{}>>\\n{}\" as {}\n",
                        stereotype, module.id, module.id
                    ));
                }
            }
        }
    }

    output.push_str("}\n\n");
}

/// Find the interface provided by an element
fn find_provided_interface(diagram: &LogicConceptDiagram, element_id: &str) -> Option<String> {
    for rel in &diagram.provide_relations {
        if rel.element_id == element_id {
            return Some(rel.interface_id.clone());
        }
    }
    None
}

/// Generate a module rectangle with nested elements
fn generate_module_rectangle(
    output: &mut String,
    diagram: &LogicConceptDiagram,
    module: &Module,
    containment_tree: &HashMap<String, Vec<String>>,
    subtree_elements: &HashSet<String>,
) {
    // Determine element type stereotype
    let stereotype = if diagram.submodule_ids.contains(&module.id) {
        "SUBMODULE"
    } else {
        "MODULE"
    };

    // Start module rectangle - use id as display name
    output.push_str(&format!(
        "rectangle \"<<{}>>\\n{}\" as {} {{\n\n",
        stereotype, module.id, module.id
    ));

    // Get child modules from containment tree
    let child_modules: Vec<&str> = containment_tree
        .get(&module.id)
        .map(|children| {
            children
                .iter()
                .filter(|id| find_module_in_diagram(diagram, id).is_some())
                .map(|s| s.as_str())
                .collect()
        })
        .unwrap_or_default();

    // Generate interfaces for child modules (inside the rectangle, before child modules)
    for child_id in &child_modules {
        if subtree_elements.contains(*child_id) {
            if let Some(child_interface) = find_provided_interface(diagram, child_id) {
                if subtree_elements.contains(&child_interface) {
                    if let Some(iface) = find_interface_in_diagram(diagram, &child_interface) {
                        output.push_str(&format!("    interface {}\n", iface.id));
                    }
                }
            }
        }
    }

    output.push('\n');

    // Generate child modules
    for child_id in &child_modules {
        if subtree_elements.contains(*child_id) {
            if let Some(child_module) = find_module_in_diagram(diagram, child_id) {
                // Determine child element type stereotype
                let child_stereotype = if diagram.submodule_ids.contains(&child_module.id) {
                    "SUBMODULE"
                } else {
                    "MODULE"
                };
                output.push_str(&format!(
                    "    rectangle \"<<{}>>\\n{}\" as {}\n",
                    child_stereotype, child_module.id, child_module.id
                ));
            }
        }
    }

    output.push_str("}\n\n");
}

/// Generate interface containment relationships (*..)
fn generate_interface_containments(
    output: &mut String,
    diagram: &LogicConceptDiagram,
    subtree_elements: &HashSet<String>,
    _containment_tree: &HashMap<String, Vec<String>>,
    root_interfaces: &[String],
) {
    // Helper function to check if an ID is an interface
    // Checks both logic view interfaces and root interfaces (from context diagram)
    let is_interface = |id: &str| -> bool {
        find_interface_in_diagram(diagram, id).is_some() || root_interfaces.contains(&id.to_string())
    };

    // Check all containments in the diagram for interface-to-interface relationships
    for containment in &diagram.containments {
        // Check if both parent and child are interfaces
        let parent_is_interface = is_interface(&containment.parent_id);
        let child_is_interface = is_interface(&containment.child_id);

        if parent_is_interface && child_is_interface {
            // Only generate if both are in the subtree
            if subtree_elements.contains(&containment.parent_id)
                && subtree_elements.contains(&containment.child_id)
            {
                output.push_str(&format!("{} *.. {}\n\n", containment.parent_id, containment.child_id));
            }
        }
    }
}

/// Generate provide relations (interface --- module)
fn generate_provide_relations(
    output: &mut String,
    diagram: &LogicConceptDiagram,
    subtree_elements: &HashSet<String>,
    root_interfaces: &[String],
    is_system_root: bool,
) {
    // For system root, generate provide relations from context diagram interfaces
    if is_system_root {
        for iface_id in root_interfaces {
            output.push_str(&format!("{} --- {}\n", iface_id, diagram.system.id));
        }
    }

    // Generate provide relations for child elements
    for rel in &diagram.provide_relations {
        if subtree_elements.contains(&rel.element_id) && rel.element_id != diagram.system.id {
            output.push_str(&format!(
                "{} --- {}\n",
                rel.interface_id, rel.element_id
            ));
        }
    }
}

/// Generate dependencies (..>)
fn generate_dependencies(
    output: &mut String,
    diagram: &LogicConceptDiagram,
    subtree_elements: &HashSet<String>,
) {
    // Generate dependencies from diagram-level dependencies
    for dep in &diagram.dependencies {
        if subtree_elements.contains(&dep.from) {
            output.push_str(&format!(
                "{} ..> {}\n",
                dep.from, dep.to
            ));
        }
    }

    // Generate dependencies from module-level dependencies
    // Find all modules in subtree and output their dependencies
    for element_id in subtree_elements {
        if let Some(module) = find_module_in_diagram(diagram, element_id) {
            for dep_interface_id in &module.dependencies {
                output.push_str(&format!(
                    "{} ..> {}\n",
                    module.id, dep_interface_id
                ));
            }
        }
    }
}

/// Generate a component rectangle
fn generate_component_rectangle(
    output: &mut String,
    diagram: &LogicConceptDiagram,
    component: &Component,
    containment_tree: &HashMap<String, Vec<String>>,
    subtree_elements: &HashSet<String>,
) {
    output.push_str(&format!(
        "rectangle \"<<COMPONENT>>\\n{}\" as {} {{\n\n",
        component.id, component.id
    ));

    // Get child modules from containment tree
    if let Some(children) = containment_tree.get(&component.id) {
        // Generate interfaces for child modules (inside the rectangle)
        for child_id in children {
            if subtree_elements.contains(child_id) {
                if let Some(child_interface) = find_provided_interface(diagram, child_id) {
                    if subtree_elements.contains(&child_interface) {
                        output.push_str(&format!("    interface {}\n", child_interface));
                    }
                }
            }
        }

        output.push('\n');

        // Generate child modules
        for child_id in children {
            if subtree_elements.contains(child_id) {
                if let Some(child_module) = find_module_in_diagram(diagram, child_id) {
                    let stereotype = if diagram.submodule_ids.contains(&child_module.id) {
                        "SUBMODULE"
                    } else {
                        "MODULE"
                    };
                    output.push_str(&format!(
                        "    rectangle \"<<{}>>\\n{}\" as {}\n",
                        stereotype, child_module.id, child_module.id
                    ));
                }
            }
        }
    }

    output.push_str("}\n\n");
}

/// Generate a subsystem rectangle
fn generate_subsystem_rectangle(
    output: &mut String,
    diagram: &LogicConceptDiagram,
    subsystem: &crate::model::logic::concept::Subsystem,
    containment_tree: &HashMap<String, Vec<String>>,
    subtree_elements: &HashSet<String>,
) {
    output.push_str(&format!(
        "rectangle \"<<SUBSYSTEM>>\\n{}\" as {} {{\n\n",
        subsystem.id, subsystem.id
    ));

    // Get child components from containment tree
    if let Some(children) = containment_tree.get(&subsystem.id) {
        // Generate interfaces for child components (inside the rectangle)
        for child_id in children {
            if subtree_elements.contains(child_id) {
                if let Some(child_interface) = find_provided_interface(diagram, child_id) {
                    if subtree_elements.contains(&child_interface) {
                        output.push_str(&format!("    interface {}\n", child_interface));
                    }
                }
            }
        }

        output.push('\n');

        // Generate child components
        for child_id in children {
            if subtree_elements.contains(child_id) {
                if let Some(child_comp) = find_component_in_diagram(diagram, child_id) {
                    output.push_str(&format!(
                        "    rectangle \"<<COMPONENT>>\\n{}\" as {}\n",
                        child_comp.id, child_comp.id
                    ));
                }
            }
        }
    }

    output.push_str("}\n\n");
}

/// Collect all IDs in a subtree
fn collect_subtree_ids(
    root: &str,
    tree: &HashMap<String, Vec<String>>,
    ids: &mut HashSet<String>,
) {
    if let Some(children) = tree.get(root) {
        for child in children {
            ids.insert(child.clone());
            collect_subtree_ids(child, tree, ids);
        }
    }
}

/// Generate the full diagram
fn generate_full_diagram(
    output: &mut String,
    diagram: &LogicConceptDiagram,
    containment_tree: &HashMap<String, Vec<String>>,
) {
    let mut all_elements = HashSet::new();
    all_elements.insert(diagram.system.id.clone());
    collect_subtree_ids(&diagram.system.id, containment_tree, &mut all_elements);

    // Process each component
    for comp in &diagram.system.components {
        generate_component_rectangle(output, diagram, comp, containment_tree, &all_elements);
    }

    // Process subsystems
    for sub in &diagram.system.subsystems {
        generate_subsystem_rectangle(output, diagram, sub, containment_tree, &all_elements);
    }

    // Process standalone modules at system level
    for module in &diagram.system.modules {
        generate_module_rectangle(output, diagram, module, containment_tree, &all_elements);
    }

    // Generate all relationships
    generate_interface_containments(output, diagram, &all_elements, containment_tree, &[]);
    generate_provide_relations(output, diagram, &all_elements, &[], false);
    generate_dependencies(output, diagram, &all_elements);
}

/// Find a module by ID in the diagram
fn find_module_in_diagram<'a>(
    diagram: &'a LogicConceptDiagram,
    id: &str,
) -> Option<&'a Module> {
    // Check modules at system level
    for module in &diagram.system.modules {
        if module.id == id {
            return Some(module);
        }
        if let Some(found) = find_module_recursive(module, id) {
            return Some(found);
        }
    }
    // Check modules in components
    for comp in &diagram.system.components {
        for module in &comp.modules {
            if module.id == id {
                return Some(module);
            }
            if let Some(found) = find_module_recursive(module, id) {
                return Some(found);
            }
        }
    }
    // Check modules in subsystems
    for sub in &diagram.system.subsystems {
        for comp in &sub.components {
            for module in &comp.modules {
                if module.id == id {
                    return Some(module);
                }
                if let Some(found) = find_module_recursive(module, id) {
                    return Some(found);
                }
            }
        }
    }
    None
}

fn find_module_recursive<'a>(module: &'a Module, id: &str) -> Option<&'a Module> {
    for m in &module.modules {
        if m.id == id {
            return Some(m);
        }
        if let Some(found) = find_module_recursive(m, id) {
            return Some(found);
        }
    }
    None
}

/// Find an interface by ID in the diagram
fn find_interface_in_diagram<'a>(
    diagram: &'a LogicConceptDiagram,
    id: &str,
) -> Option<&'a Interface> {
    // Check interfaces at system level
    for iface in &diagram.system.interfaces {
        if iface.id == id {
            return Some(iface);
        }
    }
    // Check interfaces in modules
    for module in &diagram.system.modules {
        if let Some(found) = find_interface_in_module(module, id) {
            return Some(found);
        }
    }
    None
}

fn find_interface_in_module<'a>(module: &'a Module, id: &str) -> Option<&'a Interface> {
    for iface in &module.interfaces {
        if iface.id == id {
            return Some(iface);
        }
    }
    for m in &module.modules {
        if let Some(found) = find_interface_in_module(m, id) {
            return Some(found);
        }
    }
    None
}

/// Find a component by ID in the diagram
fn find_component_in_diagram<'a>(
    diagram: &'a LogicConceptDiagram,
    id: &str,
) -> Option<&'a Component> {
    for comp in &diagram.system.components {
        if comp.id == id {
            return Some(comp);
        }
    }
    for sub in &diagram.system.subsystems {
        for comp in &sub.components {
            if comp.id == id {
                return Some(comp);
            }
        }
    }
    None
}

/// Find a subsystem by ID in the diagram
fn find_subsystem_in_diagram<'a>(
    diagram: &'a LogicConceptDiagram,
    id: &str,
) -> Option<&'a crate::model::logic::concept::Subsystem> {
    diagram.system.subsystems.iter().find(|sub| sub.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_logic_concept_plantuml() {
        let diagram = LogicConceptDiagram::new("MY_SYSTEM", "My System");

        let output = generate_logic_concept_plantuml(&diagram);

        assert!(output.contains("@startuml"));
        assert!(output.contains("@enduml"));
        assert!(output.contains("top to bottom direction"));
        assert!(output.contains("skinparam defaultTextAlignment center"));
    }
}
