use crate::model::logic::concept::{Component, LogicConceptDiagram, Module, Submodule, Subsystem};

/// Generate PlantUML for Logical Architecture Concept Model Diagram
pub fn generate_logic_concept_plantuml(diagram: &LogicConceptDiagram) -> String {
    let mut output = String::new();

    // Header
    output.push_str("@startuml\n\n");
    output.push_str("skinparam defaultTextAlignment center\n\n");

    // System rectangle
    output.push_str(&format!("rectangle {}\n", diagram.system.id));

    // Relationships: System contains Subsystems and Components
    for sub in &diagram.system.subsystems {
        output.push_str(&format!("rectangle {}\n", sub.id));
        output.push_str(&format!("{} o.. {}\n", diagram.system.id, sub.id));
    }

    for comp in &diagram.system.components {
        output.push_str(&format!("rectangle {}\n", comp.id));
        output.push_str(&format!("{} o.. {}\n", diagram.system.id, comp.id));
    }

    // Process subsystems
    for sub in &diagram.system.subsystems {
        generate_subsystem_relationships(&mut output, sub);
    }

    // Process direct components
    for comp in &diagram.system.components {
        generate_component_relationships(&mut output, comp);
    }

    // Footer
    output.push_str("\n@enduml\n");

    output
}

fn generate_subsystem_relationships(output: &mut String, subsystem: &Subsystem) {
    for comp in &subsystem.components {
        output.push_str(&format!("rectangle {}\n", comp.id));
        output.push_str(&format!("{} o.. {}\n", subsystem.id, comp.id));
        generate_component_relationships(output, comp);
    }
}

fn generate_component_relationships(output: &mut String, component: &Component) {
    for module in &component.modules {
        output.push_str(&format!("rectangle {}\n", module.id));
        output.push_str(&format!("{} o.. {}\n", component.id, module.id));
        generate_module_relationships(output, module);
    }
}

fn generate_module_relationships(output: &mut String, module: &Module) {
    for sub in &module.submodules {
        output.push_str(&format!("rectangle {}\n", sub.id));
        output.push_str(&format!("{} o.. {}\n", module.id, sub.id));
        generate_submodule_relationships(output, sub);
    }
}

fn generate_submodule_relationships(output: &mut String, submodule: &Submodule) {
    for sub in &submodule.submodules {
        output.push_str(&format!("rectangle {}\n", sub.id));
        output.push_str(&format!("{} o.. {}\n", submodule.id, sub.id));
        // Recursive for nested submodules
        generate_submodule_relationships(output, sub);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::logic::concept::{Component, Module, Subsystem};

    #[test]
    fn test_generate_logic_concept_plantuml() {
        let mut diagram = LogicConceptDiagram::new("MY_SYSTEM", "My System");

        // Add subsystem with component
        diagram.system.subsystems.push(Subsystem {
            id: "SUB_SYS".to_string(),
            name: "Sub System".to_string(),
            description: None,
            components: vec![Component {
                id: "COMP".to_string(),
                name: "Component".to_string(),
                description: None,
                modules: vec![Module {
                    id: "MOD".to_string(),
                    name: "Module".to_string(),
                    description: None,
                    submodules: vec![],
                }],
            }],
        });

        let output = generate_logic_concept_plantuml(&diagram);

        assert!(output.contains("@startuml"));
        assert!(output.contains("@enduml"));
        assert!(output.contains("rectangle MY_SYSTEM"));
        assert!(output.contains("rectangle SUB_SYS"));
        assert!(output.contains("MY_SYSTEM o.. SUB_SYS"));
        assert!(output.contains("SUB_SYS o.. COMP"));
        assert!(output.contains("COMP o.. MOD"));
    }
}
