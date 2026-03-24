pub mod result;
pub mod rules;
pub mod logic_concept;

use crate::model::c4::context::ContextDiagram;
use crate::model::logic::concept::LogicConceptDiagram;
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
