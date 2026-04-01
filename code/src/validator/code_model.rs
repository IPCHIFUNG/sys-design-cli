use crate::model::code::CodeModel;
use crate::validator::result::{Severity, ValidationError, ValidationResult};
use regex::Regex;
use std::collections::HashSet;

/// Validate a CodeModel
pub fn validate(model: &CodeModel) -> ValidationResult {
    let mut result = ValidationResult::new();
    let id_pattern = Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap();

    let mut seen_ids = HashSet::new();

    for pkg in &model.packages {
        let location = format!("packages.{}", pkg.id);

        // CM001: Package must have non-empty id and name
        if pkg.id.is_empty() {
            result.add_error(ValidationError {
                code: "CM001".to_string(),
                rule: "PackageRequiredFields".to_string(),
                message: "Package ID cannot be empty".to_string(),
                severity: Severity::Error,
                location: Some(location.clone()),
            });
        }
        if pkg.name.is_empty() {
            result.add_error(ValidationError {
                code: "CM001".to_string(),
                rule: "PackageRequiredFields".to_string(),
                message: format!("Package '{}' name cannot be empty", pkg.id),
                severity: Severity::Error,
                location: Some(format!("{}.name", location)),
            });
        }

        // CM002: Package ID must be unique
        if !pkg.id.is_empty() {
            if !seen_ids.insert(pkg.id.clone()) {
                result.add_error(ValidationError {
                    code: "CM002".to_string(),
                    rule: "PackageIdUniqueness".to_string(),
                    message: format!("Duplicate package ID: {}", pkg.id),
                    severity: Severity::Error,
                    location: Some(location.clone()),
                });
            }
        }

        // CM003: Package ID should follow UPPER_SNAKE_CASE
        if !pkg.id.is_empty() && !id_pattern.is_match(&pkg.id) {
            result.add_error(ValidationError {
                code: "CM003".to_string(),
                rule: "PackageIdNaming".to_string(),
                message: format!(
                    "Package ID '{}' does not follow UPPER_SNAKE_CASE convention",
                    pkg.id
                ),
                severity: Severity::Warning,
                location: Some(format!("{}.id", location)),
            });
        }

        // CM004: Package should specify source path
        if pkg.path.is_none() {
            result.add_error(ValidationError {
                code: "CM004".to_string(),
                rule: "PackagePathRequired".to_string(),
                message: format!("Package '{}' should specify a source path", pkg.id),
                severity: Severity::Warning,
                location: Some(format!("{}.path", location)),
            });
        }
    }

    // CM005: Dependency from/to must reference existing packages
    for dep in &model.dependencies {
        if !seen_ids.contains(&dep.from) {
            result.add_error(ValidationError {
                code: "CM005".to_string(),
                rule: "DependencyReferencesValid".to_string(),
                message: format!(
                    "Dependency 'from' references non-existent package: {}",
                    dep.from
                ),
                severity: Severity::Error,
                location: Some(format!("dependencies.{}", dep.from)),
            });
        }
        if !seen_ids.contains(&dep.to) {
            result.add_error(ValidationError {
                code: "CM005".to_string(),
                rule: "DependencyReferencesValid".to_string(),
                message: format!(
                    "Dependency 'to' references non-existent package: {}",
                    dep.to
                ),
                severity: Severity::Error,
                location: Some(format!("dependencies.{}", dep.to)),
            });
        }
    }

    // CM006: Circular dependency detection
    if !model.dependencies.is_empty() && !model.packages.is_empty() {
        detect_cycles(&model, &mut result);
    }

    result
}

fn detect_cycles(model: &CodeModel, result: &mut ValidationResult) {
    let package_ids: HashSet<&str> = model.packages.iter().map(|p| p.id.as_str()).collect();

    // Build adjacency list
    let mut adj: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for dep in &model.dependencies {
        adj.entry(&dep.from).or_default().push(&dep.to);
    }

    // DFS-based cycle detection
    let mut visited: HashSet<&str> = HashSet::new();
    let mut in_stack: HashSet<&str> = HashSet::new();

    for pkg_id in &package_ids {
        if !visited.contains(pkg_id) {
            if has_cycle(pkg_id, &adj, &mut visited, &mut in_stack) {
                result.add_error(ValidationError {
                    code: "CM006".to_string(),
                    rule: "CircularDependency".to_string(),
                    message: "Circular dependency detected among packages".to_string(),
                    severity: Severity::Warning,
                    location: None,
                });
                return; // Only report once
            }
        }
    }
}

