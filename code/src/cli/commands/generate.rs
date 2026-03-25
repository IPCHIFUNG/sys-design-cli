use crate::cli::args::GenerateCommand;
use crate::generator::plantuml::{
    generate_logic_concept_plantuml_with_root, generate_plantuml, generate_concept_model_plantuml,
};
use crate::store::YamlStore;
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::PathBuf;

pub fn execute(src: &std::path::Path, output: Option<PathBuf>, command: &GenerateCommand) -> Result<()> {
    let plantuml = load_and_generate(src, command)?;

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

fn load_and_generate(src: &std::path::Path, command: &GenerateCommand) -> Result<String> {
    // Load as workspace (handles legacy formats via load_workspace_any)
    let workspace = YamlStore::load_workspace_any(src)?;
    generate_from_workspace(&workspace, command)
}

fn generate_from_workspace(
    workspace: &crate::model::workspace::Workspace,
    command: &GenerateCommand,
) -> Result<String> {
    match command {
        GenerateCommand::ContextModelDiagram => match &workspace.context_diagram {
            Some(diagram) => Ok(generate_plantuml(diagram)),
            None => Err(AppError::ElementNotFound(
                "context_diagram not found in workspace".to_string(),
            )),
        },
        GenerateCommand::ConceptModelDiagram => match &workspace.logic_architecture_concept_model
        {
            Some(model) => Ok(generate_concept_model_plantuml(model)),
            None => Err(AppError::ElementNotFound(
                "logic_architecture_concept_model not found in workspace".to_string(),
            )),
        },
        GenerateCommand::LogicModelDiagram { root } => match &workspace.logic_view {
            Some(diagram) => Ok(generate_logic_concept_plantuml_with_root(diagram, root.as_deref())),
            None => Err(AppError::ElementNotFound(
                "logic_view not found in workspace".to_string(),
            )),
        },
    }
}
