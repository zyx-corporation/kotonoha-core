//! RDE attach helpers: validation reports and source-kind labels (M2).

use crate::rde;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Input channel for an RDE assessment row (`rde_assessments.source_kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RdeSourceKind {
    Cli,
    Llm,
    Import,
    Replay,
}

impl RdeSourceKind {
    pub fn as_db_str(self) -> &'static str {
        match self {
            RdeSourceKind::Cli => "cli",
            RdeSourceKind::Llm => "llm",
            RdeSourceKind::Import => "import",
            RdeSourceKind::Replay => "replay",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "cli" => Some(RdeSourceKind::Cli),
            "llm" => Some(RdeSourceKind::Llm),
            "import" => Some(RdeSourceKind::Import),
            "replay" => Some(RdeSourceKind::Replay),
            _ => None,
        }
    }
}

/// Machine-readable summary stored in `rde_assessments.validation_report`.
pub fn build_validation_report(strict: bool, warnings: &[String]) -> Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    json!({
        "strict": strict,
        "valid": true,
        "warnings": warnings,
        "warning_count": warnings.len(),
        "validated_at_unix": now,
    })
}

/// Validates payload before attach. Returns RDE **warnings** when `strict` is false.
///
/// When `strict` is true, non-empty warnings are treated as failure (M2 trust boundary).
pub fn validate_rde_payload_for_attach(
    payload: &Value,
    strict_rde: bool,
) -> Result<Vec<String>, String> {
    if payload.get("rde_review_output").is_none() {
        return Ok(Vec::new());
    }
    let json = serde_json::to_string(payload).map_err(|e| format!("payload JSON: {e}"))?;
    let warnings = rde::validate_json(&json, strict_rde)?;
    if strict_rde && !warnings.is_empty() {
        return Err(format!(
            "strict RDE validation failed with {} warning(s): {}",
            warnings.len(),
            warnings.join("; ")
        ));
    }
    Ok(warnings)
}

/// Extract `spec_version` from an RDE payload when `rde_review_output` is present.
pub fn payload_schema_version_from_payload(payload: &Value) -> Option<String> {
    payload
        .get("rde_review_output")
        .and_then(|o| o.get("spec_version"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_kind_parse() {
        assert_eq!(RdeSourceKind::parse("CLI"), Some(RdeSourceKind::Cli));
        assert_eq!(RdeSourceKind::parse("nope"), None);
    }

    #[test]
    fn validation_report_includes_warnings() {
        let r = build_validation_report(false, &["warn a".into()]);
        assert_eq!(r["warning_count"], 1);
        assert_eq!(r["strict"], false);
    }

    #[test]
    fn payload_schema_version_reads_spec_version() {
        let p = json!({
            "rde_review_output": {
                "spec_version": "0.1",
                "subject_ref": "https://example.invalid/x",
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
        });
        assert_eq!(
            payload_schema_version_from_payload(&p).as_deref(),
            Some("0.1")
        );
    }

    #[test]
    fn strict_attach_rejects_summary_warnings() {
        let p = json!({
            "rde_review_output": {
                "spec_version": "0.1",
                "subject_ref": "https://example.invalid/x",
                "categories": {
                    "preserved": [{}],
                    "transformed": [],
                    "complemented": [],
                    "intentionally_unresolved": [],
                    "lost": [],
                    "deviation_risk": [],
                    "next_update_policy": []
                }
            }
        });
        assert!(validate_rde_payload_for_attach(&p, true).is_err());
    }

    #[test]
    fn non_strict_attach_collects_summary_warnings() {
        let p = json!({
            "rde_review_output": {
                "spec_version": "0.1",
                "subject_ref": "https://example.invalid/x",
                "categories": {
                    "preserved": [{}],
                    "transformed": [],
                    "complemented": [],
                    "intentionally_unresolved": [],
                    "lost": [],
                    "deviation_risk": [],
                    "next_update_policy": []
                }
            }
        });
        assert!(!validate_rde_payload_for_attach(&p, false)
            .unwrap()
            .is_empty());
    }
}
