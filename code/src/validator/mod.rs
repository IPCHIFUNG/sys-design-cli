pub mod result;
pub mod rules;
pub mod logic_concept;
pub mod concept_model;
pub mod runtime_view;
pub mod code_model;
pub mod build_model;
pub mod delivery_model;
pub mod deployment_model;

use crate::model::build::BuildModel;
use crate::model::delivery::DeliveryModel;
use crate::model::deployment::DeploymentModel;
use crate::model::c4::context::ContextDiagram;
use crate::model::code::CodeModel;
use crate::model::logic::concept::LogicConceptDiagram;
use crate::model::logic::concept_model::LogicArchitectureConceptModel;
use crate::model::runtime::RuntimeView;
use result::ValidationResult;

pub use result::{ValidationResult as ValResult, Severity};

/// Validate a ContextDiagram with all default rules
pub fn validate(diagram: &ContextDiagram) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Completeness checks
    result.merge(rules::completeness::validate(diagram));

    // Consistency checks
    result.merge(rules::consistency::validate(diagram));

    // Naming checks
    result.merge(rules::naming::validate(diagram));

    result
}

/// Validate a LogicConceptDiagram
pub fn validate_logic_concept(diagram: &LogicConceptDiagram) -> ValidationResult {
    logic_concept::validate(diagram)
}

/// Validate a LogicArchitectureConceptModel
pub fn validate_concept_model(model: &LogicArchitectureConceptModel) -> ValidationResult {
    concept_model::validate(model)
}

/// Validate a RuntimeView
pub fn validate_runtime_view(view: &RuntimeView) -> ValidationResult {
    runtime_view::validate(view)
}

/// Validate a CodeModel
pub fn validate_code_model(model: &CodeModel) -> ValidationResult {
    code_model::validate(model)
}

/// Validate a BuildModel
pub fn validate_build_model(model: &BuildModel) -> ValidationResult {
    build_model::validate(model)
}

/// Validate a DeliveryModel
pub fn validate_delivery_model(model: &DeliveryModel) -> ValidationResult {
    delivery_model::validate(model)
}

/// Validate a DeploymentModel
pub fn validate_deployment_model(model: &DeploymentModel) -> ValidationResult {
    deployment_model::validate(model)
}
