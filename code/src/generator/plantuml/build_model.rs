use crate::model::build::BuildModel;
use crate::model::workspace::Workspace;

/// Generate PlantUML diagram for a BuildModel
pub fn generate_build_model_plantuml(workspace: &Workspace, model: &BuildModel) -> String {
    let mut output = String::new();
    output.push_str("@startuml\n\n");

    for artifact in &model.artifacts {
        let output_type_str = format!("{:?}", artifact.output_type);

        output.push_str(&format!(
            "artifact \"{}\" <<{}>> as {}\n",
            artifact.name, output_type_str, artifact.id
        ));

        // If source_packages references code_model packages, show mapping note
        if !artifact.source_packages.is_empty() {
            let pkg_names: Vec<String> = artifact
                .source_packages
                .iter()
                .filter_map(|pkg_id| resolve_package_name(workspace, pkg_id).map(|s| s.to_string()))
                .collect();

            if !pkg_names.is_empty() {
                output.push_str(&format!(
                    "note right of {}\n  source: {}\nend note\n",
                    artifact.id,
                    pkg_names.join(", ")
                ));
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

/// Resolve package name from code_model
fn resolve_package_name<'a>(workspace: &'a Workspace, package_id: &str) -> Option<&'a str> {
    if let Some(code_model) = &workspace.code_model {
        return code_model.get_package_name(package_id);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::build::*;
    use crate::model::code::{CodeModel, CodePackage};

    fn create_test_model() -> BuildModel {
        let mut model = BuildModel::new("Test Build Model");
        model.artifacts.push(BuildArtifact {
            id: "CORE_BIN".to_string(),
            name: "Core Binary".to_string(),
            description: None,
            build_tool: Some(BuildTool::Cargo),
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        model.artifacts.push(BuildArtifact {
            id: "UTIL_LIB".to_string(),
            name: "Utility Library".to_string(),
            description: None,
            build_tool: Some(BuildTool::Cargo),
            output_type: OutputType::Library,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        model.dependencies.push(ArtifactDependency {
            from: "CORE_BIN".to_string(),
            to: "UTIL_LIB".to_string(),
        });
        model
    }

    fn create_test_workspace() -> Workspace {
        Workspace::new("Test")
    }

    #[test]
    fn test_generate_build_model() {
        let model = create_test_model();
        let ws = create_test_workspace();
        let result = generate_build_model_plantuml(&ws, &model);

        assert!(result.contains("@startuml"));
        assert!(result.contains("@enduml"));
        assert!(result.contains("artifact \"Core Binary\""));
        assert!(result.contains("as CORE_BIN"));
        assert!(result.contains("CORE_BIN --> UTIL_LIB"));
    }

    #[test]
    fn test_generate_build_model_with_source_mapping() {
        let mut model = BuildModel::new("Test");
        model.artifacts.push(BuildArtifact {
            id: "CORE_BIN".to_string(),
            name: "Core Binary".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Binary,
            source_packages: vec!["CORE_LIB".to_string()],
            build_file: None,
            profile: None,
            build_args: None,
        });

        let mut ws = Workspace::new("Test");
        let mut code_model = CodeModel::new("Test Code");
        code_model.packages.push(CodePackage {
            id: "CORE_LIB".to_string(),
            name: "Core Library".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        ws.code_model = Some(code_model);

        let result = generate_build_model_plantuml(&ws, &model);
        assert!(result.contains("source: Core Library"));
    }
}
