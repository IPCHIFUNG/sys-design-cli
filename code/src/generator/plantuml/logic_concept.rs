use crate::model::logic::concept::{Component, LogicConceptDiagram, Module, Interface};
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
                generate_element_subtree(&mut output, diagram, root_id, &containment_tree);
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

/// Generate element subtree with all children, interfaces, and dependencies
fn generate_element_subtree(
    output: &mut String,
    diagram: &LogicConceptDiagram,
    root_id: &str,
    containment_tree: &HashMap<String, Vec<String>>,
) {
    // Collect all elements in subtree
    let mut subtree_elements = HashSet::new();
    subtree_elements.insert(root_id.to_string());
    collect_subtree_ids(root_id, containment_tree, &mut subtree_elements);

    // Find the root module's provided interface (should be outside the rectangle)
    let root_interface = find_provided_interface(diagram, root_id);
    if let Some(iface_id) = &root_interface {
        // Add root interface to subtree_elements for containment generation
        subtree_elements.insert(iface_id.clone());
    }

    // Add provided interfaces for all child elements in subtree
    for element_id in subtree_elements.clone().iter() {
        if let Some(iface_id) = find_provided_interface(diagram, element_id) {
            subtree_elements.insert(iface_id);
        }
    }

    // Collect all interface IDs from containment relations into subtree_elements
    // This ensures child interfaces are included in the set
    // Need to iterate multiple times to collect all nested interface children
    let mut changed = true;
    while changed {
        changed = false;
        for containment in &diagram.containments {
            // Check if both parent and child are interfaces
            if find_interface_in_diagram(diagram, &containment.parent_id).is_some()
                && find_interface_in_diagram(diagram, &containment.child_id).is_some()
            {
                // If the parent interface is in subtree, add child interface
                if subtree_elements.contains(&containment.parent_id)
                    && !subtree_elements.contains(&containment.child_id)
                {
                    subtree_elements.insert(containment.child_id.clone());
                    changed = true;
                }
            }
        }
    }

    // Generate root interface outside the rectangle
    if let Some(iface_id) = &root_interface {
        if let Some(iface) = find_interface_in_diagram(diagram, iface_id) {
            output.push_str(&format!("interface {}\n\n", iface.id));
        }
    }

    // Determine element type and generate accordingly
    if let Some(module) = find_module_in_diagram(diagram, root_id) {
        generate_module_rectangle(output, diagram, module, containment_tree, &subtree_elements);
    } else if let Some(component) = find_component_in_diagram(diagram, root_id) {
        generate_component_rectangle(output, diagram, component, containment_tree, &subtree_elements);
    } else if let Some(subsystem) = find_subsystem_in_diagram(diagram, root_id) {
        generate_subsystem_rectangle(output, diagram, subsystem, containment_tree, &subtree_elements);
    }

    // Generate interface containment relationships
    generate_interface_containments(output, diagram, &subtree_elements, containment_tree);

    // Generate provide relations (interface --- module)
    generate_provide_relations(output, diagram, &subtree_elements);

    // Generate dependencies
    generate_dependencies(output, diagram, &subtree_elements);
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
) {
    // Check all containments in the diagram for interface-to-interface relationships
    for containment in &diagram.containments {
        // Check if both parent and child are interfaces
        let parent_is_interface = find_interface_in_diagram(diagram, &containment.parent_id).is_some();
        let child_is_interface = find_interface_in_diagram(diagram, &containment.child_id).is_some();

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
) {
    for rel in &diagram.provide_relations {
        if subtree_elements.contains(&rel.element_id) {
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
    _subtree_elements: &HashSet<String>,
) {
    output.push_str(&format!(
        "rectangle \"<<COMPONENT>>\\n{}\" as {} {{\n\n",
        component.name, component.id
    ));

    // Generate child modules
    if let Some(children) = containment_tree.get(&component.id) {
        for child_id in children {
            if let Some(child_module) = find_module_in_diagram(diagram, child_id) {
                output.push_str(&format!(
                    "    rectangle \"<<MODULE>>\\n{}\" as {}\n",
                    child_module.name, child_module.id
                ));
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
    _subtree_elements: &HashSet<String>,
) {
    output.push_str(&format!(
        "rectangle \"<<SUBSYSTEM>>\\n{}\" as {} {{\n\n",
        subsystem.name, subsystem.id
    ));

    // Generate child components
    if let Some(children) = containment_tree.get(&subsystem.id) {
        for child_id in children {
            if let Some(child_comp) = find_component_in_diagram(diagram, child_id) {
                output.push_str(&format!(
                    "    rectangle \"<<COMPONENT>>\\n{}\" as {}\n",
                    child_comp.name, child_comp.id
                ));
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
    generate_interface_containments(output, diagram, &all_elements, containment_tree);
    generate_provide_relations(output, diagram, &all_elements);
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
