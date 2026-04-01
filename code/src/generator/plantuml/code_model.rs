use crate::model::code::CodeModel;
use crate::model::workspace::Workspace;

/// Generate PlantUML diagram for a CodeModel
pub fn generate_code_model_plantuml(workspace: &Workspace, model: &CodeModel) -> String {
    let mut output = String::new();
    output.push_str("@startuml\n\n");

    for pkg in &model.packages {
        let lang_str = match &pkg.language {
            Some(lang) => format!(" <<{:?}>>", lang).replace("Custom(", "\"").replace(")", "\""),
            None => String::new(),
        };

        output.push_str(&format!(
            "package \"{}\"{} as {}\n",
            pkg.name, lang_str, pkg.id
        ));

        // If element_id references a logic_view element, show mapping note
        if let Some(element_id) = &pkg.element_id {
            let element_name = resolve_logic_element_name(workspace, element_id);
            if let Some(name) = element_name {
                output.push_str(&format!("note right of {}\n  maps to: {}\nend note\n", pkg.id, name));
            }
        }

        output.push_str("\n");
    }

    // Dependencies
    for dep in &model.dependencies {
        output.push_str(&format!("{} --> {}\n", dep.from, dep.to));
    }

    output.push_str("\n@enduml\n");
    output
}

/// Resolve element name from logic view
fn resolve_logic_element_name<'a>(workspace: &'a Workspace, element_id: &str) -> Option<&'a str> {
    if let Some(lv) = &workspace.logic_view {
        return lv.get_element_name(element_id);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::code::*;

    fn create_test_model() -> CodeModel {
        let mut model = CodeModel::new("Test Code Model");
        model.packages.push(CodePackage {
            id: "CORE_LIB".to_string(),
            name: "Core Library".to_string(),
            description: None,
            language: Some(Language::Rust),
            framework: None,
            path: None,
            element_id: None,
        });
        model.packages.push(CodePackage {
            id: "UTIL_LIB".to_string(),
            name: "Utility Library".to_string(),
            description: None,
            language: Some(Language::Python),
            framework: None,
            path: None,
            element_id: None,
        });
        model.dependencies.push(PackageDependency {
            from: "CORE_LIB".to_string(),
            to: "UTIL_LIB".to_string(),
        });
        model
    }

    fn create_test_workspace() -> Workspace {
        Workspace::new("Test")
    }

    #[test]
    fn test_generate_code_model() {
        let model = create_test_model();
        let ws = create_test_workspace();
        let result = generate_code_model_plantuml(&ws, &model);

        assert!(result.contains("@startuml"));
        assert!(result.contains("@enduml"));
        assert!(result.contains("package \"Core Library\""));
        assert!(result.contains("as CORE_LIB"));
        assert!(result.contains("CORE_LIB --> UTIL_LIB"));
    }

    #[test]
    fn test_generate_code_model_with_mapping() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "CORE_LIB".to_string(),
            name: "Core Library".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: Some("MY_SYSTEM".to_string()), // matches system ID
        });

        let mut ws = Workspace::new("Test");
        ws.logic_view = Some(crate::model::logic::concept::LogicConceptDiagram::new(
            "MY_SYSTEM", "My System",
        ));

        let result = generate_code_model_plantuml(&ws, &model);
        assert!(result.contains("maps to"));
    }
}
