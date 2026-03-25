use crate::cli::args::{AddCommand, ContextModelCommand, ListElement, RemoveCommand};
use crate::model::workspace::Workspace;
use crate::store::{Operations, YamlStore};
use crate::utils::error::Result;
use colored::Colorize;
use std::path::Path;

pub fn execute(src: &Path, cmd: ContextModelCommand) -> Result<()> {
    match cmd {
        ContextModelCommand::Add(add_cmd) => execute_add(src, add_cmd),
        ContextModelCommand::Remove(remove_cmd) => execute_remove(src, remove_cmd),
        ContextModelCommand::List { element } => execute_list(src, element),
        ContextModelCommand::Show { id } => execute_show(src, &id),
    }
}

fn execute_add(src: &Path, cmd: AddCommand) -> Result<()> {
    // Load or create workspace
    let (mut workspace, is_new_file) = if YamlStore::exists(src) {
        (YamlStore::load_workspace_any(src)?, false)
    } else {
        // Only allow auto-create for System command
        match &cmd {
            AddCommand::System { id, .. } => {
                let title = format!("{} Context Diagram", id);
                let mut ws = Workspace::new(&title);
                ws.context_diagram = Some(crate::model::c4::context::ContextDiagram::new(id, &title));
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

    // Get or create context diagram
    let mut diagram = match &cmd {
        AddCommand::System { .. } if is_new_file => {
            workspace.context_diagram.clone().unwrap()
        }
        _ => {
            if workspace.context_diagram.is_some() {
                workspace.context_diagram.clone().unwrap()
            } else {
                // Workspace exists but no context diagram - need to create one
                match &cmd {
                    AddCommand::System { id, .. } => {
                        let title = format!("{} Context Diagram", id);
                        crate::model::c4::context::ContextDiagram::new(id, &title)
                    }
                    _ => {
                        return Err(crate::utils::error::AppError::InvalidOperation(
                            "Context diagram does not exist in workspace. Create it first with 'add system' command.".to_string(),
                        ));
                    }
                }
            }
        }
    };

    match cmd {
        AddCommand::System { id, name, desc } => {
            if is_new_file {
                // Update name/description if provided
                if name.is_some() || desc.is_some() {
                    if let Some(n) = name {
                        diagram.system.name = n.to_string();
                    }
                    if let Some(d) = desc {
                        diagram.system.description = Some(d.to_string());
                    }
                    diagram.touch();
                }
                println!("{} system: {}", "Added".green(), id);
            } else {
                Operations::set_system(&mut diagram, &id, name.as_deref(), desc.as_deref())?;
                println!("{} system: {}", "Added".green(), id);
            }
        }
        AddCommand::Actor {
            id,
            name,
            desc,
            actor_type,
        } => {
            Operations::add_actor(
                &mut diagram,
                &id,
                name.as_deref(),
                desc.as_deref(),
                actor_type.into(),
            )?;
            println!("{} actor: {}", "Added".green(), id);
        }
        AddCommand::ExternalSystem {
            id,
            name,
            desc,
            tech,
        } => {
            Operations::add_external_system(
                &mut diagram,
                &id,
                name.as_deref(),
                desc.as_deref(),
                tech.as_deref(),
            )?;
            println!("{} external system: {}", "Added".green(), id);
        }
        AddCommand::Interface {
            id,
            name,
            desc,
            protocol,
        } => {
            Operations::add_interface(
                &mut diagram,
                &id,
                name.as_deref(),
                desc.as_deref(),
                protocol.into(),
            )?;
            println!("{} interface: {}", "Added".green(), id);
        }
        AddCommand::ProvideRelation {
            system_id,
            interface_id,
        } => {
            Operations::add_provide_relation(&mut diagram, &system_id, &interface_id)?;
            println!(
                "{} provide relation: {} -> {}",
                "Added".green(),
                system_id,
                interface_id
            );
        }
        AddCommand::InterfaceUsage {
            actor_id,
            interface_id,
        } => {
            Operations::add_interface_usage(&mut diagram, &actor_id, &interface_id)?;
            println!(
                "{} interface usage: {} -> {}",
                "Added".green(),
                actor_id,
                interface_id
            );
        }
    }

    YamlStore::save_context_to_workspace(src, &mut workspace, &diagram)?;
    Ok(())
}

fn execute_remove(src: &Path, cmd: RemoveCommand) -> Result<()> {
    let mut workspace = YamlStore::load_workspace_any(src)?;
    let mut diagram = workspace.context_diagram.clone().ok_or_else(|| {
        crate::utils::error::AppError::ElementNotFound(
            "context_diagram not found in workspace".to_string()
        )
    })?;

    match cmd {
        RemoveCommand::Actor { id } => {
            Operations::remove_actor(&mut diagram, &id)?;
            println!("{} actor: {}", "Removed".yellow(), id);
        }
        RemoveCommand::ExternalSystem { id } => {
            Operations::remove_external_system(&mut diagram, &id)?;
            println!("{} external system: {}", "Removed".yellow(), id);
        }
        RemoveCommand::Interface { id } => {
            Operations::remove_interface(&mut diagram, &id)?;
            println!("{} interface: {}", "Removed".yellow(), id);
        }
        RemoveCommand::ProvideRelation {
            system_id,
            interface_id,
        } => {
            Operations::remove_provide_relation(&mut diagram, &system_id, &interface_id)?;
            println!(
                "{} provide relation: {} -> {}",
                "Removed".yellow(),
                system_id,
                interface_id
            );
        }
        RemoveCommand::InterfaceUsage {
            actor_id,
            interface_id,
        } => {
            Operations::remove_interface_usage(&mut diagram, &actor_id, &interface_id)?;
            println!(
                "{} interface usage: {} -> {}",
                "Removed".yellow(),
                actor_id,
                interface_id
            );
        }
    }

    YamlStore::save_context_to_workspace(src, &mut workspace, &diagram)?;
    Ok(())
}

fn get_diagram_from_workspace(workspace: &Workspace) -> Result<crate::model::c4::context::ContextDiagram> {
    workspace.context_diagram.clone().ok_or_else(|| {
        crate::utils::error::AppError::ElementNotFound(
            "context_diagram not found in workspace".to_string()
        )
    })
}

fn execute_list(src: &Path, element: ListElement) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(src)?;
    let diagram = get_diagram_from_workspace(&workspace)?;

    match element {
        ListElement::System => {
            println!("{}: {}", "System".cyan(), diagram.system.id);
            println!("  Name: {}", diagram.system.name);
            if let Some(ref desc) = diagram.system.description {
                println!("  Description: {}", desc);
            }
        }
        ListElement::Actors => {
            println!("{}", "Actors:".cyan());
            if diagram.actors.is_empty() {
                println!("  (none)");
            } else {
                for actor in &diagram.actors {
                    let type_str = match actor.actor_type {
                        crate::model::c4::context::ActorType::External => "external",
                        crate::model::c4::context::ActorType::Internal => "internal",
                    };
                    println!("  - {} ({}) [{}]", actor.id, actor.name, type_str);
                }
            }
        }
        ListElement::ExternalSystems => {
            println!("{}", "External Systems:".cyan());
            if diagram.external_systems.is_empty() {
                println!("  (none)");
            } else {
                for ext in &diagram.external_systems {
                    println!("  - {} ({})", ext.id, ext.name);
                    if let Some(ref tech) = ext.technology {
                        println!("    Technology: {}", tech);
                    }
                }
            }
        }
        ListElement::Interfaces => {
            println!("{}", "Interfaces:".cyan());
            if diagram.interfaces.is_empty() {
                println!("  (none)");
            } else {
                for interface in &diagram.interfaces {
                    let protocol = match &interface.protocol {
                        crate::model::c4::context::Protocol::Rest => "REST",
                        crate::model::c4::context::Protocol::Grpc => "gRPC",
                        crate::model::c4::context::Protocol::Graphql => "GraphQL",
                        crate::model::c4::context::Protocol::WebSocket => "WebSocket",
                        crate::model::c4::context::Protocol::Mqtt => "MQTT",
                        crate::model::c4::context::Protocol::Amqp => "AMQP",
                        crate::model::c4::context::Protocol::Custom(s) => s,
                    };
                    println!("  - {} ({}) [{}]", interface.id, interface.name, protocol);
                }
            }
        }
        ListElement::Relations => {
            println!("{}", "Derived Relations:".cyan());
            let relations = diagram.derive_relationships();
            if relations.is_empty() {
                println!("  (none)");
            } else {
                for rel in relations {
                    let from_name = diagram.get_element_name(&rel.from).unwrap_or(&rel.from);
                    let to_name = diagram.get_element_name(&rel.to).unwrap_or(&rel.to);
                    println!(
                        "  - {} ({}) -> {} ({}) [via: {}]",
                        rel.from, from_name, rel.to, to_name, rel.via_interface
                    );
                }
            }
        }
    }

    Ok(())
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
        // Show provided interfaces
        if let Some(provider) = diagram
            .interface_providers
            .iter()
            .find(|p| p.system == id)
        {
            println!("  Provides interfaces:");
            for iface_id in &provider.interfaces {
                if let Some(iface) = diagram.interfaces.iter().find(|i| i.id == *iface_id) {
                    println!("    - {} ({})", iface.id, iface.name);
                }
            }
        }
        return Ok(());
    }

    // Check actors
    if let Some(actor) = diagram.actors.iter().find(|a| a.id == id) {
        let type_str = match actor.actor_type {
            crate::model::c4::context::ActorType::External => "external",
            crate::model::c4::context::ActorType::Internal => "internal",
        };
        println!("{}: {}", "Actor".cyan(), actor.id);
        println!("  Name: {}", actor.name);
        println!("  Type: {}", type_str);
        if let Some(ref desc) = actor.description {
            println!("  Description: {}", desc);
        }
        // Show used interfaces
        if let Some(usage) = diagram.interface_usages.iter().find(|u| u.actor == id) {
            println!("  Uses interfaces:");
            for iface_id in &usage.interfaces {
                if let Some(iface) = diagram.interfaces.iter().find(|i| i.id == *iface_id) {
                    println!("    - {} ({})", iface.id, iface.name);
                }
            }
        }
        return Ok(());
    }

    // Check external systems
    if let Some(ext) = diagram.external_systems.iter().find(|e| e.id == id) {
        println!("{}: {}", "External System".cyan(), ext.id);
        println!("  Name: {}", ext.name);
        if let Some(ref desc) = ext.description {
            println!("  Description: {}", desc);
        }
        if let Some(ref tech) = ext.technology {
            println!("  Technology: {}", tech);
        }
        // Show provided interfaces
        if let Some(provider) = diagram
            .interface_providers
            .iter()
            .find(|p| p.system == id)
        {
            println!("  Provides interfaces:");
            for iface_id in &provider.interfaces {
                if let Some(iface) = diagram.interfaces.iter().find(|i| i.id == *iface_id) {
                    println!("    - {} ({})", iface.id, iface.name);
                }
            }
        }
        // Show used interfaces
        if let Some(usage) = diagram.interface_usages.iter().find(|u| u.actor == id) {
            println!("  Uses interfaces:");
            for iface_id in &usage.interfaces {
                if let Some(iface) = diagram.interfaces.iter().find(|i| i.id == *iface_id) {
                    println!("    - {} ({})", iface.id, iface.name);
                }
            }
        }
        return Ok(());
    }

    // Check interfaces
    if let Some(iface) = diagram.interfaces.iter().find(|i| i.id == id) {
        let protocol = match &iface.protocol {
            crate::model::c4::context::Protocol::Rest => "REST",
            crate::model::c4::context::Protocol::Grpc => "gRPC",
            crate::model::c4::context::Protocol::Graphql => "GraphQL",
            crate::model::c4::context::Protocol::WebSocket => "WebSocket",
            crate::model::c4::context::Protocol::Mqtt => "MQTT",
            crate::model::c4::context::Protocol::Amqp => "AMQP",
            crate::model::c4::context::Protocol::Custom(s) => s,
        };
        println!("{}: {}", "Interface".cyan(), iface.id);
        println!("  Name: {}", iface.name);
        println!("  Protocol: {}", protocol);
        if let Some(ref desc) = iface.description {
            println!("  Description: {}", desc);
        }
        // Show providers
        let providers: Vec<_> = diagram
            .interface_providers
            .iter()
            .filter(|p| p.interfaces.contains(&id.to_string()))
            .collect();
        if !providers.is_empty() {
            println!("  Provided by:");
            for provider in providers {
                if let Some(name) = diagram.get_element_name(&provider.system) {
                    println!("    - {} ({})", provider.system, name);
                }
            }
        }
        // Show users
        let users: Vec<_> = diagram
            .interface_usages
            .iter()
            .filter(|u| u.interfaces.contains(&id.to_string()))
            .collect();
        if !users.is_empty() {
            println!("  Used by:");
            for user in users {
                if let Some(name) = diagram.get_element_name(&user.actor) {
                    println!("    - {} ({})", user.actor, name);
                }
            }
        }
        return Ok(());
    }

    Err(crate::utils::error::AppError::ElementNotFound(id.to_string()))
}
