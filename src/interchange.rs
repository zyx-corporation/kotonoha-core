//! **Exchangeable intermediate representation** — bundles lineage and/or RDE payloads for tool interchange.
//!
//! This is **not** normative in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec); it is a core-supported envelope so deployments can pass a single JSON artifact between pipelines.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::lineage::LineageUnit;

/// Top-level `format` discriminator for [`InterchangeDocument`].
pub const INTERCHANGE_FORMAT_V1: &str = "kotonoha.interchange.v1";

/// Document envelope for exchanging lineage and/or RDE review output between tools (`kotonoha-core` Phase 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterchangeDocument {
    /// Must be [`INTERCHANGE_FORMAT_V1`].
    pub format: String,
    /// Must match [`crate::TARGET_SPEC_BUNDLE`] for this crate release.
    pub spec_bundle: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_unit: Option<LineageUnit>,
    /// Full JSON document accepted by [`crate::rde::validate_json`] (includes `rde_review_output` wrapper).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rde_document: Option<Value>,
}

impl InterchangeDocument {
    /// Parses JSON and validates envelope rules (not including nested RDE `strict` summary checks — pass to [`validate_interchange_json`]).
    pub fn from_json_str(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| format!("invalid JSON: {e}"))
    }
}

/// Validates an interchange JSON document: envelope fields, optional lineage unit, optional RDE payload.
///
/// Returns RDE **warnings** (summary `SHOULD`) when `strict_rde` is false; lineage errors fail the whole validation.
pub fn validate_interchange_json(text: &str, strict_rde: bool) -> Result<Vec<String>, String> {
    let doc = InterchangeDocument::from_json_str(text)?;

    if doc.format != INTERCHANGE_FORMAT_V1 {
        return Err(format!(
            "\"format\" must be {:?} (got {:?})",
            INTERCHANGE_FORMAT_V1, doc.format
        ));
    }

    if doc.spec_bundle != crate::TARGET_SPEC_BUNDLE {
        return Err(format!(
            "\"spec_bundle\" must match kotonoha-core target {:?} (got {:?})",
            crate::TARGET_SPEC_BUNDLE,
            doc.spec_bundle
        ));
    }

    if doc.lineage_unit.is_none() && doc.rde_document.is_none() {
        return Err(
            "interchange document must include at least one of \"lineage_unit\" or \"rde_document\""
                .to_string(),
        );
    }

    if let Some(ref u) = doc.lineage_unit {
        u.validate().map_err(|_| {
            "lineage_unit: \"id\" must be non-empty (semantic-lineage-model)".to_string()
        })?;
    }

    let mut warnings = Vec::new();

    if let Some(v) = doc.rde_document {
        let rde_text =
            serde_json::to_string(&v).map_err(|e| format!("rde_document serialization: {e}"))?;
        let w = crate::rde::validate_json(&rde_text, strict_rde)?;
        warnings.extend(w);
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_lineage_only() -> String {
        serde_json::json!({
            "format": INTERCHANGE_FORMAT_V1,
            "spec_bundle": crate::TARGET_SPEC_BUNDLE,
            "lineage_unit": {
                "id": "https://example.invalid/l/1",
                "prior_unit_id": null
            }
        })
        .to_string()
    }

    #[test]
    fn accepts_lineage_only() {
        assert!(validate_interchange_json(&minimal_lineage_only(), false)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn rejects_empty_envelope() {
        let j = serde_json::json!({
            "format": INTERCHANGE_FORMAT_V1,
            "spec_bundle": crate::TARGET_SPEC_BUNDLE,
        })
        .to_string();
        assert!(validate_interchange_json(&j, false).is_err());
    }

    #[test]
    fn accepts_with_nested_rde() {
        let j = serde_json::json!({
            "format": INTERCHANGE_FORMAT_V1,
            "spec_bundle": crate::TARGET_SPEC_BUNDLE,
            "lineage_unit": { "id": "https://example.invalid/l/2" },
            "rde_document": {
                "rde_review_output": {
                    "spec_version": crate::TARGET_SPEC_BUNDLE,
                    "subject_ref": "https://example.invalid/s",
                    "categories": {
                        "preserved": [],
                        "transformed": [],
                        "complemented": [],
                        "intentionally_unresolved": [],
                        "lost": [],
                        "deviation_risk": [],
                        "next_update_policy": []
                    }
                }
            }
        })
        .to_string();
        assert!(validate_interchange_json(&j, false).unwrap().is_empty());
    }
}
