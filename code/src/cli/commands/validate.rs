use crate::cli::args::{DiagramType, OutputFormat};
use crate::store::YamlStore;
use crate::validator::{validate, validate_logic_concept, validate_concept_model};
use crate::validator::result::{ValidationResult, Severity};
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
    // First try to load as workspace (only if it has actual diagrams)
    if let Ok(workspace) = YamlStore::load_workspace(src) {
        // Only use workspace if it actually contains diagrams or concept model
        if workspace.context_diagram.is_some()
            || workspace.logic_architecture_concept_model.is_some()
            || workspace.logic_view.is_some()
        {
            return validate_from_workspace(&workspace, diagram_type);
        }
    }

    // Fallback: try loading as individual diagram type
    let mut result = match diagram_type {
        DiagramType::Context => {
            let diagram = YamlStore::load_context(src)?;
            validate(&diagram)
        }
        DiagramType::ConceptModel => {
            let model = YamlStore::load_concept_model(src)?;
            validate_concept_model(&model)
        }
        DiagramType::LogicView => {
            let diagram = YamlStore::load_logic_view(src)?;
            validate_logic_concept(&diagram)
        }
    };

    result.is_valid = !result.errors.iter().any(|e| e.severity == Severity::Error);
    Ok(result)
}

fn validate_from_workspace(
    workspace: &crate::model::workspace::Workspace,
    diagram_type: &DiagramType,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    // First, validate the requested diagram type
    match diagram_type {
        DiagramType::Context => {
            match &workspace.context_diagram {
                Some(diagram) => result.merge(validate(diagram)),
                None => return Err(AppError::ElementNotFound(
                    "context_diagram not found in workspace".to_string()
                )),
            }
        }
        DiagramType::ConceptModel => {
            match &workspace.logic_architecture_concept_model {
                Some(model) => result.merge(validate_concept_model(model)),
                None => return Err(AppError::ElementNotFound(
                    "logic_architecture_concept_model not found in workspace".to_string()
                )),
            }
        }
        DiagramType::LogicView => {
            match &workspace.logic_view {
                Some(diagram) => {
                    // If concept model exists, validate logic view against it
                    if let Some(concept_model) = &workspace.logic_architecture_concept_model {
                        result.merge(validate_logic_view_against_concept_model(diagram, concept_model));
                    } else {
                        // Just validate the logic view itself
                        result.merge(validate_logic_concept(diagram));
                    }
                }
                None => {
                    // Logic view doesn't exist - this is only an error if we're explicitly validating it
                    return Err(AppError::ElementNotFound(
                        "logic_view not found in workspace".to_string()
                    ));
                }
            }
        }
    }

    // Always check for unused concept model elements if concept model exists
    if let Some(concept_model) = &workspace.logic_architecture_concept_model {
        validate_workspace_unused_elements(workspace, concept_model, &mut result);
    }

    // Always check for orphan elements if both logic_view and concept_model exist
    if let (Some(logic_view), Some(concept_model)) = (&workspace.logic_view, &workspace.logic_architecture_concept_model) {
        validate_orphan_elements(logic_view, concept_model, &mut result);
    }

    result.is_valid = !result.errors.iter().any(|e| e.severity == Severity::Error);

    Ok(result)
}

/// Validate that all element types defined in concept model are used somewhere in the workspace
fn validate_workspace_unused_elements(
    workspace: &crate::model::workspace::Workspace,
    concept_model: &crate::model::logic::concept_model::LogicArchitectureConceptModel,
    result: &mut ValidationResult,
) {
    use crate::validator::result::ValidationError;

    // Collect used element types from logic view if it exists
    let mut used_types: Vec<&str> = Vec::new();

    if let Some(logic_view) = &workspace.logic_view {
        // Check for subsystems
        if !logic_view.system.subsystems.is_empty() {
            used_types.push("SUBSYSTEM");
        }

        // Check for components (both in system and in subsystems)
        if !logic_view.system.components.is_empty() {
            used_types.push("COMPONENT");
        }
        for sub in &logic_view.system.subsystems {
            if !sub.components.is_empty() {
                used_types.push("COMPONENT");
                break;
            }
        }

        // Check for modules
        let has_modules = logic_view.system.components.iter().any(|c| !c.modules.is_empty())
            || logic_view.system.subsystems.iter().any(|s| s.components.iter().any(|c| !c.modules.is_empty()));
        if has_modules {
            used_types.push("MODULE");
        }
    }

    // Check if any containment relationships are defined in the concept model
    let has_containment_rules = !concept_model.hierarchy.levels.is_empty();

    // Check for unused element types
    let unused = concept_model.get_unused_element_types(&used_types);
    for unused_type in unused {
        // If no containment rules are defined, it's an error (model incomplete)
        // If containment rules exist but element not instantiated, it's a warning (valid structure)
        let severity = if has_containment_rules {
            Severity::Warning
        } else {
            Severity::Error
        };

        result.add_error(ValidationError {
            code: "H002".to_string(),
            rule: "UnusedElementType".to_string(),
            message: format!(
                "Element type '{}' is defined in concept model but not used in logic view",
                unused_type
            ),
            severity,
            location: Some("logic_architecture_concept_model.element_types".to_string()),
        });
    }
}

