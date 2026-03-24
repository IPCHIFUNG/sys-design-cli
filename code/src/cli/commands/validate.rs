use crate::cli::args::{DiagramType, OutputFormat};
use crate::store::YamlStore;
use crate::validator::{validate, validate_logic_concept, result::ValidationResult, Severity};
use crate::utils::error::{AppError, Result};
use std::path::Path;

pub fn execute(src: &Path, format: OutputFormat, diagram_type: DiagramType) -> Result<()> {
    let result = load_and_validate(src, &diagram_type)?;

    match format {
        OutputFormat::Text => {
            print_result(&result);
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&result)?;
            println!("{}", json);
        }
    }

    if !result.is_valid {
        std::process::exit(1);
    }

    Ok(())
}

fn load_and_validate(src: &Path, diagram_type: &DiagramType) -> Result<ValidationResult> {
    // First try to load as workspace
    if let Ok(workspace) = YamlStore::load_workspace(src) {
        return validate_from_workspace(&workspace, diagram_type);
    }

    // Fallback: try loading as individual diagram type
    match diagram_type {
        DiagramType::Context => {
            let diagram = YamlStore::load_context(src)?;
            Ok(validate(&diagram))
        }
        DiagramType::LogicConcept => {
            let diagram = YamlStore::load_logic_concept(src)?;
            Ok(validate_logic_concept(&diagram))
        }
    }
}

fn validate_from_workspace(workspace: &crate::model::workspace::Workspace, diagram_type: &DiagramType) -> Result<ValidationResult> {
    match diagram_type {
        DiagramType::Context => {
            match &workspace.context_diagram {
                Some(diagram) => Ok(validate(diagram)),
                None => Err(AppError::ElementNotFound(
                    "context_diagram not found in workspace".to_string()
                )),
            }
        }
        DiagramType::LogicConcept => {
            match &workspace.logic_concept_diagram {
                Some(diagram) => Ok(validate_logic_concept(diagram)),
                None => Err(AppError::ElementNotFound(
                    "logic_concept_diagram not found in workspace".to_string()
                )),
            }
        }
    }
}

fn print_result(result: &ValidationResult) {
    use colored::Colorize;

    println!();
    println!("{}", "Validation Result:".cyan().bold());
    println!(
        "  Errors: {}, Warnings: {}, Info: {}",
        result.error_count().to_string().red(),
        result.warning_count().to_string().yellow(),
        result.info_count().to_string().blue()
    );
    println!();

    if result.errors.is_empty() {
        println!("{}", "No issues found.".green());
    } else {
        for error in &result.errors {
            let severity_str = match error.severity {
                Severity::Error => "ERROR".red(),
                Severity::Warning => "WARN".yellow(),
                Severity::Info => "INFO".blue(),
            };

            println!("  [{}] {}: {}", severity_str, error.code.white().bold(), error.message);

            if let Some(ref loc) = error.location {
                println!("         Location: {}", loc.as_str().dimmed());
            }
        }
    }

    println!();

    if result.is_valid {
        println!("{}", "Validation passed!".green());
    } else {
        println!("{}", "Validation failed. Please fix the errors above.".red());
    }
}
