use crate::cli::args::{LogicAddCommand, LogicListElement, LogicModelCommand, LogicRemoveCommand};
use crate::store::{LoadedLogic, LogicOperations, YamlStore};
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::Path;

pub fn execute(src: &Path, cmd: LogicModelCommand) -> Result<()> {
    match cmd {
        LogicModelCommand::Add(add_cmd) => execute_add(src, add_cmd),
        LogicModelCommand::Remove(remove_cmd) => execute_remove(src, remove_cmd),
        LogicModelCommand::List { element } => execute_list(src, element),
        LogicModelCommand::Show { id } => execute_show(src, &id),
    }
}

/// Check if an element type is allowed in the concept model
/// Fails if:
/// 1. Concept model doesn't exist (run 'concept-model init' first)
/// 2. Concept model exists but element type is not defined
fn check_element_type_allowed(loaded: &LoadedLogic, type_name: &str) -> Result<()> {
    match loaded {
        LoadedLogic::Workspace { workspace, .. } => {
            match &workspace.logic_architecture_concept_model {
                None => {
                    // No concept model - must initialize first
                    return Err(AppError::InvalidOperation(
                        "Logic Architecture Concept Model not initialized. Run 'concept-model init' first.".to_string()
                    ));
                }
                Some(model) => {
                    // Concept model exists - check if element type is defined
                    if !model.has_element_type(type_name) {
                        return Err(AppError::InvalidOperation(format!(
                            "Element type '{}' is not defined in the concept model. Add it first with 'concept-model add element {}'",
                            type_name.to_uppercase(),
                            type_name
                        )));
                    }
                }
            }
        }
        LoadedLogic::Diagram(_) => {
            // Standalone diagram - no concept model validation required
        }
    }
    Ok(())
}

fn execute_add(src: &Path, cmd: LogicAddCommand) -> Result<()> {
    // Load or create diagram (workspace-aware)
    let (loaded, mut diagram) = if YamlStore::exists(src) {
        let loaded = YamlStore::load_logic_any(src)?;
        let diagram = match &loaded {
            LoadedLogic::Workspace { workspace, has_logic_view } => {
                if *has_logic_view {
                    workspace.logic_view.clone().unwrap()
                } else {
                    // Workspace exists but no logic view - try to create one from context diagram
                    if let Some(ref context) = workspace.context_diagram {
                        // Use the context diagram's system ID for the logic view
                        let system_id = &context.system.id;
                        let title = format!("{} Logic Concept Diagram", system_id);
                        crate::model::logic::concept::LogicConceptDiagram::new(system_id, &title)
                    } else {
                        // No context diagram either - need explicit system command
                        match &cmd {
                            LogicAddCommand::System { id, .. } => {
                                let title = format!("{} Logic Concept Diagram", id);
                                crate::model::logic::concept::LogicConceptDiagram::new(id, &title)
                            }
                            _ => {
                                return Err(crate::utils::error::AppError::InvalidOperation(
                                    format!("Logic view does not exist in workspace. Create it first with 'add system' command."),
                                ));
                            }
                        }
                    }
                }
            }
            LoadedLogic::Diagram(d) => d.clone(),
        };
        (loaded, diagram)
    } else {
        // Only allow auto-create for System command
        match &cmd {
            LogicAddCommand::System { id, .. } => {
                // Auto-create with the given system ID
                let title = format!("{} Logic Concept Diagram", id);
                let new_diagram = crate::model::logic::concept::LogicConceptDiagram::new(id, &title);
                println!("{} new project file: {}", "Created".green(), src.display());
                (LoadedLogic::Diagram(new_diagram.clone()), new_diagram)
            }
            _ => {
                return Err(crate::utils::error::AppError::InvalidOperation(
                    format!("File '{}' does not exist. Create it first with 'add system' command.", src.display()),
                ));
            }
        }
    };

    // Keep a reference to loaded for validation
    let loaded_ref = if YamlStore::exists(src) {
        YamlStore::load_logic_any(src)?
    } else {
        LoadedLogic::Diagram(diagram.clone())
    };

    match cmd {
        LogicAddCommand::System { id, name, desc } => {
            // System is always allowed (it's the root)
            diagram.system.id = id.clone();
            diagram.system.name = name.unwrap_or(id.clone());
            diagram.system.description = desc;
            diagram.touch();
            println!("{} system: {}", "Added/Updated".green(), id);
        }
        LogicAddCommand::Subsystem { id, name, desc } => {
            check_element_type_allowed(&loaded_ref, "SUBSYSTEM")?;
            LogicOperations::add_subsystem(&mut diagram, &id, name.as_deref(), desc.as_deref())?;
            println!("{} subsystem: {}", "Added".green(), id);
        }
        LogicAddCommand::Component { id, name, desc, subsystem } => {
            check_element_type_allowed(&loaded_ref, "COMPONENT")?;
            // For now, add to system.components directly (subsystem support would require extending LogicOperations)
            if subsystem.is_some() {
                return Err(AppError::InvalidOperation(
                    "Adding components to subsystems is not yet supported".to_string()
                ));
            }
            LogicOperations::add_component(&mut diagram, &id, name.as_deref(), desc.as_deref())?;
            println!("{} component: {}", "Added".green(), id);
        }
        LogicAddCommand::Module { component_id, id, name, desc, parent } => {
            check_element_type_allowed(&loaded_ref, "MODULE")?;
            if let Some(parent_id) = parent {
                LogicOperations::add_nested_module(
                    &mut diagram, &component_id, &parent_id, &id,
                    name.as_deref(), desc.as_deref()
                )?;
                println!("{} nested module: {} under {}", "Added".green(), id, parent_id);
            } else {
                LogicOperations::add_module(
                    &mut diagram, &component_id, &id,
                    name.as_deref(), desc.as_deref()
                )?;
                println!("{} module: {} in component {}", "Added".green(), id, component_id);
            }
        }
        LogicAddCommand::Interface { component_id, module_id, id, name, desc } => {
            LogicOperations::add_interface(
                &mut diagram, &component_id, &module_id, &id,
                name.as_deref(), desc.as_deref()
            )?;
            println!("{} interface: {}", "Added".green(), id);
        }
        LogicAddCommand::Dependency { component_id, module_id, interface_id } => {
            LogicOperations::add_dependency(
                &mut diagram, &component_id, &module_id, &interface_id
            )?;
            println!("{} dependency: {} -> {}", "Added".green(), module_id, interface_id);
        }
        LogicAddCommand::Expose { component_id, interface_id } => {
            LogicOperations::expose_interface(&mut diagram, &component_id, &interface_id)?;
            println!("{} exposed interface: {} from {}", "Added".green(), interface_id, component_id);
        }
    }

    YamlStore::save_logic_any(src, &loaded, &diagram)?;
    Ok(())
}

