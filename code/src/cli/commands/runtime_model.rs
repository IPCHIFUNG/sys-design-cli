use crate::cli::args::{RuntimeAddCommand, RuntimeListElement, RuntimeModelCommand, RuntimeRemoveCommand};
use crate::model::runtime::RuntimeView;
use crate::store::{RuntimeOperations, YamlStore};
use crate::utils::error::{AppError, Result};
use colored::Colorize;
use std::path::Path;

pub fn execute(model_file: &Path, cmd: RuntimeModelCommand) -> Result<()> {
    match cmd {
        RuntimeModelCommand::Add(add_cmd) => execute_add(model_file, add_cmd),
        RuntimeModelCommand::Remove(remove_cmd) => execute_remove(model_file, remove_cmd),
        RuntimeModelCommand::List { element, scenario } => {
            execute_list(model_file, element, scenario)
        }
        RuntimeModelCommand::Show { scenario_id } => execute_show(model_file, &scenario_id),
    }
}

fn execute_add(model_file: &Path, cmd: RuntimeAddCommand) -> Result<()> {
    match cmd {
        RuntimeAddCommand::Scenario { id, name, desc } => {
            let mut workspace = if YamlStore::exists(model_file) {
                YamlStore::load_workspace_any(model_file)?
            } else {
                crate::model::workspace::Workspace::new(&id)
            };

            if workspace.runtime_view.is_none() {
                workspace.runtime_view = Some(RuntimeView::new(&id));
            }

            let view = workspace.runtime_view.as_mut().unwrap();
            RuntimeOperations::add_scenario(view, &id, name.as_deref(), desc.as_deref())?;

            YamlStore::save_runtime_to_workspace(model_file, &mut workspace)?;
            println!("{} scenario: {}", "Added".green(), id);
            Ok(())
        }
        RuntimeAddCommand::Participant {
            scenario_id,
            element_id,
            participant_type,
            alias,
            color,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;

            // Cross-diagram validation BEFORE mutable borrow
            validate_element_exists_in_static_models(&workspace, &element_id)?;

            let view = workspace.runtime_view.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("runtime_view not found in workspace".to_string())
            })?;

            RuntimeOperations::add_participant(
                view,
                &scenario_id,
                &element_id,
                participant_type.into(),
                alias.as_deref(),
                color.as_deref(),
            )?;

            YamlStore::save_runtime_to_workspace(model_file, &mut workspace)?;
            println!(
                "{} participant '{}' to scenario '{}'",
                "Added".green(),
                element_id,
                scenario_id
            );
            Ok(())
        }
        RuntimeAddCommand::Step {
            scenario_id,
            from,
            to,
            message,
            step_type,
            protocol,
            color,
            activate,
            group,
            branch,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;
            let view = workspace.runtime_view.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("runtime_view not found in workspace".to_string())
            })?;

            RuntimeOperations::add_step(
                view,
                &scenario_id,
                &from,
                &to,
                &message,
                step_type.into(),
                protocol.as_deref(),
                color.as_deref(),
                activate,
                group.as_deref(),
                branch.as_deref(),
            )?;

            YamlStore::save_runtime_to_workspace(model_file, &mut workspace)?;
            println!(
                "{} step '{}' -> '{}' : {} to scenario '{}'",
                "Added".green(),
                from,
                to,
                message,
                scenario_id
            );
            Ok(())
        }
        RuntimeAddCommand::Group {
            scenario_id,
            group_type,
            label,
            branches,
            group,
            branch,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;
            let view = workspace.runtime_view.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("runtime_view not found in workspace".to_string())
            })?;

            RuntimeOperations::add_group(
                view,
                &scenario_id,
                group_type.into(),
                &label,
                branches,
                group.as_deref(),
                branch.as_deref(),
            )?;

            YamlStore::save_runtime_to_workspace(model_file, &mut workspace)?;
            println!(
                "{} group '{}' to scenario '{}'",
                "Added".green(),
                label,
                scenario_id
            );
            Ok(())
        }
        RuntimeAddCommand::Note {
            scenario_id,
            position,
            target,
            text,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;
            let view = workspace.runtime_view.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("runtime_view not found in workspace".to_string())
            })?;

            RuntimeOperations::add_note(
                view,
                &scenario_id,
                position.into(),
                &target,
                &text,
            )?;

            YamlStore::save_runtime_to_workspace(model_file, &mut workspace)?;
            println!("{} note to scenario '{}'", "Added".green(), scenario_id);
            Ok(())
        }
        RuntimeAddCommand::Divider {
            scenario_id,
            label,
            after_order,
        } => {
            let mut workspace = YamlStore::load_workspace_any(model_file)?;
            let view = workspace.runtime_view.as_mut().ok_or_else(|| {
                AppError::ElementNotFound("runtime_view not found in workspace".to_string())
            })?;

            RuntimeOperations::add_divider(view, &scenario_id, &label, after_order)?;

            YamlStore::save_runtime_to_workspace(model_file, &mut workspace)?;
            println!("{} divider '{}' to scenario '{}'", "Added".green(), label, scenario_id);
            Ok(())
        }
    }
}

