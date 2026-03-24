pub mod result;
pub mod rules;

use crate::model::c4::context::ContextDiagram;
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
