//! Human Review Decision Model — Phase H implementation of the full RDE pipeline.
//!
//! This module provides minimal structs for recording human reviewer decisions
//! about Combined Review Reports. It is the first layer in the RDE pipeline
//! that may express `approve`, `reject`, `request revision`, or `defer`.
//!
//! # Non-goals
//!
//! - No automatic decision engine.
//! - No modification of RDE pipeline outputs.
//! - No safety verdict generation.
//! - No policy enforcement.
//! - No PostgreSQL persistence.
//! - No UI integration.
//!
//! # Human authority boundary
//!
//! HumanReviewDecision is an external audit record, not an internal pipeline
//! component. The decision belongs to the human reviewer. The decision record
//! references a Combined Review Report by `report_id` but does not modify it.
//!
//! # Pipeline position
//!
//! ```text
//! Phase G: EvidenceBindingReport + MetaRdeReport → Combined Review Report
//! Phase H: Combined Review Report + Reviewer → HumanReviewDecision  ← this module
//! ```

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that may occur when constructing a `HumanReviewDecision`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HumanReviewDecisionError {
    /// `decision_id` must be non-empty.
    MissingDecisionId,
    /// `report_id` must be non-empty.
    MissingReportId,
    /// `reviewer_id` must be non-empty.
    MissingReviewerId,
    /// `Approve` or `Reject` requires at least one reason.
    MissingReasonForFinalDecision,
    /// Each reason must have a non-empty `summary`.
    EmptyReasonSummary,
}

impl std::fmt::Display for HumanReviewDecisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDecisionId => write!(f, "decision_id must be non-empty"),
            Self::MissingReportId => write!(f, "report_id must be non-empty"),
            Self::MissingReviewerId => write!(f, "reviewer_id must be non-empty"),
            Self::MissingReasonForFinalDecision => {
                write!(f, "Approve or Reject requires at least one reason")
            }
            Self::EmptyReasonSummary => write!(f, "each reason must have a non-empty summary"),
        }
    }
}

impl std::error::Error for HumanReviewDecisionError {}

// ---------------------------------------------------------------------------
// Decision kind
// ---------------------------------------------------------------------------

/// Kinds of decisions a human reviewer may make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanReviewDecisionKind {
    /// Acceptable in the current context; not a claim of absolute correctness.
    Approve,
    /// Unacceptable in the current context; not a permanent invalidation.
    Reject,
    /// Partially usable but requires correction or re-generation.
    RequestRevision,
    /// Evidence or authority insufficient; judgment is postponed.
    Defer,
}

// ---------------------------------------------------------------------------
// Reason
// ---------------------------------------------------------------------------

/// A reason for a human review decision, referencing evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanReviewReason {
    /// Concise summary of the reason.
    pub summary: String,
    /// IDs of referenced evidence bindings.
    pub referenced_evidence_ids: Vec<String>,
    /// IDs of referenced Meta-RDE findings.
    pub referenced_meta_finding_ids: Vec<String>,
    /// Optional additional note.
    pub note: Option<String>,
}

impl HumanReviewReason {
    /// Creates a reason with a required non-empty summary.
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            referenced_evidence_ids: Vec::new(),
            referenced_meta_finding_ids: Vec::new(),
            note: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

/// A required follow-up action from a human review decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanReviewAction {
    /// Unique identifier for this action.
    pub action_id: String,
    /// The kind of action.
    pub action_kind: HumanReviewActionKind,
    /// Concise description.
    pub summary: String,
}

impl HumanReviewAction {
    /// Creates an action with a required non-empty id and kind.
    pub fn new(
        action_id: impl Into<String>,
        action_kind: HumanReviewActionKind,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            action_id: action_id.into(),
            action_kind,
            summary: summary.into(),
        }
    }
}

/// Kinds of follow-up actions from a human review decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanReviewActionKind {
    AddEvidence,
    ReviseOutput,
    ReRunPipeline,
    EscalateReview,
    NoActionRequired,
}

// ---------------------------------------------------------------------------
// Decision
// ---------------------------------------------------------------------------

/// A human reviewer's decision about an RDE pipeline output.
///
/// This struct is an external audit record. It does **not** modify the
/// Combined Review Report or any upstream RDE pipeline output. The decision
/// references the report by `report_id` only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanReviewDecision {
    /// Unique identifier for this decision record.
    pub decision_id: String,
    /// Reference to the Combined Review Report this decision addresses.
    pub report_id: String,
    /// Identifier of the reviewer making the decision.
    pub reviewer_id: String,
    /// The decision kind.
    pub decision: HumanReviewDecisionKind,
    /// Reasons for the decision, referencing evidence.
    pub reasons: Vec<HumanReviewReason>,
    /// Required follow-up actions.
    pub required_actions: Vec<HumanReviewAction>,
    /// When the decision was made (deferred: no chrono dependency).
    pub created_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Constructor with validation
