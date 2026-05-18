use serde::de::DeserializeOwned;

use crate::ParseError;

const DIVERGENT_BOOLISH: &[&str] = &["yes", "no", "on", "off"];

pub fn parse_yaml_document<T>(source: &str) -> Result<T, ParseError>
where
    T: DeserializeOwned,
{
    serde_yml::from_str(source).map_err(|error| ParseError::InvalidYaml {
        field: "yaml".to_owned(),
        message: error.to_string(),
    })
}

#[must_use]
pub fn yaml_scalar_subset_allows(literal: &str) -> bool {
    let trimmed = literal.trim();
    !is_boolish(trimmed)
        && !is_base_prefixed_number(trimmed)
        && !is_sexagesimal_like(trimmed)
        && !is_date_like(trimmed)
        && !is_special_float(trimmed)
}

pub fn assert_yaml_scalar_subset(field: &str, literal: &str) -> Result<(), ParseError> {
    if yaml_scalar_subset_allows(literal) {
        return Ok(());
    }
    Err(ParseError::UnsupportedScalar {
        field: field.to_owned(),
        literal: literal.to_owned(),
    })
}

fn is_boolish(value: &str) -> bool {
    DIVERGENT_BOOLISH
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn is_base_prefixed_number(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    unsigned.starts_with("0x") || unsigned.starts_with("0X") || unsigned.starts_with("0o")
}

fn is_sexagesimal_like(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let mut parts = unsigned.split(':');
    let Some(first) = parts.next() else {
        return false;
    };
    first.chars().all(|char| char.is_ascii_digit())
        && parts.clone().count() > 0
        && parts.all(|part| !part.is_empty() && part.chars().all(|char| char.is_ascii_digit()))
}

fn is_date_like(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 10
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

fn is_special_float(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        ".nan" | ".inf" | "+.inf" | "-.inf"
    )
}

#[cfg(test)]
mod tests {
    use super::{assert_yaml_scalar_subset, yaml_scalar_subset_allows};

    #[test]
    fn scalar_subset_rejects_divergent_forms() {
        for literal in ["yes", "ON", "0x10", "0o10", "12:34", "2026-05-18", ".nan"] {
            assert!(!yaml_scalar_subset_allows(literal), "{literal}");
        }
    }

    #[test]
    fn scalar_subset_allows_explicit_json_like_scalars() -> Result<(), crate::ParseError> {
        for literal in ["true", "false", "1", "1.5", "plain text", "\"yes\""] {
            assert_yaml_scalar_subset("fixture", literal)?;
        }
        Ok(())
    }
}
