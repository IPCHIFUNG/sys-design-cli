use crate::cli::args::{ConceptModelAddCommand, ConceptModelCommand, ConceptModelRemoveCommand};
use crate::model::logic::concept_model::{LevelDefinition, LogicArchitectureConceptModel};
use crate::model::workspace::Workspace;
use crate::store::YamlStore;
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::Path;

pub fn execute(src: &Path, cmd: ConceptModelCommand) -> Result<()> {
    match cmd {
        ConceptModelCommand::Add(add_cmd) => execute_add(src, add_cmd),
        ConceptModelCommand::Remove(remove_cmd) => execute_remove(src, remove_cmd),
        ConceptModelCommand::List => execute_list(src),
        ConceptModelCommand::Show { id } => execute_show(src, &id),
    }
}

fn execute_add(src: &Path, cmd: ConceptModelAddCommand) -> Result<()> {
    match cmd {
        ConceptModelAddCommand::Element { type_name } => execute_add_element(src, &type_name),
        ConceptModelAddCommand::Containment { parent, child } => {
            execute_add_containment(src, &parent, &child)
        }
        ConceptModelAddCommand::Level { id, name, desc, can_contain } => {
            execute_add_level(src, id, name, desc, can_contain)
        }
    }
}

fn execute_remove(src: &Path, cmd: ConceptModelRemoveCommand) -> Result<()> {
    match cmd {
        ConceptModelRemoveCommand::Element { type_name } => execute_remove_element(src, &type_name),
        ConceptModelRemoveCommand::Level { id } => execute_remove_level(src, &id),
    }
}

/// Load workspace from file, creating new if doesn't exist
fn load_workspace(src: &Path) -> Result<Workspace> {
    if !YamlStore::exists(src) {
        return Ok(Workspace::new("New Project"));
    }
    YamlStore::load_workspace_any(src)
}

fn execute_add_level(
    src: &Path,
    id: String,
    name: Option<String>,
    desc: Option<String>,
    can_contain: Vec<String>,
) -> Result<()> {
    let mut workspace = load_workspace(src)?;

    let model = workspace.logic_architecture_concept_model.as_mut().ok_or_else(|| {
        AppError::InvalidOperation("Concept model not found. Run 'init' first.".to_string())
    })?;

    // Check for duplicate
    if model.get_level(&id).is_some() {
        return Err(AppError::ElementAlreadyExists(format!("level: {}", id)));
    }

    let level = LevelDefinition {
        id: id.clone(),
        name: name.unwrap_or_else(|| id.clone()),
        description: desc,
        can_contain: if can_contain.is_empty() { vec![] } else { can_contain },
    };

    model.hierarchy.levels.push(level);
    model.touch();
    workspace.touch();

    YamlStore::save_workspace(src, &workspace)?;
    println!("{} level: {}", "Added".green(), id);
    Ok(())
}

fn execute_remove_level(src: &Path, id: &str) -> Result<()> {
    let mut workspace = load_workspace(src)?;

    let model = workspace.logic_architecture_concept_model.as_mut().ok_or_else(|| {
        AppError::InvalidOperation("Concept model not found. Run 'init' first.".to_string())
    })?;

    let idx = model.hierarchy.levels.iter().position(|l| l.id == id).ok_or_else(|| {
        AppError::ElementNotFound(format!("level: {}", id))
    })?;

    model.hierarchy.levels.remove(idx);
    model.touch();
    workspace.touch();

    YamlStore::save_workspace(src, &workspace)?;
    println!("{} level: {}", "Removed".yellow(), id);
    Ok(())
}

