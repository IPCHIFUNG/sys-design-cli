use crate::model::code::{CodeModel, CodePackage, Language, PackageDependency};
use crate::utils::error::{AppError, Result};

/// Operations for modifying CodeModel
pub struct CodeOperations;

impl CodeOperations {
    /// Add a new package (rejects duplicate ID)
    pub fn add_package(
        model: &mut CodeModel,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        language: Option<Language>,
        framework: Option<&str>,
        path: Option<&str>,
        element_id: Option<&str>,
    ) -> Result<()> {
        if model.packages.iter().any(|p| p.id == id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "package: {}",
                id
            )));
        }

        model.packages.push(CodePackage {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: description.map(|s| s.to_string()),
            language,
            framework: framework.map(|s| s.to_string()),
            path: path.map(|s| s.to_string()),
            element_id: element_id.map(|s| s.to_string()),
        });
        model.touch();
        Ok(())
    }

    /// Remove a package by ID (cascade: does NOT clean upstream references, caller handles that)
    pub fn remove_package(model: &mut CodeModel, id: &str) -> Result<()> {
        let idx = model
            .packages
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("package: {}", id)))?;

        model.packages.remove(idx);

        // Remove dependencies referencing this package
        model
            .dependencies
            .retain(|d| d.from != id && d.to != id);

        model.touch();
        Ok(())
    }

    /// Add a dependency between packages
    pub fn add_dependency(model: &mut CodeModel, from: &str, to: &str) -> Result<()> {
        if !model.packages.iter().any(|p| p.id == from) {
            return Err(AppError::ElementNotFound(format!("package: {}", from)));
        }
        if !model.packages.iter().any(|p| p.id == to) {
            return Err(AppError::ElementNotFound(format!("package: {}", to)));
        }

        let dep = PackageDependency {
            from: from.to_string(),
            to: to.to_string(),
        };
        if model.dependencies.contains(&dep) {
            return Err(AppError::ElementAlreadyExists(format!(
                "dependency: {} -> {}",
                from, to
            )));
        }

        model.dependencies.push(dep);
        model.touch();
        Ok(())
    }

    /// Remove a dependency
    pub fn remove_dependency(model: &mut CodeModel, from: &str, to: &str) -> Result<()> {
        let idx = model
            .dependencies
            .iter()
            .position(|d| d.from == from && d.to == to)
            .ok_or_else(|| {
                AppError::ElementNotFound(format!("dependency: {} -> {}", from, to))
            })?;

        model.dependencies.remove(idx);
        model.touch();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> CodeModel {
        CodeModel::new("Test Code Model")
    }

    #[test]
    fn test_add_package() {
        let mut model = create_test_model();
        CodeOperations::add_package(
            &mut model,
            "CORE_LIB",
            Some("Core Library"),
            Some("Core functionality"),
            Some(Language::Rust),
            None,
            Some("src/core"),
            None,
        )
        .unwrap();
        assert_eq!(model.packages.len(), 1);
        assert_eq!(model.packages[0].id, "CORE_LIB");
        assert_eq!(model.packages[0].name, "Core Library");
    }

    #[test]
    fn test_add_duplicate_package() {
        let mut model = create_test_model();
        CodeOperations::add_package(&mut model, "PKG_A", None, None, None, None, None, None).unwrap();
        let result = CodeOperations::add_package(&mut model, "PKG_A", None, None, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_package() {
        let mut model = create_test_model();
        CodeOperations::add_package(&mut model, "PKG_A", None, None, None, None, None, None).unwrap();
        CodeOperations::add_package(&mut model, "PKG_B", None, None, None, None, None, None).unwrap();
        CodeOperations::add_dependency(&mut model, "PKG_A", "PKG_B").unwrap();

        CodeOperations::remove_package(&mut model, "PKG_A").unwrap();
        assert_eq!(model.packages.len(), 1);
        // Dependencies referencing PKG_A should be removed
        assert!(model.dependencies.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_package() {
        let mut model = create_test_model();
        let result = CodeOperations::remove_package(&mut model, "NOPE");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_dependency() {
        let mut model = create_test_model();
        CodeOperations::add_package(&mut model, "PKG_A", None, None, None, None, None, None).unwrap();
        CodeOperations::add_package(&mut model, "PKG_B", None, None, None, None, None, None).unwrap();
        CodeOperations::add_dependency(&mut model, "PKG_A", "PKG_B").unwrap();
        assert_eq!(model.dependencies.len(), 1);
    }

    #[test]
    fn test_add_dependency_invalid_from() {
        let mut model = create_test_model();
        CodeOperations::add_package(&mut model, "PKG_B", None, None, None, None, None, None).unwrap();
        let result = CodeOperations::add_dependency(&mut model, "PKG_A", "PKG_B");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_dependency_invalid_to() {
        let mut model = create_test_model();
        CodeOperations::add_package(&mut model, "PKG_A", None, None, None, None, None, None).unwrap();
        let result = CodeOperations::add_dependency(&mut model, "PKG_A", "PKG_B");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_duplicate_dependency() {
        let mut model = create_test_model();
        CodeOperations::add_package(&mut model, "PKG_A", None, None, None, None, None, None).unwrap();
        CodeOperations::add_package(&mut model, "PKG_B", None, None, None, None, None, None).unwrap();
        CodeOperations::add_dependency(&mut model, "PKG_A", "PKG_B").unwrap();
        let result = CodeOperations::add_dependency(&mut model, "PKG_A", "PKG_B");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_dependency() {
        let mut model = create_test_model();
        CodeOperations::add_package(&mut model, "PKG_A", None, None, None, None, None, None).unwrap();
        CodeOperations::add_package(&mut model, "PKG_B", None, None, None, None, None, None).unwrap();
        CodeOperations::add_dependency(&mut model, "PKG_A", "PKG_B").unwrap();
        CodeOperations::remove_dependency(&mut model, "PKG_A", "PKG_B").unwrap();
        assert!(model.dependencies.is_empty());
    }

    #[test]
    fn test_list_empty_packages() {
        let model = create_test_model();
        assert!(model.packages.is_empty());
        assert!(model.all_package_ids().is_empty());
    }
}