/// Validate logic view against concept model hierarchy
fn validate_logic_view_against_concept_model(
    logic_view: &crate::model::logic::concept::LogicConceptDiagram,
    concept_model: &crate::model::logic::concept_model::LogicArchitectureConceptModel,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    // First run standard validation
    let standard_result = validate_logic_concept(logic_view);
    result.errors.extend(standard_result.errors);

    // Validate hierarchy conformance
    validate_hierarchy_conformance(logic_view, concept_model, &mut result);

    // Validate unused element types
    validate_unused_element_types(logic_view, concept_model, &mut result);

    // Validate orphan elements (elements without containment relationships)
    validate_orphan_elements(logic_view, concept_model, &mut result);

    result.is_valid = !result.errors.iter().any(|e| e.severity == Severity::Error);

    result
}

/// Validate that all element types defined in concept model are used in logic view
fn validate_unused_element_types(
    logic_view: &crate::model::logic::concept::LogicConceptDiagram,
    concept_model: &crate::model::logic::concept_model::LogicArchitectureConceptModel,
    result: &mut ValidationResult,
) {
    use crate::validator::result::ValidationError;

    // Collect used element types from logic view
    let mut used_types: Vec<&str> = Vec::new();

    // Check for subsystems
    if !logic_view.system.subsystems.is_empty() {
        used_types.push("SUBSYSTEM");
    }

    // Check for components (both in system and in subsystems)
    if !logic_view.system.components.is_empty() {
        used_types.push("COMPONENT");
    }
    for sub in &logic_view.system.subsystems {
        if !sub.components.is_empty() {
            used_types.push("COMPONENT");
            break;
        }
    }

    // Check for modules (in system.modules, components, or subsystems)
    let has_modules = !logic_view.system.modules.is_empty()
        || logic_view.system.components.iter().any(|c| !c.modules.is_empty())
        || logic_view.system.subsystems.iter().any(|s| s.components.iter().any(|c| !c.modules.is_empty()));
    if has_modules {
        used_types.push("MODULE");
    }

    // Check for submodules (nested modules)
    fn has_nested_modules(modules: &[crate::model::logic::concept::Module]) -> bool {
        modules.iter().any(|m| !m.modules.is_empty() || has_nested_modules(&m.modules))
    }
    let has_submodules = has_nested_modules(&logic_view.system.modules)
        || logic_view.system.components.iter().any(|c| has_nested_modules(&c.modules))
        || logic_view.system.subsystems.iter().any(|s| s.components.iter().any(|c| has_nested_modules(&c.modules)));
    if has_submodules {
        used_types.push("SUBMODULE");
    }

    // Check if any containment relationships are defined in the concept model
    let has_containment_rules = !concept_model.hierarchy.levels.is_empty();

    // Check for unused element types
    let unused = concept_model.get_unused_element_types(&used_types);
    for unused_type in unused {
        // If no containment rules are defined, it's an error (model incomplete)
        // If containment rules exist but element not instantiated, it's a warning (valid structure)
        let severity = if has_containment_rules {
            Severity::Warning
        } else {
            Severity::Error
        };

        result.add_error(ValidationError {
            code: "H002".to_string(),
            rule: "UnusedElementType".to_string(),
            message: format!(
                "Element type '{}' is defined in concept model but not used in logic view",
                unused_type
            ),
            severity,
            location: Some("logic_architecture_concept_model.element_types".to_string()),
        });
    }
}

fn validate_hierarchy_conformance(
    logic_view: &crate::model::logic::concept::LogicConceptDiagram,
    concept_model: &crate::model::logic::concept_model::LogicArchitectureConceptModel,
    result: &mut ValidationResult,
) {
    use crate::validator::result::ValidationError;

    // Check System level
    // System can contain: SUBSYSTEM, COMPONENT (based on default hierarchy)
    if !logic_view.system.subsystems.is_empty() {
        if !concept_model.can_contain("SYSTEM", "SUBSYSTEM") {
            result.add_error(ValidationError {
                code: "H001".to_string(),
                rule: "HierarchyConformance".to_string(),
                message: "System cannot contain Subsystem according to concept model".to_string(),
                severity: Severity::Error,
                location: Some("system.subsystems".to_string()),
            });
        }
    }

    if !logic_view.system.components.is_empty() {
        if !concept_model.can_contain("SYSTEM", "COMPONENT") {
            result.add_error(ValidationError {
                code: "H001".to_string(),
                rule: "HierarchyConformance".to_string(),
                message: "System cannot contain Component according to concept model".to_string(),
                severity: Severity::Error,
                location: Some("system.components".to_string()),
            });
        }
    }

    // Check for modules directly under system
    if !logic_view.system.modules.is_empty() {
        if !concept_model.can_contain("SYSTEM", "MODULE") {
            result.add_error(ValidationError {
                code: "H001".to_string(),
                rule: "HierarchyConformance".to_string(),
                message: "System cannot contain Module according to concept model".to_string(),
                severity: Severity::Error,
                location: Some("system.modules".to_string()),
            });
        }
        // Validate module hierarchy for system-level modules
        for module in &logic_view.system.modules {
            validate_module_hierarchy(module, concept_model, result, "system.modules");
        }
    }

    // Check Subsystem level
    for subsystem in &logic_view.system.subsystems {
        if !subsystem.components.is_empty() {
            if !concept_model.can_contain("SUBSYSTEM", "COMPONENT") {
                result.add_error(ValidationError {
                    code: "H001".to_string(),
                    rule: "HierarchyConformance".to_string(),
                    message: format!(
                        "Subsystem '{}' cannot contain Component according to concept model",
                        subsystem.id
                    ),
                    severity: Severity::Error,
                    location: Some(format!("subsystems.{}.components", subsystem.id)),
                });
            }
        }
    }

    // Check Component level
    for component in &logic_view.system.components {
        validate_component_hierarchy(component, concept_model, result);
    }
    for subsystem in &logic_view.system.subsystems {
        for component in &subsystem.components {
            validate_component_hierarchy(component, concept_model, result);
        }
    }
}