fn has_cycle<'a>(
    node: &'a str,
    adj: &std::collections::HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    in_stack: &mut HashSet<&'a str>,
) -> bool {
    visited.insert(node);
    in_stack.insert(node);

    if let Some(neighbors) = adj.get(node) {
        for &neighbor in neighbors {
            if !visited.contains(neighbor) {
                if has_cycle(neighbor, adj, visited, in_stack) {
                    return true;
                }
            } else if in_stack.contains(neighbor) {
                return true;
            }
        }
    }

    in_stack.remove(node);
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::code::*;

    fn create_test_model() -> CodeModel {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG_A".to_string(),
            name: "Package A".to_string(),
            description: None,
            language: Some(Language::Rust),
            framework: None,
            path: Some("src/a".to_string()),
            element_id: None,
        });
        model.packages.push(CodePackage {
            id: "PKG_B".to_string(),
            name: "Package B".to_string(),
            description: None,
            language: None,
            framework: None,
            path: Some("src/b".to_string()),
            element_id: None,
        });
        model.dependencies.push(PackageDependency {
            from: "PKG_A".to_string(),
            to: "PKG_B".to_string(),
        });
        model
    }

    #[test]
    fn test_valid_code_model() {
        let model = create_test_model();
        let result = validate(&model);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_cm001_empty_id() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "".to_string(),
            name: "No ID".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "CM001" && e.severity == Severity::Error));
    }

    #[test]
    fn test_cm001_empty_name() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG".to_string(),
            name: "".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "CM001" && e.severity == Severity::Error));
    }

    #[test]
    fn test_cm002_duplicate_id() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG".to_string(),
            name: "First".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        model.packages.push(CodePackage {
            id: "PKG".to_string(),
            name: "Second".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "CM002"));
    }

    #[test]
    fn test_cm003_naming() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "bad-name".to_string(),
            name: "Bad Name".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        let result = validate(&model);
        assert!(result.is_valid); // Warning only
        assert!(result.errors.iter().any(|e| e.code == "CM003" && e.severity == Severity::Warning));
    }

    #[test]
    fn test_cm004_no_path() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG".to_string(),
            name: "Pkg".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        let result = validate(&model);
        assert!(result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "CM004" && e.severity == Severity::Warning));
    }

    #[test]
    fn test_cm005_invalid_dep() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG_A".to_string(),
            name: "A".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        model.dependencies.push(PackageDependency {
            from: "PKG_A".to_string(),
            to: "PKG_NONEXISTENT".to_string(),
        });
        let result = validate(&model);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "CM005"));
    }

    #[test]
    fn test_cm006_circular_dep() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG_A".to_string(),
            name: "A".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        model.packages.push(CodePackage {
            id: "PKG_B".to_string(),
            name: "B".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        model.dependencies.push(PackageDependency {
            from: "PKG_A".to_string(),
            to: "PKG_B".to_string(),
        });
        model.dependencies.push(PackageDependency {
            from: "PKG_B".to_string(),
            to: "PKG_A".to_string(),
        });
        let result = validate(&model);
        assert!(result.is_valid); // Warning only
        assert!(result.errors.iter().any(|e| e.code == "CM006" && e.severity == Severity::Warning));
    }

    #[test]
    fn test_cm006_self_dependency() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG_A".to_string(),
            name: "A".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        model.dependencies.push(PackageDependency {
            from: "PKG_A".to_string(),
            to: "PKG_A".to_string(),
        });
        let result = validate(&model);
        assert!(result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "CM006" && e.severity == Severity::Warning));
    }

    #[test]
    fn test_cm006_three_node_cycle() {
        let mut model = CodeModel::new("Test");
        model.packages.push(CodePackage {
            id: "PKG_A".to_string(),
            name: "A".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        model.packages.push(CodePackage {
            id: "PKG_B".to_string(),
            name: "B".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        model.packages.push(CodePackage {
            id: "PKG_C".to_string(),
            name: "C".to_string(),
            description: None,
            language: None,
            framework: None,
            path: None,
            element_id: None,
        });
        model.dependencies.push(PackageDependency {
            from: "PKG_A".to_string(),
            to: "PKG_B".to_string(),
        });
        model.dependencies.push(PackageDependency {
            from: "PKG_B".to_string(),
            to: "PKG_C".to_string(),
        });
        model.dependencies.push(PackageDependency {
            from: "PKG_C".to_string(),
            to: "PKG_A".to_string(),
        });
        let result = validate(&model);
        assert!(result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "CM006" && e.severity == Severity::Warning));
    }
}
