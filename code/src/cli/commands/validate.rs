use crate::cli::args::{DiagramType, OutputFormat};
use crate::store::YamlStore;
use crate::validator::{validate, validate_logic_concept, validate_concept_model, validate_runtime_view, validate_code_model, validate_build_model, validate_delivery_model, validate_deployment_model};
use crate::validator::result::{ValidationResult, ValidationError, Severity};
use crate::utils::error::{AppError, Result};
use std::path::Path;

pub fn execute(model_file: &Path, format: OutputFormat, diagram_type: DiagramType) -> Result<()> {
    let result = load_and_validate(model_file, &diagram_type)?;

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

fn load_and_validate(model_file: &Path, diagram_type: &DiagramType) -> Result<ValidationResult> {
    // Load as workspace (handles legacy formats via load_workspace_any)
    let workspace = YamlStore::load_workspace_any(model_file)?;
    validate_from_workspace(&workspace, diagram_type)
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
        DiagramType::RuntimeView => {
            match &workspace.runtime_view {
                Some(view) => {
                    result.merge(validate_runtime_view(view));
                    // Cross-diagram validation: participant element_id must exist
                    validate_runtime_participant_references(workspace, view, &mut result);
                }
                None => return Err(AppError::ElementNotFound(
                    "runtime_view not found in workspace".to_string()
                )),
            }
        }
        DiagramType::CodeModel => {
            match &workspace.code_model {
                Some(model) => {
                    result.merge(validate_code_model(model));
                    // Cross-diagram validation: element_id must exist in logic_view
                    validate_code_model_element_references(workspace, model, &mut result);
                }
                None => return Err(AppError::ElementNotFound(
                    "code_model not found in workspace".to_string()
                )),
            }
        }
        DiagramType::BuildModel => {
            match &workspace.build_model {
                Some(model) => {
                    result.merge(validate_build_model(model));
                    // Cross-diagram validation: source_packages must exist in code_model
                    validate_build_model_source_references(workspace, model, &mut result);
                }
                None => return Err(AppError::ElementNotFound(
                    "build_model not found in workspace".to_string()
                )),
            }
        }
        DiagramType::DeliveryModel => {
            match &workspace.delivery_model {
                Some(model) => {
                    result.merge(validate_delivery_model(model));
                    // Cross-diagram validation: artifacts must exist in build_model
                    validate_delivery_model_artifact_references(workspace, model, &mut result);
                }
                None => return Err(AppError::ElementNotFound(
                    "delivery_model not found in workspace".to_string()
                )),
            }
        }
        DiagramType::DeploymentModel => {
            match &workspace.deployment_model {
                Some(model) => {
                    result.merge(validate_deployment_model(model));
                    // Cross-diagram validation: delivery_package must exist in delivery_model
                    validate_deployment_service_delivery_references(workspace, model, &mut result);
                }
                None => return Err(AppError::ElementNotFound(
                    "deployment_model not found in workspace".to_string()
                )),
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

    // SYSTEM is used if context_diagram has a system
    if workspace.context_diagram.is_some() {
        used_types.push("SYSTEM");
    }

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

        // Check for modules (in system.modules, components, or subsystems)
        let has_modules = !logic_view.system.modules.is_empty()
            || logic_view.system.components.iter().any(|c| !c.modules.is_empty())
            || logic_view.system.subsystems.iter().any(|s| s.components.iter().any(|c| !c.modules.is_empty()));
        if has_modules {
            used_types.push("MODULE");
        }

        // Check for submodules (via submodule_ids)
        if !logic_view.submodule_ids.is_empty() {
            used_types.push("SUBMODULE");
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
    if !logic_view.system.subsystems.is_empty()
        && !concept_model.can_contain("SYSTEM", "SUBSYSTEM")
    {
        result.add_error(ValidationError {
            code: "H001".to_string(),
            rule: "HierarchyConformance".to_string(),
            message: "System cannot contain Subsystem according to concept model".to_string(),
            severity: Severity::Error,
            location: Some("system.subsystems".to_string()),
        });
    }

    if !logic_view.system.components.is_empty()
        && !concept_model.can_contain("SYSTEM", "COMPONENT")
    {
        result.add_error(ValidationError {
            code: "H001".to_string(),
            rule: "HierarchyConformance".to_string(),
            message: "System cannot contain Component according to concept model".to_string(),
            severity: Severity::Error,
            location: Some("system.components".to_string()),
        });
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
        if !subsystem.components.is_empty()
            && !concept_model.can_contain("SUBSYSTEM", "COMPONENT")
        {
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

    if !component.modules.is_empty()
        && !concept_model.can_contain("COMPONENT", "MODULE")
    {
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

    if !module.modules.is_empty()
        && !concept_model.can_contain("MODULE", "MODULE")
    {
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

/// Validate that runtime view participants reference existing elements in static models
fn validate_runtime_participant_references(
    workspace: &crate::model::workspace::Workspace,
    view: &crate::model::runtime::RuntimeView,
    result: &mut ValidationResult,
) {
    // Collect all valid element IDs from static models
    let mut valid_ids: Vec<&str> = Vec::new();

    if let Some(ctx) = &workspace.context_diagram {
        valid_ids.extend(ctx.all_element_ids());
    }

    if let Some(lv) = &workspace.logic_view {
        valid_ids.extend(lv.all_element_ids());
    }

    // Check each participant in each scenario
    for scenario in &view.scenarios {
        for participant in &scenario.participants {
            if !valid_ids.contains(&participant.element_id.as_str()) {
                result.add_error(ValidationError {
                    code: "R007".to_string(),
                    rule: "ElementReferenceExists".to_string(),
                    message: format!(
                        "Participant '{}' in scenario '{}' does not exist in context_diagram or logic_view",
                        participant.element_id, scenario.id
                    ),
                    severity: Severity::Error,
                    location: Some(format!(
                        "runtime_view.scenarios.{}.participants.{}",
                        scenario.id, participant.element_id
                    )),
                });
            }
        }
    }
}

/// Validate that code model element_id references exist in logic_view
fn validate_code_model_element_references(
    workspace: &crate::model::workspace::Workspace,
    model: &crate::model::code::CodeModel,
    result: &mut ValidationResult,
) {
    for pkg in &model.packages {
        if let Some(ref element_id) = pkg.element_id {
            let found = workspace
                .logic_view
                .as_ref()
                .map(|lv| lv.get_element_name(element_id).is_some())
                .unwrap_or(false);

            if !found {
                result.add_error(ValidationError {
                    code: "CM010".to_string(),
                    rule: "ElementReferenceExists".to_string(),
                    message: format!(
                        "Package '{}' element_id '{}' does not exist in logic_view",
                        pkg.id, element_id
                    ),
                    severity: Severity::Error,
                    location: Some(format!("code_model.packages.{}.element_id", pkg.id)),
                });
            }
        }
    }
}

/// Validate that build model source_packages references exist in code_model
fn validate_build_model_source_references(
    workspace: &crate::model::workspace::Workspace,
    model: &crate::model::build::BuildModel,
    result: &mut ValidationResult,
) {
    if workspace.code_model.is_none() {
        return;
    }

    let code_model = workspace.code_model.as_ref().unwrap();

    for artifact in &model.artifacts {
        for pkg_id in &artifact.source_packages {
            if code_model.find_package(pkg_id).is_none() {
                result.add_error(ValidationError {
                    code: "BM006".to_string(),
                    rule: "SourcePackageReferenceExists".to_string(),
                    message: format!(
                        "Artifact '{}' source_package '{}' does not exist in code_model",
                        artifact.id, pkg_id
                    ),
                    severity: Severity::Error,
                    location: Some(format!("build_model.artifacts.{}.source_packages", artifact.id)),
                });
            }
        }
    }
}

/// Validate that delivery model artifact references exist in build_model
fn validate_delivery_model_artifact_references(
    workspace: &crate::model::workspace::Workspace,
    model: &crate::model::delivery::DeliveryModel,
    result: &mut ValidationResult,
) {
    if workspace.build_model.is_none() {
        return;
    }

    let build_model = workspace.build_model.as_ref().unwrap();

    for package in &model.packages {
        for art_id in &package.artifacts {
            if build_model.find_artifact(art_id).is_none() {
                result.add_error(ValidationError {
                    code: "DM006".to_string(),
                    rule: "ArtifactReferenceExists".to_string(),
                    message: format!(
                        "Package '{}' artifact '{}' does not exist in build_model",
                        package.id, art_id
                    ),
                    severity: Severity::Error,
                    location: Some(format!("delivery_model.packages.{}.artifacts", package.id)),
                });
            }
        }
    }
}

/// Validate that deployment model service delivery_package references exist in delivery_model
fn validate_deployment_service_delivery_references(
    workspace: &crate::model::workspace::Workspace,
    model: &crate::model::deployment::DeploymentModel,
    result: &mut ValidationResult,
) {
    if workspace.delivery_model.is_none() {
        return;
    }

    let delivery_model = workspace.delivery_model.as_ref().unwrap();

    for service in &model.services {
        if delivery_model.find_package(&service.delivery_package).is_none() {
            result.add_error(ValidationError {
                code: "DP020".to_string(),
                rule: "ServiceDeliveryPackageExists".to_string(),
                message: format!(
                    "Service '{}' delivery_package '{}' does not exist in delivery_model",
                    service.id, service.delivery_package
                ),
                severity: Severity::Error,
                location: Some(format!("deployment_model.services.{}.delivery_package", service.id)),
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