// ---------------------------------------------------------------------------

/// Creates a validated `HumanReviewDecision`.
///
/// Validation rules:
/// - `decision_id`, `report_id`, `reviewer_id` must be non-empty.
/// - `Approve` / `Reject` requires at least one reason.
/// - Every reason must have a non-empty `summary`.
/// - `RequestRevision` / `Defer` may have empty reasons but actions or notes are recommended.
pub fn create_human_review_decision(
    decision_id: impl Into<String>,
    report_id: impl Into<String>,
    reviewer_id: impl Into<String>,
    decision: HumanReviewDecisionKind,
    reasons: Vec<HumanReviewReason>,
    required_actions: Vec<HumanReviewAction>,
    created_at: Option<String>,
) -> Result<HumanReviewDecision, HumanReviewDecisionError> {
    let decision_id = decision_id.into();
    let report_id = report_id.into();
    let reviewer_id = reviewer_id.into();

    if decision_id.trim().is_empty() {
        return Err(HumanReviewDecisionError::MissingDecisionId);
    }
    if report_id.trim().is_empty() {
        return Err(HumanReviewDecisionError::MissingReportId);
    }
    if reviewer_id.trim().is_empty() {
        return Err(HumanReviewDecisionError::MissingReviewerId);
    }

    match decision {
        HumanReviewDecisionKind::Approve | HumanReviewDecisionKind::Reject => {
            if reasons.is_empty() {
                return Err(HumanReviewDecisionError::MissingReasonForFinalDecision);
            }
        }
        HumanReviewDecisionKind::RequestRevision | HumanReviewDecisionKind::Defer => {
            // reasons may be empty; actions or notes are recommended but not required
        }
    }

    for reason in &reasons {
        if reason.summary.trim().is_empty() {
            return Err(HumanReviewDecisionError::EmptyReasonSummary);
        }
    }

    Ok(HumanReviewDecision {
        decision_id,
        report_id,
        reviewer_id,
        decision,
        reasons,
        required_actions,
        created_at,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────

    fn reason(s: &str) -> HumanReviewReason {
        HumanReviewReason::new(s)
    }

    fn action(id: &str, kind: HumanReviewActionKind, summary: &str) -> HumanReviewAction {
        HumanReviewAction::new(id, kind, summary)
    }

    // ── Approval with reason ─────────────────────────────────────────────

    #[test]
    fn can_create_approve_decision_with_reason() {
        let d = create_human_review_decision(
            "d/1",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::Approve,
            vec![reason("evidence supports preservation and transformation")],
            vec![],
            None,
        );
        assert!(d.is_ok());
        let d = d.unwrap();
        assert_eq!(d.decision, HumanReviewDecisionKind::Approve);
        assert_eq!(d.report_id, "report/1");
        assert_eq!(d.reviewer_id, "reviewer/alice");
    }

    // ── Reject also requires reason ──────────────────────────────────────

    #[test]
    fn can_create_reject_decision_with_reason() {
        let d = create_human_review_decision(
            "d/2",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::Reject,
            vec![reason("critical distortion confirmed; output unacceptable")],
            vec![],
            None,
        );
        assert!(d.is_ok());
        assert_eq!(d.unwrap().decision, HumanReviewDecisionKind::Reject);
    }

    // ── Approve without reason is rejected ───────────────────────────────

    #[test]
    fn human_review_decision_requires_reason_for_approve_or_reject() {
        let r = create_human_review_decision(
            "d/3",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::Approve,
            vec![],
            vec![],
            None,
        );
        assert!(r.is_err());
        assert_eq!(
            r.unwrap_err(),
            HumanReviewDecisionError::MissingReasonForFinalDecision
        );

        // same for Reject
        let r = create_human_review_decision(
            "d/4",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::Reject,
            vec![],
            vec![],
            None,
        );
        assert!(r.is_err());
    }

    // ── RequestRevision is separate from Reject ──────────────────────────

    #[test]
    fn human_review_decision_allows_request_revision_without_rejection() {
        let d = create_human_review_decision(
            "d/5",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::RequestRevision,
            vec![],
            vec![action(
                "a/1",
                HumanReviewActionKind::ReviseOutput,
                "re-run with additional must_not_lose context",
            )],
            None,
        );
        assert!(d.is_ok());
        let d = d.unwrap();
        assert_eq!(d.decision, HumanReviewDecisionKind::RequestRevision);
        // should NOT be Reject
        assert_ne!(d.decision, HumanReviewDecisionKind::Reject);
    }

    // ── Defer for missing evidence ───────────────────────────────────────

    #[test]
    fn human_review_decision_allows_defer_for_missing_evidence() {
        let d = create_human_review_decision(
            "d/6",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::Defer,
            vec![reason("missing source context for must_not_lose items")],
            vec![action(
                "a/2",
                HumanReviewActionKind::AddEvidence,
                "provide original design spec",
            )],
            None,
        );
        assert!(d.is_ok());
        let d = d.unwrap();
        assert_eq!(d.decision, HumanReviewDecisionKind::Defer);
    }

    // ── Reviewer accountability ──────────────────────────────────────────

    #[test]
    fn human_review_decision_preserves_reviewer_accountability() {
        // empty reviewer_id is rejected
        let r = create_human_review_decision(
            "d/7",
            "report/1",
            "   ",
            HumanReviewDecisionKind::Defer,
            vec![],
            vec![],
            None,
        );
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), HumanReviewDecisionError::MissingReviewerId);
    }

    // ── Decision does not modify RDE outputs ─────────────────────────────

    #[test]
    fn human_review_decision_does_not_modify_rde_outputs() {
        // The decision record has no access to RDE pipeline objects.
        // It references a report by report_id only.
        let d = create_human_review_decision(
            "d/8",
            "report/2",
            "reviewer/bob",
            HumanReviewDecisionKind::Approve,
            vec![reason("looks good")],
            vec![],
            None,
        )
        .unwrap();

        // The decision references the report, does not embed or modify it
        assert_eq!(d.report_id, "report/2");
        // No mutation of any RDE object is possible through this struct
    }

    // ── Required actions ─────────────────────────────────────────────────

    #[test]
    fn human_review_decision_can_record_required_actions() {
        let actions = vec![
            action("a/1", HumanReviewActionKind::AddEvidence, "add context"),
            action(
                "a/2",
                HumanReviewActionKind::ReRunPipeline,
                "re-run after fixing must_not_lose",
            ),
            action(
                "a/3",
                HumanReviewActionKind::NoActionRequired,
                "no further action",
            ),
        ];
        let d = create_human_review_decision(
            "d/9",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::RequestRevision,
            vec![],
            actions,
            None,
        )
        .unwrap();

        assert_eq!(d.required_actions.len(), 3);
        assert_eq!(
            d.required_actions[0].action_kind,
            HumanReviewActionKind::AddEvidence
        );
        assert_eq!(
            d.required_actions[1].action_kind,
            HumanReviewActionKind::ReRunPipeline
        );
        assert_eq!(
            d.required_actions[2].action_kind,
            HumanReviewActionKind::NoActionRequired
        );
    }

    // ── Empty decision_id / report_id ────────────────────────────────────

    #[test]
    fn rejects_empty_decision_id() {
        let r = create_human_review_decision(
            "",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::Defer,
            vec![],
            vec![],
            None,
        );
        assert!(matches!(
            r.unwrap_err(),
            HumanReviewDecisionError::MissingDecisionId
        ));
    }

    #[test]
    fn rejects_empty_report_id() {
        let r = create_human_review_decision(
            "d/10",
            "   ",
            "reviewer/alice",
            HumanReviewDecisionKind::Defer,
            vec![],
            vec![],
            None,
        );
        assert!(matches!(
            r.unwrap_err(),
            HumanReviewDecisionError::MissingReportId
        ));
    }

    // ── Empty reason summary ─────────────────────────────────────────────

    #[test]
    fn rejects_empty_reason_summary() {
        let r = create_human_review_decision(
            "d/11",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::Approve,
            vec![HumanReviewReason {
                summary: "  ".to_string(),
                referenced_evidence_ids: vec![],
                referenced_meta_finding_ids: vec![],
                note: None,
            }],
            vec![],
            None,
        );
        assert!(matches!(
            r.unwrap_err(),
            HumanReviewDecisionError::EmptyReasonSummary
        ));
    }

    // ── created_at preserved ─────────────────────────────────────────────

    #[test]
    fn preserves_created_at() {
        let d = create_human_review_decision(
            "d/12",
            "report/1",
            "reviewer/alice",
            HumanReviewDecisionKind::Approve,
            vec![reason("ok")],
            vec![],
            Some("2026-06-07T01:00:00Z".to_string()),
        )
        .unwrap();
        assert_eq!(d.created_at.as_deref(), Some("2026-06-07T01:00:00Z"));
    }

    // ── Error Display ────────────────────────────────────────────────────

    #[test]
    fn error_display_is_human_readable() {
        let e = HumanReviewDecisionError::MissingReasonForFinalDecision;
        assert!(e.to_string().contains("reason"));

        let e = HumanReviewDecisionError::MissingReviewerId;
        assert!(e.to_string().contains("reviewer_id"));
    }
}
