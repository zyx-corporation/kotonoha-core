//! M5 context pack export types (`kotonoha.context_pack.v0.1`).
//!
//! Issue: <https://github.com/zyx-corporation/kotonoha-core/issues/34>
//! Spec: <https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/31_m5_agent_run_integration_spec_draft.md> §5

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::observation_rde::{map_observation_to_rde_hints, ObservationRdeHints};
use crate::semantic_lineage::GitAnchor;

/// Envelope `format` field for M5 context export.
pub const CONTEXT_PACK_FORMAT: &str = "kotonoha.context_pack.v0.1";

/// Optional uncommitted MeaningDelta fields for agent channels (not persisted until human/CLI confirms).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeaningDeltaDraft {
    #[serde(default)]
    pub observation: Value,
    #[serde(default)]
    pub source_context: Value,
}

/// Input to build a [`ContextPack`].
#[derive(Debug, Clone)]
pub struct BuildContextPackInput {
    pub git_anchor: GitAnchor,
    pub meaning_delta_draft: Option<MeaningDeltaDraft>,
    pub policy_ref: Option<String>,
}

/// Context pack envelope for external agent channels (stdout JSON from CLI `context export`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPack {
    pub format: String,
    pub generated_at_unix: u64,
    pub git_anchor: GitAnchor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meaning_delta_draft: Option<MeaningDeltaDraft>,
    pub rde_hints: ObservationRdeHints,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_ref: Option<String>,
}

/// Errors when validating a context pack envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextPackError {
    WrongFormat {
        expected: &'static str,
        actual: String,
    },
    InvalidGitAnchor(crate::semantic_lineage::SemanticLineageError),
}

impl std::fmt::Display for ContextPackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextPackError::WrongFormat { expected, actual } => {
                write!(f, "context pack format: expected {expected}, got {actual}")
            }
            ContextPackError::InvalidGitAnchor(e) => write!(f, "git_anchor: {e}"),
        }
    }
}

impl std::error::Error for ContextPackError {}

/// Builds a new context pack with `generated_at_unix` from the system clock.
pub fn build_context_pack(input: BuildContextPackInput) -> ContextPack {
    let observation = input
        .meaning_delta_draft
        .as_ref()
        .map(|d| d.observation.clone())
        .unwrap_or(Value::Object(serde_json::Map::new()));
    let rde_hints = map_observation_to_rde_hints(&observation);
    let generated_at_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    ContextPack {
        format: CONTEXT_PACK_FORMAT.to_string(),
        generated_at_unix,
        git_anchor: input.git_anchor,
        meaning_delta_draft: input.meaning_delta_draft,
        rde_hints,
        policy_ref: input.policy_ref,
    }
}

/// Validates envelope `format` and embedded `git_anchor`.
pub fn validate_context_pack(pack: &ContextPack) -> Result<(), ContextPackError> {
    if pack.format != CONTEXT_PACK_FORMAT {
        return Err(ContextPackError::WrongFormat {
            expected: CONTEXT_PACK_FORMAT,
            actual: pack.format.clone(),
        });
    }
    pack.git_anchor
        .validate()
        .map_err(ContextPackError::InvalidGitAnchor)
}

/// Parses JSON and validates as a context pack.
pub fn parse_context_pack_json(json: &str) -> Result<ContextPack, ContextPackError> {
    let pack: ContextPack =
        serde_json::from_str(json).map_err(|e| ContextPackError::WrongFormat {
            expected: CONTEXT_PACK_FORMAT,
            actual: e.to_string(),
        })?;
    validate_context_pack(&pack)?;
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_anchor() -> GitAnchor {
        GitAnchor {
            git_commit: "07caa1e47d204e15547e70187e6e5a8feb4b5011".into(),
            file_path: "note.md".into(),
            line_range_start: Some(2),
            line_range_end: Some(2),
            diff_ref: None,
        }
    }

    #[test]
    fn build_includes_format_and_hints() {
        let pack = build_context_pack(BuildContextPackInput {
            git_anchor: sample_anchor(),
            meaning_delta_draft: Some(MeaningDeltaDraft {
                observation: json!({
                    "intended_change": "M5 context export test",
                    "preserved": ["intent"]
                }),
                source_context: json!({}),
            }),
            policy_ref: Some(
                "https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/31_m5_agent_run_integration_spec_draft.md"
                    .into(),
            ),
        });
        assert_eq!(pack.format, CONTEXT_PACK_FORMAT);
        assert!(validate_context_pack(&pack).is_ok());
        assert_eq!(pack.rde_hints.hints.len(), 1);
        assert!(pack
            .rde_hints
            .unknown_keys
            .contains(&"intended_change".to_string()));
    }

    #[test]
    fn validate_rejects_wrong_format() {
        let pack = ContextPack {
            format: "kotonoha.other.v0.1".into(),
            generated_at_unix: 0,
            git_anchor: sample_anchor(),
            meaning_delta_draft: None,
            rde_hints: ObservationRdeHints::default(),
            policy_ref: None,
        };
        assert!(matches!(
            validate_context_pack(&pack),
            Err(ContextPackError::WrongFormat { .. })
        ));
    }

    #[test]
    fn roundtrip_sample_json() {
        let pack = build_context_pack(BuildContextPackInput {
            git_anchor: sample_anchor(),
            meaning_delta_draft: None,
            policy_ref: None,
        });
        let s = serde_json::to_string_pretty(&pack).expect("serialize");
        let parsed = parse_context_pack_json(&s).expect("parse");
        assert_eq!(parsed.format, pack.format);
        assert_eq!(parsed.git_anchor, pack.git_anchor);
    }
}
