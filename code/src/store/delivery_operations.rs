use crate::model::delivery::{DeliveryModel, DeliveryPackage, DeliveryType};
use crate::utils::error::{AppError, Result};

/// Operations for modifying DeliveryModel
pub struct DeliveryOperations;

impl DeliveryOperations {
    /// Add a new package (rejects duplicate ID)
    pub fn add_package(
        model: &mut DeliveryModel,
        id: &str,
        name: Option<&str>,
        desc: Option<&str>,
        version: Option<&str>,
        delivery_type: Option<DeliveryType>,
        artifacts: Vec<String>,
        registry: Option<&str>,
    ) -> Result<()> {
        if model.packages.iter().any(|p| p.id == id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "package: {}",
                id
            )));
        }

        model.packages.push(DeliveryPackage {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: desc.map(|s| s.to_string()),
            version: version.map(|s| s.to_string()),
            delivery_type: delivery_type.unwrap_or_default(),
            artifacts,
            registry: registry.map(|s| s.to_string()),
        });
        model.touch();
        Ok(())
    }

    /// Remove a package by ID (no cascade - caller handles blocking if services reference it)
    pub fn remove_package(model: &mut DeliveryModel, id: &str) -> Result<()> {
        let idx = model
            .packages
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("package: {}", id)))?;

        model.packages.remove(idx);
        model.touch();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> DeliveryModel {
        DeliveryModel::new("Test Delivery Model")
    }

    #[test]
    fn test_add_package() {
        let mut model = create_test_model();
        DeliveryOperations::add_package(
            &mut model,
            "CORE_IMG",
            Some("Core Image"),
            Some("Main container image"),
            Some("1.0.0"),
            Some(DeliveryType::ContainerImage),
            vec!["CORE_BIN".to_string()],
            Some("registry.example.com/core"),
        )
        .unwrap();
        assert_eq!(model.packages.len(), 1);
        assert_eq!(model.packages[0].id, "CORE_IMG");
        assert_eq!(model.packages[0].name, "Core Image");
        assert_eq!(model.packages[0].artifacts, vec!["CORE_BIN"]);
    }

    #[test]
    fn test_add_duplicate_package() {
        let mut model = create_test_model();
        DeliveryOperations::add_package(&mut model, "PKG_A", None, None, None, None, vec![], None).unwrap();
        let result = DeliveryOperations::add_package(&mut model, "PKG_A", None, None, None, None, vec![], None);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_package() {
        let mut model = create_test_model();
        DeliveryOperations::add_package(&mut model, "PKG_A", None, None, None, None, vec![], None).unwrap();
        DeliveryOperations::add_package(&mut model, "PKG_B", None, None, None, None, vec![], None).unwrap();

        DeliveryOperations::remove_package(&mut model, "PKG_A").unwrap();
        assert_eq!(model.packages.len(), 1);
        assert_eq!(model.packages[0].id, "PKG_B");
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut model = create_test_model();
        let result = DeliveryOperations::remove_package(&mut model, "PKG_X");
        assert!(result.is_err());
    }
}