fn execute_add_containment(src: &Path, parent: &str, child: &str) -> Result<()> {
    let mut workspace = load_workspace(src)?;

    // Auto-initialize concept model if it doesn't exist
    if workspace.logic_architecture_concept_model.is_none() {
        let title = "Logic Architecture Concept Model".to_string();
        let mut model = LogicArchitectureConceptModel::new(&title);
        // Add "system" as a built-in element type
        model.add_element_type("system");
        workspace.logic_architecture_concept_model = Some(model);
        println!("{} concept model", "Initialized".green());
    }

    let model = workspace.logic_architecture_concept_model.as_mut().unwrap();

    // Check that both parent and child element types exist
    if !model.has_element_type(parent) {
        return Err(AppError::ElementNotFound(format!("element type: {}", parent)));
    }
    if !model.has_element_type(child) {
        return Err(AppError::ElementNotFound(format!("element type: {}", child)));
    }

    if !model.add_containment(parent, child) {
        return Err(AppError::ElementAlreadyExists(format!("containment: {} -> {}", parent, child)));
    }
    workspace.touch();

    YamlStore::save_workspace(src, &workspace)?;
    println!("{} containment: {} -> {}", "Added".green(), parent.to_uppercase(), child.to_uppercase());
    Ok(())
}

fn execute_add_element(src: &Path, type_name: &str) -> Result<()> {
    let mut workspace = load_workspace(src)?;

    // Auto-initialize concept model if it doesn't exist
    if workspace.logic_architecture_concept_model.is_none() {
        let title = "Logic Architecture Concept Model".to_string();
        let mut model = LogicArchitectureConceptModel::new(&title);
        // Add "system" as a built-in element type
        model.add_element_type("system");
        workspace.logic_architecture_concept_model = Some(model);
        println!("{} concept model", "Initialized".green());
    }

    let model = workspace.logic_architecture_concept_model.as_mut().unwrap();

    if !model.add_element_type(type_name) {
        return Err(AppError::ElementAlreadyExists(format!("element type: {}", type_name)));
    }
    workspace.touch();

    YamlStore::save_workspace(src, &workspace)?;
    println!("{} element type: {}", "Added".green(), type_name.to_uppercase());
    Ok(())
}

fn execute_remove_element(src: &Path, type_name: &str) -> Result<()> {
    let mut workspace = load_workspace(src)?;

    let model = workspace.logic_architecture_concept_model.as_mut().ok_or_else(|| {
        AppError::InvalidOperation("Concept model not found. Run 'init' first.".to_string())
    })?;

    if !model.remove_element_type(type_name) {
        return Err(AppError::ElementNotFound(format!("element type: {}", type_name)));
    }
    workspace.touch();

    YamlStore::save_workspace(src, &workspace)?;
    println!("{} element type: {}", "Removed".yellow(), type_name.to_uppercase());
    Ok(())
}

fn execute_list(src: &Path) -> Result<()> {
    let workspace = load_workspace(src)?;

    let model = workspace.logic_architecture_concept_model.as_ref().ok_or_else(|| {
        AppError::InvalidOperation("Concept model not found. Run 'init' first.".to_string())
    })?;

    println!("{}", "Hierarchy Levels:".cyan().bold());
    println!();

    for level in &model.hierarchy.levels {
        println!("  {} ({})", level.id.green(), level.name);
        if let Some(ref desc) = level.description {
            println!("    Description: {}", desc);
        }
        if !level.can_contain.is_empty() {
            println!("    Can contain: {}", level.can_contain.join(", "));
        }
        println!();
    }

    Ok(())
}

fn execute_show(src: &Path, id: &str) -> Result<()> {
    let workspace = load_workspace(src)?;

    let model = workspace.logic_architecture_concept_model.as_ref().ok_or_else(|| {
        AppError::InvalidOperation("Concept model not found. Run 'init' first.".to_string())
    })?;

    let level = model.get_level(id).ok_or_else(|| {
        AppError::ElementNotFound(format!("level: {}", id))
    })?;

    println!("{}: {}", "Level".cyan(), level.id);
    println!("  Name: {}", level.name);
    if let Some(ref desc) = level.description {
        println!("  Description: {}", desc);
    }
    if !level.can_contain.is_empty() {
        println!("  Can contain: {}", level.can_contain.join(", "));
    }

    Ok(())
}
