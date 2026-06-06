# Human Review Decision Model — Design Gate Phase 1

Status: **non-normative design guidance** — implementation not started.

This document defines the design boundary for the Human Review Decision Model. It is the first layer in the RDE pipeline that may express `approve`, `reject`, `request revision`, or `defer`. However, it is **not** an automatic decision engine. It records the decision made by a human reviewer after examining the Combined Review Report.

## Purpose

The Human Review Decision Model records the outcome of human review of an RDE pipeline output. It does **not** modify RDE classifications, evidence bindings, Meta-RDE findings, or the Combined Review Report. It is an external audit record, not an internal pipeline component.

## Pipeline position

```text
Phase B: RdeContextBundle → SemanticExtraction
Phase C: SemanticExtraction × SemanticExtraction → DeltaMReport
Phase D: DeltaMReport → RdeEvaluation / SLS-4
Phase E: RdeEvaluation × Source Context → EvidenceBindingReport
Phase F: EvidenceBindingReport → MetaRdeReport
Phase G: EvidenceBindingReport + MetaRdeReport → Combined Review Report
Phase H: Combined Review Report + Reviewer → HumanReviewDecision
```

Phase H is the first layer that can express `approve`, `reject`, `request revision`, or `defer`. However, the decision belongs to the human or explicitly designated reviewer actor. Phase H is **not** an automatic decision engine. It does **not** modify RDE pipeline outputs. Its decisions are recorded as separate audit trail entries.

## Responsibilities

### What Human Review Decision Model does

- Records the decision outcome of a human reviewer examining a Combined Review Report.
- Expresses `approve`, `reject`, `request revision`, or `defer` as decision kinds.
- Records decision reasons with references to specific evidence bindings and Meta-RDE findings.
- Records required follow-up actions.
- Identifies the reviewer accountable for the decision.
- Provides a decision record suitable for audit trails.

### What Human Review Decision Model does NOT do

- Does **not** directly rewrite RDE classifier output.
- Does **not** directly modify `EvidenceBindingReport`.
- Does **not** directly modify `MetaRdeReport`.
- Does **not** retroactively alter the Combined Review Report.
- Does **not** disguise human judgment as automatic determination.
- Does **not** approve or reject without recording decision reasons.
- Does **not** substitute for a safety policy engine.

## Decision categories

### Approve

The reviewer has examined the Combined Review Report and judged the pipeline output acceptable in the current context. Approve does **not** mean the RDE output is absolutely correct. It means the reviewer, given current evidence and context, accepts the result.

### Reject

The reviewer has examined the Combined Review Report and judged it unacceptable in the current context. Reject does **not** permanently invalidate the original input or candidate. Re-generation, revision, or re-review with additional evidence remains possible.

### RequestRevision

The output is partially usable but requires correction, supplementation, or re-generation. Useful when preserved, authorized transformation, and suspicious drift findings are mixed.

### Defer

The evidence, context, or reviewer authority needed for a decision is insufficient. Judgment is postponed. Useful when missing evidence or unresolved findings are significant.

## Output model (implementation candidate)

The following types are design proposals, not implementation commitments.

```rust
/// A human reviewer's decision about an RDE pipeline output.
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

/// Kinds of decisions a human reviewer may make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanReviewDecisionKind {
    Approve,
    Reject,
    RequestRevision,
    Defer,
}

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

/// Kinds of follow-up actions from a human review decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanReviewActionKind {
    AddEvidence,
    ReviseOutput,
    ReRunPipeline,
    EscalateReview,
    NoActionRequired,
}
```

Design notes:
- `HumanReviewDecision` does **not** modify the Combined Review Report.
- The decision is an external record referencing the report.
- `reviewer_id` can later connect to PoP-UID, local user identity, or audit actor.
- `created_at` uses `Option<String>` to avoid a chrono dependency at this stage.

## Minimum test plan (for implementation phase)

| Test name | What it verifies |
|---|---|
| `human_review_decision_references_combined_report_without_modifying_it` | The decision record holds a `report_id` reference but does not alter the `EvidenceBindingReport` or `MetaRdeReport` contained in the referenced report. |
| `human_review_decision_requires_reason_for_approve_or_reject` | An `Approve` or `Reject` decision without at least one `HumanReviewReason` should produce a validation warning. |
| `human_review_decision_allows_request_revision_without_rejection` | `RequestRevision` is expressed as its own decision kind, not as a variant of `Reject`. |
| `human_review_decision_allows_defer_for_missing_evidence` | A `Defer` decision can reference missing evidence or unresolved findings as reasons. |
| `human_review_decision_preserves_reviewer_accountability` | The `reviewer_id` field is required and must be traceable. |
| `human_review_decision_does_not_modify_rde_outputs` | Constructing a decision record does not mutate any RDE, evidence, Meta-RDE, or report object. |
| `human_review_decision_can_record_required_actions` | `required_actions` can hold multiple follow-up action items. |

## Non-goals for Phase 1

- Rust struct implementation.
- PostgreSQL persistence.
- CLI / UI / MCP / Orchestrator changes.
- Automatic approval or rejection logic.
- Policy engine or safety filter integration.

## Next phases

- **Phase 2**: Implement minimum structs and unit tests.
- **Phase 3**: PostgreSQL persistence.
- **Phase 4**: UI integration for decision recording.

## RDE Difference Review

### Preserved

- Phase B/C/D/E/F/G non-judgment boundaries are maintained.
- RDE classifier must not generate approval/rejection is preserved.
- EvidenceBinder must not change classification is preserved.
- Meta-RDE must not substitute for Human Review is preserved.
- Combined Review Report is not a decision object is preserved.

### Authorized Transformation

- Human Review Decision Model is added after the Combined Review Report.
- A layer that can express `approve` / `reject` / `request revision` / `defer` is introduced for the first time.
- The decision is treated as an external audit record, not a modification of RDE output.

### Inferred Extension

- Reviewer accountability is included in the decision model.
- Decision reasons and evidence / meta finding reference relationships are introduced.
- `request revision` / `defer` avoids a simplistic binary approve/reject.
- Future connection to PoP-UID / audit actor / PostgreSQL persistence is left open.

### Unresolved

- Which actor identity model `reviewer_id` connects to.
- How strictly decision reason requirements should be enforced.
- Whether minimum evidence conditions should be defined for `Approve` / `Reject`.
- How `HumanReviewDecision` should be persisted in PostgreSQL.
- How to visually separate decision records from report artifacts in a UI.

### Drift Risk

- Human Review Decision Model appearing as an automatic judgment engine.
- `Approve` being misinterpreted as proof of absolute correctness.
- `Reject` being treated as permanent invalidation of the entire candidate.
- `RequestRevision` becoming an ambiguous failure state.
- `Defer` being abused to avoid accountability.
- Decision being treated as a retroactive modification of RDE output.

### Next Update Policy

- Phase 1: Design document only.
- Next phase: Minimum structs and unit tests for `HumanReviewDecision`.
- PostgreSQL persistence deferred until `HumanReviewDecision` type boundaries stabilize.
- UI deferred until separation of report artifact and decision record is fixed.
