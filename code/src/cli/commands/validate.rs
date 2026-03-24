use crate::cli::args::OutputFormat;
use crate::store::YamlStore;
use crate::validator::{validate, result::ValidationResult, Severity};
use crate::utils::error::Result;
use std::path::Path;

pub fn execute(src: &Path, format: OutputFormat) -> Result<()> {
    let diagram = YamlStore::load(src)?;
    let result = validate(&diagram);

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
