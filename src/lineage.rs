//! Minimal **lineage unit** model ([`semantic-lineage-model.md`] in `kotonoha-spec`).
//!
//! [`semantic-lineage-model.md`]: https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/semantic-lineage-model.md

use serde::{Deserialize, Serialize};

/// Smallest addressable semantic lineage record required by Phase 1 (`kotonoha-spec`).
///
/// serde **`deny_unknown_fields`**: interchange JSON must use only **`id`** / **`prior_unit_id`** on this object (`kotonoha_core::interchange`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageUnit {
    /// Unique within deployment scope; URI/IRI recommended (`kotonoha-spec`).
    pub id: String,
    /// Optional link to a prior unit (`prior_unit_id` pattern).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prior_unit_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineageValidationError {
    EmptyId,
}

impl LineageUnit {
    /// Returns `Err` if `id` is missing or whitespace-only.
    pub fn validate(&self) -> Result<(), LineageValidationError> {
        if self.id.trim().is_empty() {
            return Err(LineageValidationError::EmptyId);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_nonempty_id() {
        let u = LineageUnit {
            id: "https://example.invalid/u/1".to_string(),
            prior_unit_id: None,
        };
        assert!(u.validate().is_ok());
    }

    #[test]
    fn rejects_empty_id() {
        let u = LineageUnit {
            id: "   ".to_string(),
            prior_unit_id: None,
        };
        assert!(matches!(u.validate(), Err(LineageValidationError::EmptyId)));
    }

    #[test]
    fn rejects_json_unknown_field() {
        let j = r#"{"id":"https://example.invalid/u/2","prior_unit_id":null,"extra":1}"#;
        let r: Result<LineageUnit, _> = serde_json::from_str(j);
        assert!(r.is_err(), "expected deny_unknown_fields: {r:?}");
    }
}
