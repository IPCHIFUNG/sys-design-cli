use crate::cli::args::DiagramType;
use crate::generator::plantuml::{generate_logic_concept_plantuml, generate_plantuml, generate_concept_model_plantuml};
use crate::store::YamlStore;
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::PathBuf;

pub fn execute(src: &std::path::Path, output: Option<PathBuf>, diagram_type: DiagramType) -> Result<()> {
    let plantuml = load_and_generate(src, &diagram_type)?;

    match output {
        Some(path) => {
            std::fs::write(&path, &plantuml)?;
            println!(
                "{} PlantUML diagram to: {}",
                "Generated".green(),
                path.display()
            );
        }
        None => {
            println!("{}", plantuml);
        }
    }

    Ok(())
}

fn load_and_generate(src: &std::path::Path, diagram_type: &DiagramType) -> Result<String> {
    // First try to load as workspace (only if it has actual diagrams)
    if let Ok(workspace) = YamlStore::load_workspace(src) {
        // Only use workspace if it actually contains diagrams
        if workspace.context_diagram.is_some()
            || workspace.logic_architecture_concept_model.is_some()
            || workspace.logic_view.is_some() {
            return generate_from_workspace(&workspace, diagram_type);
        }
    }

    // Fallback: try loading as individual diagram type
    match diagram_type {
        DiagramType::Context => {
            let diagram = YamlStore::load_context(src)?;
            Ok(generate_plantuml(&diagram))
        }
        DiagramType::ConceptModel => {
            let model = YamlStore::load_concept_model(src)?;
            Ok(generate_concept_model_plantuml(&model))
        }
        DiagramType::LogicView => {
            let diagram = YamlStore::load_logic_view(src)?;
            Ok(generate_logic_concept_plantuml(&diagram))
        }
    }
}

fn generate_from_workspace(workspace: &crate::model::workspace::Workspace, diagram_type: &DiagramType) -> Result<String> {
    match diagram_type {
        DiagramType::Context => {
            match &workspace.context_diagram {
                Some(diagram) => Ok(generate_plantuml(diagram)),
                None => Err(AppError::ElementNotFound(
                    "context_diagram not found in workspace".to_string()
                )),
            }
        }
        DiagramType::ConceptModel => {
            match &workspace.logic_architecture_concept_model {
                Some(model) => Ok(generate_concept_model_plantuml(model)),
                None => Err(AppError::ElementNotFound(
                    "logic_architecture_concept_model not found in workspace".to_string()
                )),
            }
        }
        DiagramType::LogicView => {
            match &workspace.logic_view {
                Some(diagram) => Ok(generate_logic_concept_plantuml(diagram)),
                None => Err(AppError::ElementNotFound(
                    "logic_view not found in workspace".to_string()
                )),
            }
        }
    }
}
