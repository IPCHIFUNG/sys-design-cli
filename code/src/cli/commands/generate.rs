use crate::cli::args::DiagramType;
use crate::generator::plantuml::{generate_logic_concept_plantuml, generate_plantuml};
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
    // First try to load as workspace
    if let Ok(workspace) = YamlStore::load_workspace(src) {
        return generate_from_workspace(&workspace, diagram_type);
    }

    // Fallback: try loading as individual diagram type
    match diagram_type {
        DiagramType::Context => {
            let diagram = YamlStore::load_context(src)?;
            Ok(generate_plantuml(&diagram))
        }
        DiagramType::LogicConcept => {
            let diagram = YamlStore::load_logic_concept(src)?;
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
        DiagramType::LogicConcept => {
            match &workspace.logic_concept_diagram {
                Some(diagram) => Ok(generate_logic_concept_plantuml(diagram)),
                None => Err(AppError::ElementNotFound(
                    "logic_concept_diagram not found in workspace".to_string()
                )),
            }
        }
    }
}
