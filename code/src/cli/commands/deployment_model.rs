use crate::cli::args::{
    DeploymentModelAddCommand, DeploymentModelCommand, DeploymentModelListElement,
    DeploymentModelRemoveCommand,
};
use crate::model::deployment::DeploymentModel;
use crate::store::{DeploymentOperations, YamlStore};
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::Path;

pub fn execute(model_file: &Path, cmd: DeploymentModelCommand) -> Result<()> {
    match cmd {
        DeploymentModelCommand::Add(add_cmd) => execute_add(model_file, add_cmd),
        DeploymentModelCommand::Remove(remove_cmd) => execute_remove(model_file, remove_cmd),
        DeploymentModelCommand::List { element } => execute_list(model_file, element),
        DeploymentModelCommand::Show { id } => execute_show(model_file, &id),
    }
}

fn execute_add(model_file: &Path, cmd: DeploymentModelAddCommand) -> Result<()> {
    match cmd {
        DeploymentModelAddCommand::Environment { id, name, desc } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            if workspace.deployment_model.is_none() {
                workspace.deployment_model = Some(DeploymentModel::new("Deployment Model"));
            }

            let model = workspace.deployment_model.as_mut().unwrap();
            DeploymentOperations::add_environment(model, &id, name.as_deref(), desc.as_deref())?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} environment: {}", "Added".green(), id);
            Ok(())
        }
        DeploymentModelAddCommand::Node {
            id,
            name,
            desc,
            node_type,
            environment,
            technology,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            if workspace.deployment_model.is_none() {
                workspace.deployment_model = Some(DeploymentModel::new("Deployment Model"));
            }

            let model = workspace.deployment_model.as_mut().unwrap();
            DeploymentOperations::add_node(
                model,
                &id,
                name.as_deref(),
                desc.as_deref(),
                node_type.map(|t| t.into()),
                environment.as_deref(),
                technology.as_deref(),
            )?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} node: {}", "Added".green(), id);
            Ok(())
        }
        DeploymentModelAddCommand::Service {
            id,
            name,
            desc,
            delivery_package,
            target_node,
            replicas,
            port,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            // Cross-diagram validation: delivery_package must exist in workspace.delivery_model
            if let Some(delivery_model) = &workspace.delivery_model {
                if delivery_model.find_package(&delivery_package).is_none() {
                    return Err(AppError::ElementNotFound(format!(
                        "delivery_package '{}' 在 delivery_model 中未找到",
                        delivery_package
                    )));
                }
            } else {
                return Err(AppError::ElementNotFound(
                    "delivery_model 在工作空间中未找到".to_string(),
                ));
            }

            if workspace.deployment_model.is_none() {
                workspace.deployment_model = Some(DeploymentModel::new("Deployment Model"));
            }

            let model = workspace.deployment_model.as_mut().unwrap();
            DeploymentOperations::add_service(
                model,
                &id,
                name.as_deref(),
                desc.as_deref(),
                &delivery_package,
                &target_node,
                replicas,
                port,
            )?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} service: {}", "Added".green(), id);
            Ok(())
        }
        DeploymentModelAddCommand::NetworkLink {
            id,
            from,
            to,
            protocol,
            port,
            desc,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            if workspace.deployment_model.is_none() {
                workspace.deployment_model = Some(DeploymentModel::new("Deployment Model"));
            }

            let model = workspace.deployment_model.as_mut().unwrap();
            DeploymentOperations::add_network_link(
                model,
                &id,
                &from,
                &to,
                protocol.as_deref(),
                port,
                desc.as_deref(),
            )?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} network-link: {}", "Added".green(), id);
            Ok(())
        }
    }
}

fn execute_remove(model_file: &Path, cmd: DeploymentModelRemoveCommand) -> Result<()> {
    match cmd {
        DeploymentModelRemoveCommand::Environment { id } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.deployment_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("deployment_model 在工作空间中未找到".to_string())
            })?;

            DeploymentOperations::remove_environment(model, &id)?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} environment: {}", "Removed".green(), id);
            Ok(())
        }
        DeploymentModelRemoveCommand::Node { id } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.deployment_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("deployment_model 在工作空间中未找到".to_string())
            })?;

            DeploymentOperations::remove_node(model, &id)?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} node: {}", "Removed".green(), id);
            Ok(())
        }
        DeploymentModelRemoveCommand::Service { id } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.deployment_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("deployment_model 在工作空间中未找到".to_string())
            })?;

            DeploymentOperations::remove_service(model, &id)?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} service: {}", "Removed".green(), id);
            Ok(())
        }
        DeploymentModelRemoveCommand::NetworkLink { id } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            let model = workspace.deployment_model.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("deployment_model 在工作空间中未找到".to_string())
            })?;

            DeploymentOperations::remove_network_link(model, &id)?;

            workspace.touch();
            YamlStore::save_workspace(model_file, &workspace)?;
            println!("{} network-link: {}", "Removed".green(), id);
            Ok(())
        }
    }
}

