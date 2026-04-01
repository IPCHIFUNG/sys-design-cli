use crate::model::delivery::DeliveryModel;
use crate::model::workspace::Workspace;

/// Generate PlantUML diagram for a DeliveryModel
pub fn generate_delivery_model_plantuml(workspace: &Workspace, model: &DeliveryModel) -> String {
    let mut output = String::new();
    output.push_str("@startuml\n\n");

    for package in &model.packages {
        let delivery_type_str = format!("{:?}", package.delivery_type);

        output.push_str(&format!(
            "rectangle \"{} <<{}>>\" as {}\n",
            package.name, delivery_type_str, package.id
        ));

        // Build notes showing version and resolved artifact names
        let mut note_lines: Vec<String> = Vec::new();
        if let Some(ref ver) = package.version {
            note_lines.push(format!("version: {}", ver));
        }
        if !package.artifacts.is_empty() {
            let artifact_names: Vec<String> = package
                .artifacts
                .iter()
                .filter_map(|art_id| resolve_artifact_name(workspace, art_id).map(|s| s.to_string()))
                .collect();
            if !artifact_names.is_empty() {
                note_lines.push(format!("artifacts: {}", artifact_names.join(", ")));
            }
        }

        if !note_lines.is_empty() {
            output.push_str(&format!(
                "note right of {}\n  {}\nend note\n",
                package.id,
                note_lines.join("\\n")
            ));
        }

        // Registry note
        if let Some(ref registry) = package.registry {
            output.push_str(&format!(
                "note left of {}\n  registry: {}\nend note\n",
                package.id, registry
            ));
        }

        output.push_str("\n");
    }

    output.push_str("@enduml\n");
    output
}

/// Resolve artifact name from build_model
fn resolve_artifact_name<'a>(workspace: &'a Workspace, artifact_id: &str) -> Option<&'a str> {
    if let Some(build_model) = &workspace.build_model {
        return build_model.get_artifact_name(artifact_id);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::delivery::*;
    use crate::model::build::{BuildModel, BuildArtifact, OutputType};

    fn create_test_model() -> DeliveryModel {
        let mut model = DeliveryModel::new("Test Delivery Model");
        model.packages.push(DeliveryPackage {
            id: "CORE_IMG".to_string(),
            name: "Core Image".to_string(),
            description: None,
            version: Some("1.0.0".to_string()),
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: Some("registry.example.com/core".to_string()),
        });
        model.packages.push(DeliveryPackage {
            id: "UTIL_CRATE".to_string(),
            name: "Utility Crate".to_string(),
            description: None,
            version: Some("0.1.0".to_string()),
            delivery_type: DeliveryType::Crate,
            artifacts: vec![],
            registry: None,
        });
        model
    }

    fn create_test_workspace() -> Workspace {
        Workspace::new("Test")
    }

    #[test]
    fn test_generate_delivery_model() {
        let model = create_test_model();
        let ws = create_test_workspace();
        let result = generate_delivery_model_plantuml(&ws, &model);

        assert!(result.contains("@startuml"));
        assert!(result.contains("@enduml"));
        assert!(result.contains("Core Image"));
        assert!(result.contains("as CORE_IMG"));
        assert!(result.contains("<<ContainerImage>>"));
        assert!(result.contains("Utility Crate"));
        assert!(result.contains("<<Crate>>"));
        assert!(result.contains("registry: registry.example.com/core"));
    }

    #[test]
    fn test_generate_delivery_model_with_artifacts() {
        let mut model = DeliveryModel::new("Test");
        model.packages.push(DeliveryPackage {
            id: "CORE_IMG".to_string(),
            name: "Core Image".to_string(),
            description: None,
            version: Some("2.0.0".to_string()),
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec!["CORE_BIN".to_string()],
            registry: None,
        });

        let mut ws = Workspace::new("Test");
        let mut build_model = BuildModel::new("Test Build");
        build_model.artifacts.push(BuildArtifact {
            id: "CORE_BIN".to_string(),
            name: "Core Binary".to_string(),
            description: None,
            build_tool: None,
            output_type: OutputType::Binary,
            source_packages: vec![],
            build_file: None,
            profile: None,
            build_args: None,
        });
        ws.build_model = Some(build_model);

        let result = generate_delivery_model_plantuml(&ws, &model);
        assert!(result.contains("version: 2.0.0"));
        assert!(result.contains("artifacts: Core Binary"));
    }
}
