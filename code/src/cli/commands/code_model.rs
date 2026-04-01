use crate::cli::args::{CodeModelAddCommand, CodeModelCommand, CodeModelListElement, CodeModelRemoveCommand};
use crate::model::code::CodeModel;
use crate::store::{CodeOperations, YamlStore};
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::Path;

pub fn execute(model_file: &Path, cmd: CodeModelCommand) -> Result<()> {
    match cmd {
        CodeModelCommand::Add(add_cmd) => execute_add(model_file, add_cmd),
        CodeModelCommand::Remove(remove_cmd) => execute_remove(model_file, remove_cmd),
        CodeModelCommand::List { element } => execute_list(model_file, element),
        CodeModelCommand::Show { id } => execute_show(model_file, &id),
    }
}

fn execute_add(model_file: &Path, cmd: CodeModelAddCommand) -> Result<()> {
    match cmd {
        CodeModelAddCommand::Package {
            id,
            name,
            desc,
            language,
            framework,
            path,
            element_id,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            // Cross-diagram validation: element_id must exist in logic_view
            if let Some(ref eid) = element_id {
                if let Some(lv) = &workspace.logic_view {
                    if lv.get_element_name(eid).is_none() {
                        return Err(AppError::ElementNotFound(format!(
                            "element_id '{}' not found in logic_view",
                            eid
                        )));
                    }
                } else {
                    return Err(AppError::ElementNotFound(
                        "logic_view not found in workspace".to_string(),
                    ));
                }
            }

            if workspace.code_model.is_none() {
                workspace.code_model = Some(CodeModel::new("Code Model"));
            }

            let model = workspace.code_model.as_mut().unwrap();
            CodeOperations::add_package(
                model,
                &id,
                name.as_deref(),
                desc.as_deref(),
                language.map(|l| l.into()),
                framework.as_deref(),
                path.as_deref(),
                element_id.as_deref(),
            )?;

            YamlStore::save_code_model_to_workspace(model_file, &mut workspace)?;
            println!("{} package: {}", "Added".green(), id);
            Ok(())
        }
        CodeModelAddCommand::Dependency { from, to } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.code_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("code_model not found in workspace".to_string())
            })?;

            CodeOperations::add_dependency(model, &from, &to)?;

            YamlStore::save_code_model_to_workspace(model_file, &mut workspace)?;
            println!("{} dependency: {} -> {}", "Added".green(), from, to);
            Ok(())
        }
    }
}

fn execute_remove(model_file: &Path, cmd: CodeModelRemoveCommand) -> Result<()> {
    match cmd {
        CodeModelRemoveCommand::Package { id } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.code_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("code_model not found in workspace".to_string())
            })?;

            CodeOperations::remove_package(model, &id)?;

            YamlStore::save_code_model_to_workspace(model_file, &mut workspace)?;
            println!("{} package: {}", "Removed".green(), id);
            Ok(())
        }
        CodeModelRemoveCommand::Dependency { from, to } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.code_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("code_model not found in workspace".to_string())
            })?;

            CodeOperations::remove_dependency(model, &from, &to)?;

            YamlStore::save_code_model_to_workspace(model_file, &mut workspace)?;
            println!("{} dependency: {} -> {}", "Removed".green(), from, to);
            Ok(())
        }
    }
}

fn execute_list(model_file: &Path, element: CodeModelListElement) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(model_file)?;

    let model = workspace.code_model.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("code_model not found in workspace".to_string())
    })?;

    match element {
        CodeModelListElement::Packages => {
            if model.packages.is_empty() {
                println!("No packages found.");
            } else {
                println!("{}", "Packages:".cyan().bold());
                for pkg in &model.packages {
                    println!(
                        "  {} ({})",
                        pkg.id.white().bold(),
                        pkg.name
                    );
                    if let Some(ref lang) = pkg.language {
                        println!("    language: {:?}", lang);
                    }
                    if let Some(ref path) = pkg.path {
                        println!("    path: {}", path);
                    }
                }
            }
        }
        CodeModelListElement::Dependencies => {
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

    let model = workspace.code_model.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("code_model not found in workspace".to_string())
    })?;

    let pkg = model.find_package(id).ok_or_else(|| {
        AppError::ElementNotFound(format!("package: {}", id))
    })?;

    println!("{}", format!("Package: {}", pkg.id).cyan().bold());
    println!("  name: {}", pkg.name);
    if let Some(ref desc) = pkg.description {
        println!("  description: {}", desc);
    }
    if let Some(ref lang) = pkg.language {
        println!("  language: {:?}", lang);
    }
    if let Some(ref fw) = pkg.framework {
        println!("  framework: {}", fw);
    }
    if let Some(ref path) = pkg.path {
        println!("  path: {}", path);
    }
    if let Some(ref eid) = pkg.element_id {
        println!("  element_id: {}", eid);
    }

    Ok(())
}
