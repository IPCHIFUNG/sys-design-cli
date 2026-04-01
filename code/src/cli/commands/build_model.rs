use crate::cli::args::{BuildModelAddCommand, BuildModelCommand, BuildModelListElement, BuildModelRemoveCommand};
use crate::model::build::BuildModel;
use crate::store::{BuildOperations, YamlStore};
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::Path;

pub fn execute(model_file: &Path, cmd: BuildModelCommand) -> Result<()> {
    match cmd {
        BuildModelCommand::Add(add_cmd) => execute_add(model_file, add_cmd),
        BuildModelCommand::Remove(remove_cmd) => execute_remove(model_file, remove_cmd),
        BuildModelCommand::List { element } => execute_list(model_file, element),
        BuildModelCommand::Show { id } => execute_show(model_file, &id),
    }
}

fn execute_add(model_file: &Path, cmd: BuildModelAddCommand) -> Result<()> {
    match cmd {
        BuildModelAddCommand::Artifact {
            id,
            name,
            desc,
            build_tool,
            output_type,
            source_packages,
            build_file,
            profile,
            build_args,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            // Cross-diagram validation: source_packages must exist in code_model (if code_model exists)
            if !source_packages.is_empty() {
                if let Some(code_model) = &workspace.code_model {
                    for pkg_id in &source_packages {
                        if code_model.find_package(pkg_id).is_none() {
                            return Err(AppError::ElementNotFound(format!(
                                "source_package '{}' not found in code_model",
                                pkg_id
                            )));
                        }
                    }
                } else {
                    return Err(AppError::ElementNotFound(
                        "code_model not found in workspace".to_string(),
                    ));
                }
            }

            if workspace.build_model.is_none() {
                workspace.build_model = Some(BuildModel::new("Build Model"));
            }

            let model = workspace.build_model.as_mut().unwrap();
            BuildOperations::add_artifact(
                model,
                &id,
                name.as_deref(),
                desc.as_deref(),
                build_tool.map(|t| t.into()),
                output_type.map(|t| t.into()),
                source_packages,
                build_file.as_deref(),
                profile.map(|p| p.into()),
                build_args.as_deref(),
            )?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} artifact: {}", "Added".green(), id);
            Ok(())
        }
        BuildModelAddCommand::Dependency { from, to } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.build_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("build_model not found in workspace".to_string())
            })?;

            BuildOperations::add_dependency(model, &from, &to)?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} dependency: {} -> {}", "Added".green(), from, to);
            Ok(())
        }
    }
}

fn execute_remove(model_file: &Path, cmd: BuildModelRemoveCommand) -> Result<()> {
    match cmd {
        BuildModelRemoveCommand::Artifact { id } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.build_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("build_model not found in workspace".to_string())
            })?;

            BuildOperations::remove_artifact(model, &id)?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} artifact: {}", "Removed".green(), id);
            Ok(())
        }
        BuildModelRemoveCommand::Dependency { from, to } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.build_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("build_model not found in workspace".to_string())
            })?;

            BuildOperations::remove_dependency(model, &from, &to)?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} dependency: {} -> {}", "Removed".green(), from, to);
            Ok(())
        }
    }
}

fn execute_list(model_file: &Path, element: BuildModelListElement) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(model_file)?;

    let model = workspace.build_model.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("build_model not found in workspace".to_string())
    })?;

    match element {
        BuildModelListElement::Artifacts => {
            if model.artifacts.is_empty() {
                println!("No artifacts found.");
            } else {
                println!("{}", "Artifacts:".cyan().bold());
                for artifact in &model.artifacts {
                    println!(
                        "  {} ({})",
                        artifact.id.white().bold(),
                        artifact.name
                    );
                    if let Some(ref tool) = artifact.build_tool {
                        println!("    build_tool: {:?}", tool);
                    }
                    println!("    output_type: {:?}", artifact.output_type);
                    if !artifact.source_packages.is_empty() {
                        println!("    source_packages: {}", artifact.source_packages.join(", "));
                    }
                }
            }
        }
        BuildModelListElement::Dependencies => {
            if model.dependencies.is_empty() {
                println!("No dependencies found.");
            } else {
                println!("{}", "Dependencies:".cyan().bold());
                for dep in &model.dependencies {
                    println!("  {} -> {}", dep.from, dep.to);
                }
            }
        }
    }

    Ok(())
}

fn execute_show(model_file: &Path, id: &str) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(model_file)?;

    let model = workspace.build_model.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("build_model not found in workspace".to_string())
    })?;

    let artifact = model.find_artifact(id).ok_or_else(|| {
        AppError::ElementNotFound(format!("artifact: {}", id))
    })?;

    println!("{}", format!("Artifact: {}", artifact.id).cyan().bold());
    println!("  name: {}", artifact.name);
    if let Some(ref desc) = artifact.description {
        println!("  description: {}", desc);
    }
    if let Some(ref tool) = artifact.build_tool {
        println!("  build_tool: {:?}", tool);
    }
    println!("  output_type: {:?}", artifact.output_type);
    if !artifact.source_packages.is_empty() {
        println!("  source_packages: {}", artifact.source_packages.join(", "));
    }
    if let Some(ref bf) = artifact.build_file {
        println!("  build_file: {}", bf);
    }
    if let Some(ref profile) = artifact.profile {
        println!("  profile: {:?}", profile);
    }
    if let Some(ref args) = artifact.build_args {
        println!("  build_args: {}", args);
    }

    Ok(())
}