fn validate_component_hierarchy(
    component: &crate::model::logic::concept::Component,
    concept_model: &crate::model::logic::concept_model::LogicArchitectureConceptModel,
    result: &mut ValidationResult,
) {
    use crate::validator::result::ValidationError;

    if !component.modules.is_empty() {
        if !concept_model.can_contain("COMPONENT", "MODULE") {
            result.add_error(ValidationError {
                code: "H001".to_string(),
                rule: "HierarchyConformance".to_string(),
                message: format!(
                    "Component '{}' cannot contain Module according to concept model",
                    component.id
                ),
                severity: Severity::Error,
                location: Some(format!("components.{}.modules", component.id)),
            });
        }
    }

    // Check Module level (recursive)
    for module in &component.modules {
        validate_module_hierarchy(module, concept_model, result, &format!(
            "components.{}.modules",
            component.id
        ));
    }
}

fn validate_module_hierarchy(
    module: &crate::model::logic::concept::Module,
    concept_model: &crate::model::logic::concept_model::LogicArchitectureConceptModel,
    result: &mut ValidationResult,
    location: &str,
) {
    use crate::validator::result::ValidationError;

    if !module.modules.is_empty() {
        if !concept_model.can_contain("MODULE", "MODULE") {
            result.add_error(ValidationError {
                code: "H001".to_string(),
                rule: "HierarchyConformance".to_string(),
                message: format!(
                    "Module '{}' cannot contain nested Module according to concept model",
                    module.id
                ),
                severity: Severity::Error,
                location: Some(format!("{}.{}.modules", location, module.id)),
            });
        }
    }

    // Recursively check nested modules
    for nested in &module.modules {
        validate_module_hierarchy(
            nested,
            concept_model,
            result,
            &format!("{}.{}", location, module.id),
        );
    }
}

/// Validate that all elements have containment relationships (no orphan elements)
fn validate_orphan_elements(
    logic_view: &crate::model::logic::concept::LogicConceptDiagram,
    concept_model: &crate::model::logic::concept_model::LogicArchitectureConceptModel,
    result: &mut ValidationResult,
) {
    use crate::validator::result::ValidationError;

    // Only validate if concept model has containment rules defined
    if concept_model.hierarchy.levels.is_empty() {
        return;
    }

    // Collect all child IDs from containment relationships
    let contained_children: std::collections::HashSet<&str> = logic_view
        .containments
        .iter()
        .map(|c| c.child_id.as_str())
        .collect();

    // Check subsystems
    for sub in &logic_view.system.subsystems {
        if !contained_children.contains(sub.id.as_str()) {
            result.add_error(ValidationError {
                code: "O001".to_string(),
                rule: "OrphanElement".to_string(),
                message: format!(
                    "Subsystem '{}' is not contained by any parent element. Add containment relationship.",
                    sub.id
                ),
                severity: Severity::Error,
                location: Some(format!("subsystems.{}", sub.id)),
            });
        }
    }

    // Check components
    for comp in &logic_view.system.components {
        if !contained_children.contains(comp.id.as_str()) {
            result.add_error(ValidationError {
                code: "O001".to_string(),
                rule: "OrphanElement".to_string(),
                message: format!(
                    "Component '{}' is not contained by any parent element. Add containment relationship.",
                    comp.id
                ),
                severity: Severity::Error,
                location: Some(format!("components.{}", comp.id)),
            });
        }
    }

    // Check modules at system level
    for module in &logic_view.system.modules {
        if !contained_children.contains(module.id.as_str()) {
            result.add_error(ValidationError {
                code: "O001".to_string(),
                rule: "OrphanElement".to_string(),
                message: format!(
                    "Module '{}' is not contained by any parent element. Add containment relationship.",
                    module.id
                ),
                severity: Severity::Error,
                location: Some(format!("modules.{}", module.id)),
            });
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

            println!(
                "  [{}] {}: {}",
                severity_str,
                error.code.white().bold(),
                error.message
            );

            if let Some(ref loc) = error.location {
                println!("         Location: {}", loc.dimmed());
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
