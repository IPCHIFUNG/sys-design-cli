use crate::cli::args::{LogicAddCommand, LogicListElement, LogicModelCommand, LogicRemoveCommand};
use crate::model::workspace::Workspace;
use crate::store::{LogicOperations, YamlStore};
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
fn check_element_type_allowed(workspace: &Workspace, type_name: &str) -> Result<()> {
    match &workspace.logic_architecture_concept_model {
        None => {
            // No concept model - must initialize first
            Err(AppError::InvalidOperation(
                "Logic Architecture Concept Model not initialized. Run 'concept-model init' first.".to_string()
            ))
        }
        Some(model) => {
            // Concept model exists - check if element type is defined
            if !model.has_element_type(type_name) {
                Err(AppError::InvalidOperation(format!(
                    "Element type '{}' is not defined in the concept model. Add it first with 'concept-model add element {}'",
                    type_name.to_uppercase(),
                    type_name
                )))
            } else {
                Ok(())
            }
        }
    }
}

fn execute_add(src: &Path, cmd: LogicAddCommand) -> Result<()> {
    // Load or create workspace
    let (mut workspace, is_new_file) = if YamlStore::exists(src) {
        (YamlStore::load_workspace_any(src)?, false)
    } else {
        // Only allow auto-create for System command
        match &cmd {
            LogicAddCommand::System { id, .. } => {
                let title = format!("{} Logic Concept Diagram", id);
                let mut ws = Workspace::new(&title);
                ws.logic_view = Some(crate::model::logic::concept::LogicConceptDiagram::new(id, &title));
                println!("{} new project file: {}", "Created".green(), src.display());
                (ws, true)
            }
            _ => {
                return Err(crate::utils::error::AppError::InvalidOperation(
                    format!("File '{}' does not exist. Create it first with 'add system' command.", src.display()),
                ));
            }
        }
    };

    // Get or create logic view
    let mut diagram = match &cmd {
        LogicAddCommand::System { .. } if is_new_file => {
            workspace.logic_view.clone().unwrap()
        }
        _ => {
            if workspace.logic_view.is_some() {
                workspace.logic_view.clone().unwrap()
            } else {
                // Workspace exists but no logic view - try to create one from context diagram
                if let Some(ref context) = workspace.context_diagram {
                    let system_id = &context.system.id;
                    let title = format!("{} Logic Concept Diagram", system_id);
                    crate::model::logic::concept::LogicConceptDiagram::new(system_id, &title)
                } else {
                    match &cmd {
                        LogicAddCommand::System { id, .. } => {
                            let title = format!("{} Logic Concept Diagram", id);
                            crate::model::logic::concept::LogicConceptDiagram::new(id, &title)
                        }
                        _ => {
                            return Err(crate::utils::error::AppError::InvalidOperation(
                                "Logic view does not exist in workspace. Create it first with 'add system' command.".to_string(),
                            ));
                        }
                    }
                }
            }
        }
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
            check_element_type_allowed(&workspace, "SUBSYSTEM")?;
            LogicOperations::add_subsystem(&mut diagram, &id, name.as_deref(), desc.as_deref())?;
            println!("{} subsystem: {}", "Added".green(), id);
        }
        LogicAddCommand::Component { id, name, desc, subsystem } => {
            check_element_type_allowed(&workspace, "COMPONENT")?;
            // For now, add to system.components directly (subsystem support would require extending LogicOperations)
            if subsystem.is_some() {
                return Err(AppError::InvalidOperation(
                    "Adding components to subsystems is not yet supported".to_string()
                ));
            }
            LogicOperations::add_component(&mut diagram, &id, name.as_deref(), desc.as_deref())?;
            println!("{} component: {}", "Added".green(), id);
        }
        LogicAddCommand::Module { id, name, desc } => {
            check_element_type_allowed(&workspace, "MODULE")?;
            // Add module as standalone element (at system level)
            LogicOperations::add_module_to_system(
                &mut diagram, &id,
                name.as_deref(), desc.as_deref()
            )?;
            println!("{} module: {}", "Added".green(), id);
        }
        LogicAddCommand::Submodule { id, name, desc } => {
            check_element_type_allowed(&workspace, "SUBMODULE")?;
            // Add submodule as standalone element (at system level)
            LogicOperations::add_submodule(
                &mut diagram, &id,
                name.as_deref(), desc.as_deref()
            )?;
            println!("{} submodule: {}", "Added".green(), id);
        }
        LogicAddCommand::Element { type_name, id, name, desc } => {
            let type_upper = type_name.to_uppercase();
            check_element_type_allowed(&workspace, &type_upper)?;
            // Route to appropriate add method based on type
            match type_upper.as_str() {
                "SUBSYSTEM" => {
                    LogicOperations::add_subsystem(&mut diagram, &id, name.as_deref(), desc.as_deref())?;
                    println!("{} subsystem: {}", "Added".green(), id);
                }
                "COMPONENT" => {
                    LogicOperations::add_component(&mut diagram, &id, name.as_deref(), desc.as_deref())?;
                    println!("{} component: {}", "Added".green(), id);
                }
                "MODULE" => {
                    LogicOperations::add_module_to_system(&mut diagram, &id, name.as_deref(), desc.as_deref())?;
                    println!("{} module: {}", "Added".green(), id);
                }
                "SUBMODULE" => {
                    LogicOperations::add_submodule(&mut diagram, &id, name.as_deref(), desc.as_deref())?;
                    println!("{} submodule: {}", "Added".green(), id);
                }
                _ => {
                    return Err(AppError::InvalidOperation(format!(
                        "Unknown element type: {}",
                        type_name
                    )));
                }
            }
        }
        LogicAddCommand::Interface { id, name, desc } => {
            // Add standalone interface at system level
            LogicOperations::add_standalone_interface(
                &mut diagram, &id,
                name.as_deref(), desc.as_deref()
            )?;
            println!("{} interface: {}", "Added".green(), id);
        }
        LogicAddCommand::ProvideRelation { element_id, interface_id } => {
            LogicOperations::add_provide_relation(&mut diagram, &element_id, &interface_id)?;
            println!("{} provide relation: {} -> {}", "Added".green(), element_id, interface_id);
        }
        LogicAddCommand::Containment { parent_id, child_id } => {
            // Validate containment follows concept model
            validate_containment_against_concept_model(&workspace, &parent_id, &child_id)?;
            LogicOperations::add_element_containment(&mut diagram, &parent_id, &child_id)?;
            println!("{} containment: {} -> {}", "Added".green(), parent_id, child_id);
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

    YamlStore::save_logic_to_workspace(src, &mut workspace, &diagram)?;
    Ok(())
}

fn get_diagram_from_workspace(workspace: &Workspace) -> Result<crate::model::logic::concept::LogicConceptDiagram> {
    workspace.logic_view.clone().ok_or_else(|| {
        crate::utils::error::AppError::ElementNotFound(
            "logic_view not found in workspace".to_string()
        )
    })
}

fn execute_remove(src: &Path, cmd: LogicRemoveCommand) -> Result<()> {
    let mut workspace = YamlStore::load_workspace_any(src)?;
    let mut diagram = get_diagram_from_workspace(&workspace)?;

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

    YamlStore::save_logic_to_workspace(src, &mut workspace, &diagram)?;
    Ok(())
}

fn execute_list(src: &Path, element: LogicListElement) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(src)?;
    let diagram = get_diagram_from_workspace(&workspace)?;

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
    let workspace = YamlStore::load_workspace_any(src)?;
    let diagram = get_diagram_from_workspace(&workspace)?;

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

/// Validate that a containment relationship is allowed by the concept model
fn validate_containment_against_concept_model(
    workspace: &Workspace,
    parent_id: &str,
    child_id: &str,
) -> Result<()> {
    // Get concept model from workspace
    let concept_model = match &workspace.logic_architecture_concept_model {
        Some(model) => model,
        None => {
            // No concept model - can't validate
            return Err(AppError::InvalidOperation(
                "Logic Architecture Concept Model not initialized. Run 'concept-model init' first.".to_string()
            ));
        }
    };

    // Get logic view to find element types
    let logic = match &workspace.logic_view {
        Some(lv) => lv,
        None => {
            // No logic view yet, can't determine element types
            return Ok(());
        }
    };

    // Get parent and child element types
    let parent_type = get_element_type(logic, parent_id);
    let child_type = get_element_type(logic, child_id);

    match (parent_type, child_type) {
        (Some(parent_t), Some(child_t)) => {
            // Check if concept model allows this containment
            if !concept_model.has_containment(&parent_t, &child_t) {
                return Err(AppError::InvalidOperation(format!(
                    "Concept model does not allow '{}' to contain '{}'. Add containment with 'concept-model add containment {} {}'",
                    parent_t, child_t, parent_t.to_lowercase(), child_t.to_lowercase()
                )));
            }
            Ok(())
        }
        (None, _) => Err(AppError::ElementNotFound(format!("parent element: {}", parent_id))),
        (_, None) => Err(AppError::ElementNotFound(format!("child element: {}", child_id))),
    }
}

/// Get the element type for a given element ID
fn get_element_type(
    logic: &crate::model::logic::concept::LogicConceptDiagram,
    id: &str,
) -> Option<String> {
    // Check if it's the system
    if logic.system.id == id {
        return Some("SYSTEM".to_string());
    }

    // Check subsystems
    if logic.system.subsystems.iter().any(|s| s.id == id) {
        return Some("SUBSYSTEM".to_string());
    }

    // Check components
    if logic.system.components.iter().any(|c| c.id == id) {
        return Some("COMPONENT".to_string());
    }
    for sub in &logic.system.subsystems {
        if sub.components.iter().any(|c| c.id == id) {
            return Some("COMPONENT".to_string());
        }
    }

    // Check if it's explicitly marked as a submodule
    if logic.submodule_ids.iter().any(|sid| sid == id) {
        return Some("SUBMODULE".to_string());
    }

    // Check modules (at system level)
    if logic.system.modules.iter().any(|m| m.id == id) {
        return Some("MODULE".to_string());
    }

    // Check modules in components
    for comp in &logic.system.components {
        if let Some(is_nested) = find_module_type(&comp.modules, id) {
            return Some(if is_nested { "SUBMODULE" } else { "MODULE" }.to_string());
        }
    }

    // Check modules in subsystems
    for sub in &logic.system.subsystems {
        for comp in &sub.components {
            if let Some(is_nested) = find_module_type(&comp.modules, id) {
                return Some(if is_nested { "SUBMODULE" } else { "MODULE" }.to_string());
            }
        }
    }

    // Check if it's an interface
    if logic.find_interface(id).is_some() {
        return Some("INTERFACE".to_string());
    }

    None
}

/// Recursively find a module and return whether it's a nested module (submodule)
/// Returns Some(true) if found as nested, Some(false) if found at top level, None if not found
fn find_module_type(
    modules: &[crate::model::logic::concept::Module],
    id: &str,
) -> Option<bool> {
    for m in modules {
        if m.id == id {
            return Some(false); // Found at current level (not nested relative to this call)
        }
        // Check nested modules (these would be submodules)
        if find_module_type(&m.modules, id).is_some() {
            return Some(true); // Found nested, so it's a submodule
        }
    }
    None
}
