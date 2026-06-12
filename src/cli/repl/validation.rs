use reedline::{ValidationResult, Validator};
use scheme_rs::syntax::Syntax;

use super::commands::parse_error_is_incomplete;

pub(super) struct ReplValidator;

impl Validator for ReplValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(':') || matches!(trimmed, "(exit)" | ",q") {
            return ValidationResult::Complete;
        }
        match Syntax::from_str(line, Some("<repl>")) {
            Ok(_) => ValidationResult::Complete,
            Err(error) if parse_error_is_incomplete(&error) => ValidationResult::Incomplete,
            Err(_) => ValidationResult::Complete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repl_validator_keeps_incomplete_scheme_forms_multiline() {
        let validator = ReplValidator;

        assert!(matches!(
            validator.validate("(tool"),
            ValidationResult::Incomplete
        ));
        assert!(matches!(
            validator.validate("(tool \"rg\" (system \"rg\"))"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate(":doc tool"),
            ValidationResult::Complete
        ));
    }
}
