use crate::model::logic::concept::{Component, LogicConceptDiagram, Module};

/// Generate PlantUML for Logical Architecture Concept Model Diagram
pub fn generate_logic_concept_plantuml(diagram: &LogicConceptDiagram) -> String {
    let mut output = String::new();

    // Header
    output.push_str("@startuml\n\n");
    output.push_str("top to bottom direction\n");
    output.push_str("skinparam defaultTextAlignment center\n\n");

    // Process each component as a module container
    for comp in &diagram.system.components {
        generate_component(&mut output, comp);
    }

    // Process subsystems
    for sub in &diagram.system.subsystems {
        for comp in &sub.components {
            generate_component(&mut output, comp);
        }
    }

    // Generate global dependencies
    generate_global_dependencies(&mut output, diagram);

    // Footer
    output.push_str("\n@enduml\n");

    output
}

fn generate_component(output: &mut String, component: &Component) {
    // Start module rectangle
    output.push_str(&format!(
        "rectangle \"<<COMPONENT>>\\n{}\" as {} {{\n\n",
        component.name, component.id
    ));

    // Generate modules
    for module in &component.modules {
        generate_module(output, module, 1);
    }

    // Generate module dependencies
    generate_module_dependencies(output, component);

    // Close component rectangle
    output.push_str("}\n\n");

    // Generate exposed interfaces
    for iface_id in &component.exposed_interfaces {
        output.push_str(&format!("{} *.. {}\n", iface_id, find_first_interface_owner(component, iface_id)));
    }
}

fn generate_module(output: &mut String, module: &Module, level: usize) {
    let indent = "    ".repeat(level);

    // Generate interfaces first
    for iface in &module.interfaces {
        output.push_str(&format!(
            "{}interface {}\n",
            indent, iface.id
        ));
    }

    // Generate module rectangle
    output.push_str(&format!(
        "{}rectangle \"<<MODULE>>\\n{}\" as {}\n",
        indent, module.name, module.id
    ));

    // Link interfaces to module
    for iface in &module.interfaces {
        output.push_str(&format!(
            "{}{} --- {}\n",
            indent, iface.id, module.id
        ));
    }

    // Generate nested modules
    for m in &module.modules {
        generate_module(output, m, level + 1);
    }
}

fn generate_module_dependencies(output: &mut String, component: &Component) {
    // Collect all dependencies within the component
    for module in &component.modules {
        generate_module_deps_recursive(output, module);
    }
}

fn generate_module_deps_recursive(output: &mut String, module: &Module) {
    for dep_id in &module.dependencies {
        output.push_str(&format!(
            "    {} ..> {}\n",
            module.id, dep_id
        ));
    }

    // Process nested modules
    for m in &module.modules {
        generate_module_deps_recursive(output, m);
    }
}

fn generate_global_dependencies(output: &mut String, diagram: &LogicConceptDiagram) {
    // Generate dependencies at diagram level (cross-component)
    for dep in &diagram.dependencies {
        output.push_str(&format!(
            "{} ..> {}\n",
            dep.from, dep.to
        ));
    }
}

fn find_first_interface_owner(component: &Component, interface_id: &str) -> String {
    for module in &component.modules {
        if let Some(owner) = find_interface_owner_in_module(module, interface_id) {
            return owner;
        }
    }
    interface_id.to_string()
}

fn find_interface_owner_in_module(module: &Module, interface_id: &str) -> Option<String> {
    if module.interfaces.iter().any(|i| i.id == interface_id) {
        return Some(module.id.clone());
    }
    for m in &module.modules {
        if let Some(owner) = find_interface_owner_in_module(m, interface_id) {
            return Some(owner);
        }
    }
    None
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