fn execute_list(model_file: &Path, element: DeploymentModelListElement) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(model_file)?;

    let model = workspace.deployment_model.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("deployment_model 在工作空间中未找到".to_string())
    })?;

    match element {
        DeploymentModelListElement::Environments => {
            if model.environments.is_empty() {
                println!("No environments found.");
            } else {
                println!("{}", "Environments:".cyan().bold());
                for env in &model.environments {
                    println!("  {} ({})", env.id.white().bold(), env.name);
                    if let Some(ref desc) = env.description {
                        println!("    description: {}", desc);
                    }
                }
            }
        }
        DeploymentModelListElement::Nodes => {
            if model.nodes.is_empty() {
                println!("No nodes found.");
            } else {
                println!("{}", "Nodes:".cyan().bold());
                for node in &model.nodes {
                    println!("  {} ({})", node.id.white().bold(), node.name);
                    println!("    node_type: {:?}", node.node_type);
                    if let Some(ref env) = node.environment {
                        println!("    environment: {}", env);
                    }
                    if let Some(ref tech) = node.technology {
                        println!("    technology: {}", tech);
                    }
                    if let Some(ref desc) = node.description {
                        println!("    description: {}", desc);
                    }
                }
            }
        }
        DeploymentModelListElement::Services => {
            if model.services.is_empty() {
                println!("No services found.");
            } else {
                println!("{}", "Services:".cyan().bold());
                for svc in &model.services {
                    println!("  {} ({})", svc.id.white().bold(), svc.name);
                    println!("    delivery_package: {}", svc.delivery_package);
                    println!("    target_node: {}", svc.target_node);
                    if let Some(replicas) = svc.replicas {
                        println!("    replicas: {}", replicas);
                    }
                    if let Some(port) = svc.port {
                        println!("    port: {}", port);
                    }
                    if let Some(ref desc) = svc.description {
                        println!("    description: {}", desc);
                    }
                }
            }
        }
        DeploymentModelListElement::NetworkLinks => {
            if model.network_links.is_empty() {
                println!("No network-links found.");
            } else {
                println!("{}", "Network Links:".cyan().bold());
                for link in &model.network_links {
                    println!("  {} ({})", link.id.white().bold(), link.id);
                    println!("    from: {}", link.from_service);
                    println!("    to: {}", link.to_service);
                    if let Some(ref proto) = link.protocol {
                        println!("    protocol: {}", proto);
                    }
                    if let Some(port) = link.port {
                        println!("    port: {}", port);
                    }
                    if let Some(ref desc) = link.description {
                        println!("    description: {}", desc);
                    }
                }
            }
        }
    }

    Ok(())
}

fn execute_show(model_file: &Path, id: &str) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(model_file)?;

    let model = workspace.deployment_model.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("deployment_model 在工作空间中未找到".to_string())
    })?;

    if let Some(env) = model.find_environment(id) {
        println!("{}", format!("Environment: {}", env.id).cyan().bold());
        println!("  name: {}", env.name);
        if let Some(ref desc) = env.description {
            println!("  description: {}", desc);
        }
    } else if let Some(node) = model.find_node(id) {
        println!("{}", format!("Node: {}", node.id).cyan().bold());
        println!("  name: {}", node.name);
        println!("  node_type: {:?}", node.node_type);
        if let Some(ref env) = node.environment {
            println!("  environment: {}", env);
        }
        if let Some(ref tech) = node.technology {
            println!("  technology: {}", tech);
        }
        if let Some(ref desc) = node.description {
            println!("  description: {}", desc);
        }
    } else if let Some(svc) = model.find_service(id) {
        println!("{}", format!("Service: {}", svc.id).cyan().bold());
        println!("  name: {}", svc.name);
        println!("  delivery_package: {}", svc.delivery_package);
        println!("  target_node: {}", svc.target_node);
        if let Some(replicas) = svc.replicas {
            println!("  replicas: {}", replicas);
        }
        if let Some(port) = svc.port {
            println!("  port: {}", port);
        }
        if let Some(ref desc) = svc.description {
            println!("  description: {}", desc);
        }
    } else if let Some(link) = model.find_network_link(id) {
        println!("{}", format!("Network Link: {}", link.id).cyan().bold());
        println!("  from: {}", link.from_service);
        println!("  to: {}", link.to_service);
        if let Some(ref proto) = link.protocol {
            println!("  protocol: {}", proto);
        }
        if let Some(port) = link.port {
            println!("  port: {}", port);
        }
        if let Some(ref desc) = link.description {
            println!("  description: {}", desc);
        }
    } else {
        return Err(AppError::ElementNotFound(format!(
            "deployment_model 中未找到元素: {}",
            id
        )));
    }

    Ok(())
}
