use cjtoolkit_structured_validator::common::locale::{
    LocaleData, LocaleMessage, ValidateErrorCollector,
};
use cjtoolkit_structured_validator::common::validation_check::ValidationCheck;
use cjtoolkit_structured_validator::types::name::name_alias::{Field, FieldError, FieldRules};
use regex::Regex;
use std::sync::{Arc, OnceLock};

fn url_path_rule() -> FieldRules {
    FieldRules {
        is_mandatory: true,
        min_length: Some(1),
        max_length: Some(100),
    }
}

struct MustBeKebabCaseLocale;

impl LocaleMessage for MustBeKebabCaseLocale {
    fn get_locale_data(&self) -> Arc<LocaleData> {
        LocaleData::new("validate-must-be-kebab-case")
    }
}

static KEBAB_CASE_REGEX_CACHE: OnceLock<Regex> = OnceLock::new();

fn must_be_kebab_case(url_path: &str) -> Result<(), FieldError> {
    let mut messages = ValidateErrorCollector::new();
    // check kebab case
    let regex = KEBAB_CASE_REGEX_CACHE
        .get_or_init(|| Regex::new(r"^([a-z0-9]+(-[a-z0-9]+)*)+$").expect("Invalid regex"));
    if !regex.is_match(url_path) {
        messages.push((
            "Must be kebab case".to_string(),
            Box::new(MustBeKebabCaseLocale),
        ));
    }
    FieldError::validate_check(messages)?;
    Ok(())
}

pub trait UrlPathRulesExt {
    fn parse_url_path(url_path: Option<&str>) -> Result<Field, FieldError>;
}

impl UrlPathRulesExt for Field {
    fn parse_url_path(url_path: Option<&str>) -> Result<Field, FieldError> {
        let url_path = Field::parse_custom(url_path, url_path_rule());
        if let Ok(url_path_ref) = url_path.as_ref() {
            must_be_kebab_case(url_path_ref.as_str())?;
        }
        url_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_must_be_kebab_case() {
        let url_path = "test-url-path";
        let result = must_be_kebab_case(url_path);
        assert!(result.is_ok());

        let url_path = "test-url-path_1";
        let result = must_be_kebab_case(url_path);
        assert!(result.is_err());
    }
}
