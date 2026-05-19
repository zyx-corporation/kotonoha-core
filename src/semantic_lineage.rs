//! M1 semantic lineage types (MeaningDelta, RDEAssessment, ReviewDecision).
//!
//! DDL: [`../migrations/20260520000000_m1_semantic_lineage.sql`](../migrations/20260520000000_m1_semantic_lineage.sql).
//! Informative schema notes: [`../docs/postgresql-schema-m1.md`](../docs/postgresql-schema-m1.md).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Git content-lineage anchor for a [`MeaningDeltaInput`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitAnchor {
    pub git_commit: String,
    pub file_path: String,
    pub line_range_start: Option<i32>,
    pub line_range_end: Option<i32>,
    pub diff_ref: Option<String>,
}

/// Validation errors for M1 inputs (before database I/O).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticLineageError {
    EmptyGitCommit,
    EmptyFilePath,
    MissingGitAnchorDetail,
    InvalidLineRange,
    EmptyDecidedBy,
}

impl std::fmt::Display for SemanticLineageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticLineageError::EmptyGitCommit => f.write_str("git_commit must not be empty"),
            SemanticLineageError::EmptyFilePath => f.write_str("file_path must not be empty"),
            SemanticLineageError::MissingGitAnchorDetail => {
                f.write_str("either line_range_start/end or diff_ref is required")
            }
            SemanticLineageError::InvalidLineRange => {
                f.write_str("line_range_end must be >= line_range_start")
            }
            SemanticLineageError::EmptyDecidedBy => f.write_str("decided_by must not be empty"),
        }
    }
}

impl std::error::Error for SemanticLineageError {}

impl GitAnchor {
    /// Matches DB `CHECK` on `meaning_deltas`.
    pub fn validate(&self) -> Result<(), SemanticLineageError> {
        if self.git_commit.trim().is_empty() {
            return Err(SemanticLineageError::EmptyGitCommit);
        }
        if self.file_path.trim().is_empty() {
            return Err(SemanticLineageError::EmptyFilePath);
        }
        if let (Some(start), Some(end)) = (self.line_range_start, self.line_range_end) {
            if end < start {
                return Err(SemanticLineageError::InvalidLineRange);
            }
            return Ok(());
        }
        if self
            .diff_ref
            .as_deref()
            .is_some_and(|d| !d.trim().is_empty())
        {
            return Ok(());
        }
        Err(SemanticLineageError::MissingGitAnchorDetail)
    }
}

/// Input for inserting a meaning change (ΔM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeaningDeltaInput {
    pub document_object_id: Option<Uuid>,
    pub prior_meaning_state_id: Option<Uuid>,
    pub new_meaning_state_id: Option<Uuid>,
    pub agent_run_id: Option<Uuid>,
    pub git_anchor: GitAnchor,
    #[serde(default)]
    pub observation: Value,
    #[serde(default)]
    pub source_context: Value,
}

impl MeaningDeltaInput {
    pub fn validate(&self) -> Result<(), SemanticLineageError> {
        self.git_anchor.validate()
    }
}

/// Human or institutional review outcome (not a substitute for authority).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecisionKind {
    Approve,
    Hold,
    Reject,
    NeedsRevision,
}

impl ReviewDecisionKind {
    pub fn as_db_str(self) -> &'static str {
        match self {
            ReviewDecisionKind::Approve => "approve",
            ReviewDecisionKind::Hold => "hold",
            ReviewDecisionKind::Reject => "reject",
            ReviewDecisionKind::NeedsRevision => "needs_revision",
        }
    }
}

/// Input for `record_review_decision`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordReviewDecisionInput {
    pub meaning_delta_id: Uuid,
    pub rde_assessment_id: Option<Uuid>,
    pub decision: ReviewDecisionKind,
    pub decided_by: String,
    #[serde(default)]
    pub rationale: Value,
}

impl RecordReviewDecisionInput {
    pub fn validate(&self) -> Result<(), SemanticLineageError> {
        if self.decided_by.trim().is_empty() {
            return Err(SemanticLineageError::EmptyDecidedBy);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_anchor_accepts_line_range() {
        GitAnchor {
            git_commit: "abc123".into(),
            file_path: "docs/a.md".into(),
            line_range_start: Some(1),
            line_range_end: Some(10),
            diff_ref: None,
        }
        .validate()
        .unwrap();
    }

    #[test]
    fn git_anchor_accepts_diff_ref() {
        GitAnchor {
            git_commit: "abc123".into(),
            file_path: "docs/a.md".into(),
            line_range_start: None,
            line_range_end: None,
            diff_ref: Some("staged:sha256:deadbeef".into()),
        }
        .validate()
        .unwrap();
    }

    #[test]
    fn git_anchor_rejects_missing_detail() {
        let err = GitAnchor {
            git_commit: "abc".into(),
            file_path: "f".into(),
            line_range_start: None,
            line_range_end: None,
            diff_ref: None,
        }
        .validate()
        .unwrap_err();
        assert_eq!(err, SemanticLineageError::MissingGitAnchorDetail);
    }
}
