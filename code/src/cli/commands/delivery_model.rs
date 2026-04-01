use crate::cli::args::{DeliveryModelAddCommand, DeliveryModelCommand, DeliveryModelListElement, DeliveryModelRemoveCommand};
use crate::model::delivery::DeliveryModel;
use crate::store::{DeliveryOperations, YamlStore};
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::Path;

pub fn execute(model_file: &Path, cmd: DeliveryModelCommand) -> Result<()> {
    match cmd {
        DeliveryModelCommand::Add(add_cmd) => execute_add(model_file, add_cmd),
        DeliveryModelCommand::Remove(remove_cmd) => execute_remove(model_file, remove_cmd),
        DeliveryModelCommand::List { element } => execute_list(model_file, element),
        DeliveryModelCommand::Show { id } => execute_show(model_file, &id),
    }
}

fn execute_add(model_file: &Path, cmd: DeliveryModelAddCommand) -> Result<()> {
    match cmd {
        DeliveryModelAddCommand::Package {
            id,
            name,
            desc,
            version,
            delivery_type,
            artifacts,
            registry,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            // Cross-diagram validation: artifacts must exist in build_model (if build_model exists)
            if !artifacts.is_empty() {
                if let Some(build_model) = &workspace.build_model {
                    for art_id in &artifacts {
                        if build_model.find_artifact(art_id).is_none() {
                            return Err(AppError::ElementNotFound(format!(
                                "artifact '{}' not found in build_model",
                                art_id
                            )));
                        }
                    }
                } else {
                    return Err(AppError::ElementNotFound(
                        "build_model not found in workspace".to_string(),
                    ));
                }
            }

            if workspace.delivery_model.is_none() {
                workspace.delivery_model = Some(DeliveryModel::new("Delivery Model"));
            }

            let model = workspace.delivery_model.as_mut().unwrap();
            DeliveryOperations::add_package(
                model,
                &id,
                name.as_deref(),
                desc.as_deref(),
                version.as_deref(),
                delivery_type.map(|t| t.into()),
                artifacts,
                registry.as_deref(),
            )?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} package: {}", "Added".green(), id);
            Ok(())
        }
    }
}

fn execute_remove(model_file: &Path, cmd: DeliveryModelRemoveCommand) -> Result<()> {
    match cmd {
        DeliveryModelRemoveCommand::Package { id } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.delivery_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("delivery_model not found in workspace".to_string())
            })?;

            DeliveryOperations::remove_package(model, &id)?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} package: {}", "Removed".green(), id);
            Ok(())
        }
    }
}

fn execute_list(model_file: &Path, element: DeliveryModelListElement) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(model_file)?;

    let model = workspace.delivery_model.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("delivery_model not found in workspace".to_string())
    })?;

    match element {
        DeliveryModelListElement::Packages => {
            if model.packages.is_empty() {
                println!("No packages found.");
            } else {
                println!("{}", "Packages:".cyan().bold());
                for package in &model.packages {
                    println!(
                        "  {} ({})",
                        package.id.white().bold(),
                        package.name
                    );
                    println!("    delivery_type: {:?}", package.delivery_type);
                    if let Some(ref ver) = package.version {
                        println!("    version: {}", ver);
                    }
                    if !package.artifacts.is_empty() {
                        println!("    artifacts: {}", package.artifacts.join(", "));
                    }
                    if let Some(ref reg) = package.registry {
                        println!("    registry: {}", reg);
                    }
                }
            }
        }
    }

    Ok(())
}

fn execute_show(model_file: &Path, id: &str) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(model_file)?;

    let model = workspace.delivery_model.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("delivery_model not found in workspace".to_string())
    })?;

    let package = model.find_package(id).ok_or_else(|| {
        AppError::ElementNotFound(format!("package: {}", id))
    })?;

    println!("{}", format!("Package: {}", package.id).cyan().bold());
    println!("  name: {}", package.name);
    if let Some(ref desc) = package.description {
        println!("  description: {}", desc);
    }
    if let Some(ref ver) = package.version {
        println!("  version: {}", ver);
    }
    println!("  delivery_type: {:?}", package.delivery_type);
    if !package.artifacts.is_empty() {
        println!("  artifacts: {}", package.artifacts.join(", "));
    }
    if let Some(ref reg) = package.registry {
        println!("  registry: {}", reg);
    }

    Ok(())
}
