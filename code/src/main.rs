use clap::Parser;
use sys_design::cli::args::{Args, Commands};
use sys_design::cli::commands;
use anyhow::Result;

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::ContextModel { model_file, command } => {
            commands::context_model::execute(&model_file, command)?;
        }
        Commands::LogicModel { model_file, command } => {
            commands::logic_model::execute(&model_file, command)?;
        }
        Commands::ConceptModel { model_file, command } => {
            commands::concept_model::execute(&model_file, command)?;
        }
        Commands::Generate { model_file, output, command } => {
            commands::generate::execute(&model_file, output, &command)?;
        }
        Commands::Validate { model_file, format, type_ } => {
            commands::validate::execute(&model_file, format, type_)?;
        }
    }

    Ok(())
}
