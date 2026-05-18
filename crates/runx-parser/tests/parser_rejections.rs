use std::collections::BTreeSet;

use runx_parser::{
    ParseErrorKind, ValidationErrorKind, assert_yaml_scalar_subset, parse_graph_yaml,
    parse_skill_markdown, parse_tool_manifest_json, validate_graph, validate_skill,
};

#[test]
fn parse_rejections_cover_every_error_kind() -> Result<(), String> {
    let mut kinds = BTreeSet::new();

    kinds.insert(parse_error_kind(parse_graph_yaml("name: [unterminated\n"))?);
    kinds.insert(parse_error_kind(parse_tool_manifest_json("{"))?);
    kinds.insert(parse_error_kind(parse_skill_markdown(
        "# missing frontmatter\n",
    ))?);
    kinds.insert(parse_error_kind(assert_yaml_scalar_subset(
        "fixture", "yes",
    ))?);

    assert_eq!(
        kinds,
        BTreeSet::from([
            ParseErrorKind::InvalidYaml,
            ParseErrorKind::InvalidJson,
            ParseErrorKind::InvalidDocument,
            ParseErrorKind::UnsupportedScalar,
        ]),
    );
    Ok(())
}

#[test]
fn validation_rejections_cover_every_error_kind() -> Result<(), String> {
    let mut kinds = BTreeSet::new();

    let missing_step_id = parse_graph_yaml(
        r#"
name: bad
steps:
  - skill: ../../skills/echo
"#,
    )
    .map_err(|error| error.to_string())?;
    kinds.insert(validation_error_kind(validate_graph(missing_step_id))?);

    let invalid_fanout_gate = parse_graph_yaml(
        r#"
name: fanout
fanout:
  groups:
    advisors:
      threshold_gates:
        - step: risk
          field: risk_score
          above: 0.8
          action: pause
          sentiment: negative
steps:
  - id: risk
    mode: fanout
    fanout_group: advisors
    skill: ../../skills/echo
"#,
    )
    .map_err(|error| error.to_string())?;
    kinds.insert(validation_error_kind(validate_graph(invalid_fanout_gate))?);

    assert_eq!(
        kinds,
        BTreeSet::from([
            ValidationErrorKind::MissingField,
            ValidationErrorKind::InvalidField,
        ]),
    );
    Ok(())
}

#[test]
fn strict_skill_validation_matches_runx_object_error() -> Result<(), String> {
    let raw = parse_skill_markdown(
        r#"---
name: bad-runx
runx: invalid
---
Body
"#,
    )
    .map_err(|error| error.to_string())?;

    match validate_skill(raw) {
        Ok(_) => Err("expected strict runx validation rejection".to_owned()),
        Err(error) => {
            assert_eq!(error.to_string(), "runx must be an object when present.");
            Ok(())
        }
    }
}

fn parse_error_kind<T>(
    result: Result<T, runx_parser::ParseError>,
) -> Result<ParseErrorKind, String> {
    match result {
        Ok(_) => Err("expected parse rejection".to_owned()),
        Err(error) => Ok(error.kind()),
    }
}

fn validation_error_kind<T>(
    result: Result<T, runx_parser::ValidationError>,
) -> Result<ValidationErrorKind, String> {
    match result {
        Ok(_) => Err("expected validation rejection".to_owned()),
        Err(error) => Ok(error.kind()),
    }
}
