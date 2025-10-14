use cjtoolkit_structured_validator::types::name::name_alias::{Field, FieldError, FieldRules};

fn shorty_path_rule() -> FieldRules {
    FieldRules {
        is_mandatory: true,
        min_length: None,
        max_length: Some(100),
    }
}

pub trait ShortyPathRuleExt {
    fn parse_shorty_path(shorty_path: Option<&str>) -> Result<Field, FieldError>;
}

impl ShortyPathRuleExt for Field {
    fn parse_shorty_path(shorty_path: Option<&str>) -> Result<Field, FieldError> {
        Self::parse_custom(shorty_path, shorty_path_rule())
    }
}
