use crate::model::delivery::DeliveryModel;
use crate::validator::result::{Severity, ValidationError, ValidationResult};
use regex::Regex;
use std::collections::HashSet;

/// Validate a DeliveryModel
pub fn validate(model: &DeliveryModel) -> ValidationResult {
    let mut result = ValidationResult::new();
    let id_pattern = Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap();

    let mut seen_ids = HashSet::new();

    for package in &model.packages {
        let location = format!("packages.{}", package.id);

        // DM001: Package must have non-empty id and name
        if package.id.is_empty() {
            result.add_error(ValidationError {
                code: "DM001".to_string(),
                rule: "PackageRequiredFields".to_string(),
                message: "Package ID cannot be empty".to_string(),
                severity: Severity::Error,
                location: Some(location.clone()),
            });
        }
        if package.name.is_empty() {
            result.add_error(ValidationError {
                code: "DM001".to_string(),
                rule: "PackageRequiredFields".to_string(),
                message: format!("Package '{}' name cannot be empty", package.id),
                severity: Severity::Error,
                location: Some(format!("{}.name", location)),
            });
        }

        // DM002: Package ID must be unique
        if !package.id.is_empty() {
            if !seen_ids.insert(package.id.clone()) {
                result.add_error(ValidationError {
                    code: "DM002".to_string(),
                    rule: "PackageIdUniqueness".to_string(),
                    message: format!("Duplicate package ID: {}", package.id),
                    severity: Severity::Error,
                    location: Some(location.clone()),
                });
            }
        }

        // DM003: Package ID should follow UPPER_SNAKE_CASE
        if !package.id.is_empty() && !id_pattern.is_match(&package.id) {
            result.add_error(ValidationError {
                code: "DM003".to_string(),
                rule: "PackageIdNaming".to_string(),
                message: format!(
                    "Package ID '{}' does not follow UPPER_SNAKE_CASE convention",
                    package.id
                ),
                severity: Severity::Warning,
                location: Some(format!("{}.id", location)),
            });
        }

        // DM004: Version is recommended
        if package.version.is_none() {
            result.add_error(ValidationError {
                code: "DM004".to_string(),
                rule: "VersionRequired".to_string(),
                message: format!("Package '{}' should have a version specified", package.id),
                severity: Severity::Warning,
                location: Some(format!("{}.version", location)),
            });
        }
    }

    // DM005: Minimum packages warning
    if model.packages.is_empty() {
        result.add_error(ValidationError {
            code: "DM005".to_string(),
            rule: "MinPackages".to_string(),
            message: "Delivery model should have at least 1 package".to_string(),
            severity: Severity::Warning,
            location: None,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::delivery::*;

    fn create_test_model() -> DeliveryModel {
        let mut model = DeliveryModel::new("Test");
        model.packages.push(DeliveryPackage {
            id: "PKG_A".to_string(),
            name: "Package A".to_string(),
            description: None,
            version: Some("1.0.0".to_string()),
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        model
    }

    #[test]
    fn test_valid_delivery_model() {
        let model = create_test_model();
        let result = validate(&model);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_dm001_empty_id() {
        let mut model = DeliveryModel::new("Test");
        model.packages.push(DeliveryPackage {
            id: "".to_string(),
            name: "No ID".to_string(),
            description: None,
            version: None,
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DM001" && e.severity == Severity::Error));
    }

    #[test]
    fn test_dm001_empty_name() {
        let mut model = DeliveryModel::new("Test");
        model.packages.push(DeliveryPackage {
            id: "PKG".to_string(),
            name: "".to_string(),
            description: None,
            version: Some("1.0".to_string()),
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DM001" && e.severity == Severity::Error));
    }

    #[test]
    fn test_dm002_duplicate_id() {
        let mut model = DeliveryModel::new("Test");
        model.packages.push(DeliveryPackage {
            id: "PKG".to_string(),
            name: "First".to_string(),
            description: None,
            version: Some("1.0".to_string()),
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        model.packages.push(DeliveryPackage {
            id: "PKG".to_string(),
            name: "Second".to_string(),
            description: None,
            version: Some("1.0".to_string()),
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "DM002"));
    }

    #[test]
    fn test_dm003_naming() {
        let mut model = DeliveryModel::new("Test");
        model.packages.push(DeliveryPackage {
            id: "bad-name".to_string(),
            name: "Bad Name".to_string(),
            description: None,
            version: Some("1.0".to_string()),
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        let result = validate(&model);
        assert!(result.is_valid); // Warning only
        assert!(result.errors.iter().any(|e| e.code == "DM003" && e.severity == Severity::Warning));
    }

    #[test]
    fn test_dm004_no_version() {
        let mut model = DeliveryModel::new("Test");
        model.packages.push(DeliveryPackage {
            id: "PKG_A".to_string(),
            name: "Package A".to_string(),
            description: None,
            version: None,
            delivery_type: DeliveryType::ContainerImage,
            artifacts: vec![],
            registry: None,
        });
        let result = validate(&model);
        assert!(result.is_valid); // Warning only
        assert!(result.errors.iter().any(|e| e.code == "DM004" && e.severity == Severity::Warning));
    }

    #[test]
    fn test_dm005_min_packages() {
        let model = DeliveryModel::new("Test");
        let result = validate(&model);
        assert!(result.is_valid); // Warning only
        assert!(result.errors.iter().any(|e| e.code == "DM005" && e.severity == Severity::Warning));
    }
}
