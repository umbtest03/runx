use runx_contracts::JsonObject;
use serde::{Deserialize, Serialize};

use crate::ValidationError;

use super::{optional_string, required_string, validation_error};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogKind {
    Skill,
    Graph,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogAudience {
    Public,
    Builder,
    Operator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogVisibility {
    Public,
    Private,
}

impl CatalogKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CatalogKind::Skill => "skill",
            CatalogKind::Graph => "graph",
        }
    }
}

impl CatalogAudience {
    pub fn as_str(&self) -> &'static str {
        match self {
            CatalogAudience::Public => "public",
            CatalogAudience::Builder => "builder",
            CatalogAudience::Operator => "operator",
        }
    }
}

impl CatalogVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            CatalogVisibility::Public => "public",
            CatalogVisibility::Private => "private",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogMetadata {
    pub kind: CatalogKind,
    pub audience: CatalogAudience,
    pub visibility: CatalogVisibility,
}

pub(crate) fn validate_catalog_metadata(
    value: Option<JsonObject>,
    label: &str,
) -> Result<Option<CatalogMetadata>, ValidationError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let kind = match required_string(value.get("kind"), &format!("{label}.kind"))?.as_str() {
        "skill" => CatalogKind::Skill,
        "graph" => CatalogKind::Graph,
        _ => {
            return Err(validation_error(format!(
                "{label}.kind must be skill or graph."
            )));
        }
    };
    let audience =
        match required_string(value.get("audience"), &format!("{label}.audience"))?.as_str() {
            "public" => CatalogAudience::Public,
            "builder" => CatalogAudience::Builder,
            "operator" => CatalogAudience::Operator,
            _ => {
                return Err(validation_error(format!(
                    "{label}.audience must be public, builder, or operator."
                )));
            }
        };
    let visibility = match optional_string(value.get("visibility"), &format!("{label}.visibility"))?
        .as_deref()
    {
        Some("public") | None => CatalogVisibility::Public,
        Some("private") => CatalogVisibility::Private,
        Some(_) => {
            return Err(validation_error(format!(
                "{label}.visibility must be public or private."
            )));
        }
    };
    Ok(Some(CatalogMetadata {
        kind,
        audience,
        visibility,
    }))
}
