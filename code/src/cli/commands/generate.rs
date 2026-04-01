use crate::cli::args::GenerateCommand;
use crate::generator::plantuml::{
    generate_logic_concept_plantuml_with_workspace, generate_plantuml, generate_concept_model_plantuml,
    generate_runtime_plantuml, generate_code_model_plantuml, generate_build_model_plantuml,
    generate_delivery_model_plantuml, generate_deployment_model_plantuml,
};
use crate::store::YamlStore;
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::PathBuf;

pub fn execute(model_file: &std::path::Path, output: Option<PathBuf>, command: &GenerateCommand) -> Result<()> {
    let plantuml = load_and_generate(model_file, command)?;

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

fn load_and_generate(model_file: &std::path::Path, command: &GenerateCommand) -> Result<String> {
    // Load as workspace (handles legacy formats via load_workspace_any)
    let workspace = YamlStore::load_workspace_any(model_file)?;
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
            Some(diagram) => Ok(generate_logic_concept_plantuml_with_workspace(
                Some(workspace),
                diagram,
                root.as_deref(),
            )),
            None => Err(AppError::ElementNotFound(
                "logic_view not found in workspace".to_string(),
            )),
        },
        GenerateCommand::RuntimeModelDiagram { scenario_id } => match &workspace.runtime_view {
            Some(view) => {
                let sid = match scenario_id {
                    Some(id) => id.clone(),
                    None => {
                        if view.scenarios.len() == 1 {
                            view.scenarios[0].id.clone()
                        } else {
                            return Err(AppError::InvalidOperation(
                                "Multiple scenarios exist. Please specify scenario_id.".to_string()
                            ));
                        }
                    }
                };
                Ok(generate_runtime_plantuml(workspace, view, &sid))
            }
            None => Err(AppError::ElementNotFound(
                "runtime_view not found in workspace".to_string(),
            )),
        },
        GenerateCommand::CodeModelDiagram => match &workspace.code_model {
            Some(model) => Ok(generate_code_model_plantuml(workspace, model)),
            None => Err(AppError::ElementNotFound(
                "code_model not found in workspace".to_string(),
            )),
        },
        GenerateCommand::BuildModelDiagram => match &workspace.build_model {
            Some(model) => Ok(generate_build_model_plantuml(workspace, model)),
            None => Err(AppError::ElementNotFound(
                "build_model not found in workspace".to_string(),
            )),
        },
        GenerateCommand::DeliveryModelDiagram => match &workspace.delivery_model {
            Some(model) => Ok(generate_delivery_model_plantuml(workspace, model)),
            None => Err(AppError::ElementNotFound(
                "delivery_model not found in workspace".to_string(),
            )),
        },
        GenerateCommand::DeploymentModelDiagram { environment_id } => match &workspace.deployment_model {
            Some(model) => {
                let env_id = environment_id.as_deref();
                generate_deployment_model_plantuml(workspace, model, env_id)
                    .map_err(|e| AppError::InvalidOperation(e))
            }
            None => Err(AppError::ElementNotFound(
                "deployment_model not found in workspace".to_string(),
            )),
        },
    }
}