fn execute_remove(model_file: &Path, cmd: RuntimeRemoveCommand) -> Result<()> {
    let mut workspace = YamlStore::load_workspace_any(model_file)?;

    let view = workspace.runtime_view.as_mut().ok_or_else(|| {
        AppError::ElementNotFound("runtime_view not found in workspace".to_string())
    })?;

    match cmd {
        RuntimeRemoveCommand::Scenario { id } => {
            RuntimeOperations::remove_scenario(view, &id)?;
            println!("{} scenario '{}'", "Removed".green(), id);
        }
        RuntimeRemoveCommand::Participant { scenario_id, element_id } => {
            RuntimeOperations::remove_participant(view, &scenario_id, &element_id)?;
            println!(
                "{} participant '{}' from scenario '{}' (cascaded to referencing steps)",
                "Removed".green(),
                element_id,
                scenario_id
            );
        }
        RuntimeRemoveCommand::Step { scenario_id, order, group, branch } => {
            RuntimeOperations::remove_step(view, &scenario_id, order, group.as_deref(), branch.as_deref())?;
            println!("{} step {} from scenario '{}'", "Removed".green(), order, scenario_id);
        }
        RuntimeRemoveCommand::Group { scenario_id, label, group, branch } => {
            RuntimeOperations::remove_group(view, &scenario_id, &label, group.as_deref(), branch.as_deref())?;
            println!("{} group '{}' from scenario '{}'", "Removed".green(), label, scenario_id);
        }
        RuntimeRemoveCommand::Note { scenario_id, index } => {
            RuntimeOperations::remove_note(view, &scenario_id, index)?;
            println!("{} note {} from scenario '{}'", "Removed".green(), index, scenario_id);
        }
        RuntimeRemoveCommand::Divider { scenario_id, index } => {
            RuntimeOperations::remove_divider(view, &scenario_id, index)?;
            println!("{} divider {} from scenario '{}'", "Removed".green(), index, scenario_id);
        }
    }

    YamlStore::save_runtime_to_workspace(model_file, &mut workspace)?;
    Ok(())
}

