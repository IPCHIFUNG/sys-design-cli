use crate::model::logic::concept_model::LogicArchitectureConceptModel;

/// Generate PlantUML for Logic Architecture Concept Model Diagram
pub fn generate_concept_model_plantuml(model: &LogicArchitectureConceptModel) -> String {
    let mut output = String::new();

    // Header
    output.push_str("@startuml\n\n");
    output.push_str("skinparam defaultTextAlignment center\n\n");

    // Draw hierarchy levels
    for level in &model.hierarchy.levels {
        output.push_str(&format!("rectangle {}\n", level.id));
    }

    output.push_str("\n");

    // Draw containment relationships
    for level in &model.hierarchy.levels {
        for child_type in &level.can_contain {
            // Skip recursive relationships for now (will be handled specially)
            if child_type != &level.id {
                output.push_str(&format!("{} o.. {}\n", level.id, child_type));
            }
        }
    }

    // Handle recursive relationship (SUBMODULE can contain SUBMODULE)
    if let Some(submodule_level) = model.hierarchy.levels.iter().find(|l| l.id == "SUBMODULE") {
        if submodule_level.can_contain.contains(&"SUBMODULE".to_string()) {
            output.push_str("\n");
            output.push_str("SUBMODULE o.. SUBMODULE\n");
        }
    }

    // Footer
    output.push_str("\n@enduml\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_concept_model_plantuml() {
        let model = LogicArchitectureConceptModel::new("Test Model");
        let output = generate_concept_model_plantuml(&model);

        assert!(output.contains("@startuml"));
        assert!(output.contains("@enduml"));
        assert!(output.contains("skinparam defaultTextAlignment center"));
        assert!(output.contains("rectangle SYSTEM"));
        assert!(output.contains("SYSTEM o.. SUBSYSTEM"));
    }
}
