use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub code: String,
    pub rule: String,
    pub message: String,
    pub severity: Severity,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        if error.severity == Severity::Error {
            self.is_valid = false;
        }
        self.errors.push(error);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
    }

    pub fn error_count(&self) -> usize {
        self.errors.iter().filter(|e| e.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.errors.iter().filter(|e| e.severity == Severity::Warning).count()
    }

    pub fn info_count(&self) -> usize {
        self.errors.iter().filter(|e| e.severity == Severity::Info).count()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Validation Result:")?;
        writeln!(
            f,
            "  Errors: {}, Warnings: {}, Info: {}",
            self.error_count(),
            self.warning_count(),
            self.info_count()
        )?;

        for error in &self.errors {
            let severity_str = match error.severity {
                Severity::Error => "ERROR",
                Severity::Warning => "WARN",
                Severity::Info => "INFO",
            };
            writeln!(f, "  [{}] {}: {}", severity_str, error.code, error.message)?;
            if let Some(ref loc) = error.location {
                writeln!(f, "         Location: {}", loc)?;
            }
        }

        if self.is_valid {
            writeln!(f, "\nValidation passed!")?;
        } else {
            writeln!(f, "\nValidation failed.")?;
        }

        Ok(())
    }
}