fn execute_list(
    model_file: &Path,
    element: RuntimeListElement,
    scenario: Option<String>,
) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(model_file)?;
    let view = workspace.runtime_view.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("runtime_view not found in workspace".to_string())
    })?;

    match element {
        RuntimeListElement::Scenarios => {
            println!("{}", "Scenarios:".cyan().bold());
            for s in &view.scenarios {
                println!(
                    "  {} {} - {} ({} participants, {} steps)",
                    "*".green(),
                    s.id.white().bold(),
                    s.name,
                    s.participants.len(),
                    s.collect_all_steps().len()
                );
            }
        }
        RuntimeListElement::Participants => {
            let scenario_id = scenario.as_deref().ok_or_else(|| {
                AppError::InvalidOperation(
                    "--scenario is required for listing participants".to_string()
                )
            })?;
            let scenario = view.find_scenario(scenario_id).ok_or_else(|| {
                AppError::ElementNotFound(format!("scenario: {}", scenario_id))
            })?;
            println!(
                "{} {}:",
                "Participants in scenario".cyan().bold(),
                scenario_id.white().bold()
            );
            for p in &scenario.participants {
                println!(
                    "  {} {} (type: {:?}{})",
                    "*".green(),
                    p.element_id.white().bold(),
                    p.participant_type,
                    p.alias
                        .as_ref()
                        .map(|a| format!(", alias: {}", a))
                        .unwrap_or_default()
                );
            }
        }
        RuntimeListElement::Steps => {
            let scenario_id = scenario.as_deref().ok_or_else(|| {
                AppError::InvalidOperation(
                    "--scenario is required for listing steps".to_string()
                )
            })?;
            let scenario = view.find_scenario(scenario_id).ok_or_else(|| {
                AppError::ElementNotFound(format!("scenario: {}", scenario_id))
            })?;
            println!(
                "{} {}:",
                "Steps in scenario".cyan().bold(),
                scenario_id.white().bold()
            );
            for step in scenario.collect_all_steps() {
                let arrow = match step.step_type {
                    crate::model::runtime::StepType::Sync => "->",
                    crate::model::runtime::StepType::Async => "->>",
                    crate::model::runtime::StepType::Return => "-->",
                    crate::model::runtime::StepType::Lost => "->o",
                };
                println!(
                    "  {} {} {} {} : {}",
                    format!("[{}]", step.order).yellow(),
                    step.from,
                    arrow,
                    step.to,
                    step.message
                );
            }
        }
        RuntimeListElement::Groups => {
            let scenario_id = scenario.as_deref().ok_or_else(|| {
                AppError::InvalidOperation(
                    "--scenario is required for listing groups".to_string()
                )
            })?;
            let scenario = view.find_scenario(scenario_id).ok_or_else(|| {
                AppError::ElementNotFound(format!("scenario: {}", scenario_id))
            })?;
            println!(
                "{} {}:",
                "Groups in scenario".cyan().bold(),
                scenario_id.white().bold()
            );
            for block in &scenario.blocks {
                if let crate::model::runtime::Block::Group(g) = block {
                    let branches_info = if g.branches.is_empty() {
                        String::new()
                    } else {
                        format!(
                            " (branches: {})",
                            g.branches
                                .iter()
                                .map(|b| b.label.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    };
                    println!(
                        "  {} {:?}: {}{}",
                        "*".green(),
                        g.group_type,
                        g.label.white().bold(),
                        branches_info
                    );
                }
            }
        }
    }

    Ok(())
}

fn execute_show(model_file: &Path, scenario_id: &str) -> Result<()> {
    let workspace = YamlStore::load_workspace_any(model_file)?;
    let view = workspace.runtime_view.as_ref().ok_or_else(|| {
        AppError::ElementNotFound("runtime_view not found in workspace".to_string())
    })?;
    let scenario = view.find_scenario(scenario_id).ok_or_else(|| {
        AppError::ElementNotFound(format!("scenario: {}", scenario_id))
    })?;

    println!("{}", format!("Scenario: {}", scenario.id).cyan().bold());
    println!("  Name: {}", scenario.name);
    if let Some(ref desc) = scenario.description {
        println!("  Description: {}", desc);
    }
    println!();

    println!("  {} ({}):", "Participants".bold(), scenario.participants.len());
    for p in &scenario.participants {
        println!(
            "    {} ({:?}{})",
            p.element_id,
            p.participant_type,
            p.alias.as_ref().map(|a| format!(", alias: {}", a)).unwrap_or_default()
        );
    }

    println!();
    println!("  {} ({} blocks):", "Blocks".bold(), scenario.blocks.len());
    print_blocks(&scenario.blocks, 2);

    if !scenario.notes.is_empty() {
        println!();
        println!("  {} ({}):", "Notes".bold(), scenario.notes.len());
        for note in &scenario.notes {
            println!("    {:?} {}: {}", note.position, note.target, note.text);
        }
    }

    if !scenario.dividers.is_empty() {
        println!();
        println!("  {} ({}):", "Dividers".bold(), scenario.dividers.len());
        for divider in &scenario.dividers {
            println!("    == {} == (after step {})", divider.label, divider.after_order);
        }
    }

    Ok(())
}

fn print_blocks(blocks: &[crate::model::runtime::Block], indent: usize) {
    let prefix = " ".repeat(indent * 2);
    for block in blocks {
        match block {
            crate::model::runtime::Block::Step(s) => {
                let arrow = match s.step_type {
                    crate::model::runtime::StepType::Sync => "->",
                    crate::model::runtime::StepType::Async => "->>",
                    crate::model::runtime::StepType::Return => "-->",
                    crate::model::runtime::StepType::Lost => "->o",
                };
                println!(
                    "{}[{}] {} {} {} : {}",
                    prefix, s.order, s.from, arrow, s.to, s.message
                );
            }
            crate::model::runtime::Block::Group(g) => {
                let branches_info = if g.branches.is_empty() {
                    String::new()
                } else {
                    format!(
                        " (branches: {})",
                        g.branches.iter().map(|b| b.label.as_str()).collect::<Vec<_>>().join(", ")
                    )
                };
                println!("{}{:?}: {}{}", prefix, g.group_type, g.label, branches_info);
                if g.branches.is_empty() {
                    print_blocks(&g.blocks, indent + 1);
                } else {
                    for branch in &g.branches {
                        println!("{}  {}:", " ".repeat((indent + 1) * 2), branch.label);
                        print_blocks(&branch.blocks, indent + 2);
                    }
                }
            }
        }
    }
}

/// Validate that an element_id exists in the static models
fn validate_element_exists_in_static_models(
    workspace: &crate::model::workspace::Workspace,
    element_id: &str,
) -> Result<()> {
    if let Some(ctx) = &workspace.context_diagram {
        if ctx.all_element_ids().iter().any(|id| *id == element_id) {
            return Ok(());
        }
    }

    if let Some(lv) = &workspace.logic_view {
        if lv.all_element_ids().iter().any(|id| *id == element_id) {
            return Ok(());
        }
    }

    Err(AppError::ElementNotFound(format!(
        "元素 '{}' 在 context_diagram 和 logic_view 中均未找到",
        element_id
    )))
}