fn get_diagram_from_loaded(loaded: &LoadedLogic) -> Result<crate::model::logic::concept::LogicConceptDiagram> {
    match loaded {
        LoadedLogic::Workspace { workspace, has_logic_view } => {
            if *has_logic_view {
                Ok(workspace.logic_view.clone().unwrap())
            } else {
                Err(crate::utils::error::AppError::ElementNotFound(
                    "logic_view not found in workspace".to_string()
                ))
            }
        }
        LoadedLogic::Diagram(d) => Ok(d.clone()),
    }
}

fn execute_remove(src: &Path, cmd: LogicRemoveCommand) -> Result<()> {
    let loaded = YamlStore::load_logic_any(src)?;
    let mut diagram = get_diagram_from_loaded(&loaded)?;

    match cmd {
        LogicRemoveCommand::Subsystem { id } => {
            LogicOperations::remove_subsystem(&mut diagram, &id)?;
            println!("{} subsystem: {}", "Removed".yellow(), id);
        }
        LogicRemoveCommand::Component { id } => {
            LogicOperations::remove_component(&mut diagram, &id)?;
            println!("{} component: {}", "Removed".yellow(), id);
        }
        LogicRemoveCommand::Module { component_id, id } => {
            LogicOperations::remove_module(&mut diagram, &component_id, &id)?;
            println!("{} module: {}", "Removed".yellow(), id);
        }
        LogicRemoveCommand::Interface { component_id, module_id, id } => {
            LogicOperations::remove_interface(&mut diagram, &component_id, &module_id, &id)?;
            println!("{} interface: {}", "Removed".yellow(), id);
        }
        LogicRemoveCommand::Dependency { component_id, module_id, interface_id } => {
            LogicOperations::remove_dependency(&mut diagram, &component_id, &module_id, &interface_id)?;
            println!("{} dependency: {} -> {}", "Removed".yellow(), module_id, interface_id);
        }
    }

    YamlStore::save_logic_any(src, &loaded, &diagram)?;
    Ok(())
}

