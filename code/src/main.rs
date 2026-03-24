use clap::Parser;
use sys_design::cli::args::{Args, Commands};
use sys_design::cli::commands;
use anyhow::Result;

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::ContextModel { src, command } => {
            commands::context_model::execute(&src, command)?;
        }
        Commands::Generate { src, output, type_ } => {
            commands::generate::execute(&src, output, type_)?;
        }
        Commands::Validate { src, format, type_ } => {
            commands::validate::execute(&src, format, type_)?;
        }
    }

    Ok(())
}
