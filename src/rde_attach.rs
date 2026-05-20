//! RDE attach helpers: validation reports and source-kind labels (M2).

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
}
