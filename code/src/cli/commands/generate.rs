use crate::generator::plantuml::context::generate_plantuml;
use crate::store::YamlStore;
use crate::utils::error::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn execute(src: &std::path::Path, output: Option<PathBuf>) -> Result<()> {
    let diagram = YamlStore::load(src)?;
    let plantuml = generate_plantuml(&diagram);

    match output {
        Some(path) => {
            std::fs::write(&path, &plantuml)?;
            println!(
                "{} PlantUML diagram to: {}",
                "Generated".green(),
                path.display()
            );
        }
        None => {
            println!("{}", plantuml);
        }
    }

    Ok(())
}