fn execute_list(src: &Path, element: LogicListElement) -> Result<()> {
    let loaded = YamlStore::load_logic_any(src)?;
    let diagram = get_diagram_from_loaded(&loaded)?;

    match element {
        LogicListElement::System => {
            println!("{}: {}", "System".cyan(), diagram.system.id);
            println!("  Name: {}", diagram.system.name);
            if let Some(ref desc) = diagram.system.description {
                println!("  Description: {}", desc);
            }
        }
        LogicListElement::Subsystems => {
            println!("{}", "Subsystems:".cyan());
            if diagram.system.subsystems.is_empty() {
                println!("  (none)");
            } else {
                for sub in &diagram.system.subsystems {
                    println!("  - {} ({})", sub.id, sub.name);
                    if let Some(ref desc) = sub.description {
                        println!("    Description: {}", desc);
                    }
                }
            }
        }
        LogicListElement::Components => {
            println!("{}", "Components (in system):".cyan());
            if diagram.system.components.is_empty() {
                println!("  (none)");
            } else {
                for comp in &diagram.system.components {
                    println!("  - {} ({})", comp.id, comp.name);
                    if !comp.modules.is_empty() {
                        println!("    Modules: {}", comp.modules.iter().map(|m| m.id.as_str()).collect::<Vec<_>>().join(", "));
                    }
                }
            }
            for sub in &diagram.system.subsystems {
                if !sub.components.is_empty() {
                    println!("{} (in subsystem {}):", "Components".cyan(), sub.id);
                    for comp in &sub.components {
                        println!("  - {} ({})", comp.id, comp.name);
                    }
                }
            }
        }
        LogicListElement::Modules => {
            println!("{}", "Modules:".cyan());
            let mut found = false;
            for comp in &diagram.system.components {
                for module in &comp.modules {
                    found = true;
                    list_module(module, "  ", &comp.id);
                }
            }
            if !found {
                println!("  (none)");
            }
        }
        LogicListElement::Interfaces => {
            println!("{}", "Interfaces:".cyan());
            let interfaces = diagram.all_interface_ids();
            if interfaces.is_empty() {
                println!("  (none)");
            } else {
                for iface_id in interfaces {
                    if let Some(iface) = diagram.find_interface(iface_id) {
                        println!("  - {} ({})", iface.id, iface.name);
                    }
                }
            }
        }
        LogicListElement::Dependencies => {
            println!("{}", "Dependencies:".cyan());
            let mut found = false;
            for comp in &diagram.system.components {
                for module in &comp.modules {
                    for dep_id in &module.dependencies {
                        found = true;
                        println!("  - {} -> {}", module.id, dep_id);
                    }
                    // Also check nested modules
                    list_module_deps(&module.modules, "  ", &mut found);
                }
            }
            if !found {
                println!("  (none)");
            }
        }
    }

    Ok(())
}

fn list_module(module: &crate::model::logic::concept::Module, indent: &str, path: &str) {
    let new_path = format!("{}.{}", path, module.id);
    println!("{}- {} ({}) [{}]", indent, module.id, module.name, new_path);
    for m in &module.modules {
        list_module(m, &format!("  {}", indent), &new_path);
    }
}

fn list_module_deps(
    modules: &[crate::model::logic::concept::Module],
    indent: &str,
    found: &mut bool,
) {
    for m in modules {
        for dep_id in &m.dependencies {
            *found = true;
            println!("{}- {} -> {}", indent, m.id, dep_id);
        }
        list_module_deps(&m.modules, indent, found);
    }
}

fn execute_show(src: &Path, id: &str) -> Result<()> {
    let loaded = YamlStore::load_logic_any(src)?;
    let diagram = get_diagram_from_loaded(&loaded)?;

    // Check system
    if diagram.system.id == id {
        println!("{}: {}", "System".cyan(), diagram.system.id);
        println!("  Name: {}", diagram.system.name);
        if let Some(ref desc) = diagram.system.description {
            println!("  Description: {}", desc);
        }
        println!("  Subsystems: {}", diagram.system.subsystems.len());
        println!("  Components: {}", diagram.system.components.len());
        return Ok(());
    }

    // Check subsystems
    if let Some(sub) = diagram.system.subsystems.iter().find(|s| s.id == id) {
        println!("{}: {}", "Subsystem".cyan(), sub.id);
        println!("  Name: {}", sub.name);
        if let Some(ref desc) = sub.description {
            println!("  Description: {}", desc);
        }
        println!("  Components: {}", sub.components.len());
        return Ok(());
    }

    // Check components
    for comp in &diagram.system.components {
        if comp.id == id {
            show_component(comp);
            return Ok(());
        }
    }

    // Check components in subsystems
    for sub in &diagram.system.subsystems {
        for comp in &sub.components {
            if comp.id == id {
                show_component(comp);
                return Ok(());
            }
        }
    }

    // Check interfaces
    if let Some(iface) = diagram.find_interface(id) {
        println!("{}: {}", "Interface".cyan(), iface.id);
        println!("  Name: {}", iface.name);
        if let Some(ref desc) = iface.description {
            println!("  Description: {}", desc);
        }
        return Ok(());
    }

    Err(crate::utils::error::AppError::ElementNotFound(id.to_string()))
}

fn show_component(comp: &crate::model::logic::concept::Component) {
    println!("{}: {}", "Component".cyan(), comp.id);
    println!("  Name: {}", comp.name);
    if let Some(ref desc) = comp.description {
        println!("  Description: {}", desc);
    }
    println!("  Modules:");
    for module in &comp.modules {
        show_module(module, "    ");
    }
    if !comp.exposed_interfaces.is_empty() {
        println!("  Exposed interfaces: {}", comp.exposed_interfaces.join(", "));
    }
}

fn show_module(module: &crate::model::logic::concept::Module, indent: &str) {
    println!("{}- {} ({})", indent, module.id, module.name);
    if !module.interfaces.is_empty() {
        println!("{}  Interfaces: {}", indent, module.interfaces.iter().map(|i| i.id.as_str()).collect::<Vec<_>>().join(", "));
    }
    if !module.dependencies.is_empty() {
        println!("{}  Dependencies: {}", indent, module.dependencies.join(", "));
    }
    for m in &module.modules {
        show_module(m, &format!("{}  ", indent));
    }
}
