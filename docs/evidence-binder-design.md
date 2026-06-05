# EvidenceBinder — Design Gate Phase 1

Status: **non-normative design guidance** — implementation not started.

This document defines the design boundary for the EvidenceBinder layer of the RDE pipeline. EvidenceBinder connects RDE classifications to their source evidence without replacing human judgment.

## Purpose

EvidenceBinder is the layer that binds evidence references, source spans, and uncertainty notes to RDE classification items. It does **not** classify. It does **not** approve or reject. It provides the structural linkage so that a human reviewer can trace why a classification was made.

## Pipeline position

```text
Phase B: RdeContextBundle → SemanticExtraction
Phase C: SemanticExtraction × SemanticExtraction → DeltaMReport
Phase D: DeltaMReport → RdeEvaluation / SLS-4
Phase E: RdeEvaluation × Source Context → EvidenceBindingReport   ← this layer
Human Review: EvidenceBindingReport → approval / rejection / revision
```

EvidenceBinder is placed **after** classification (Phase D) and **before** human review. It must not be placed before Phase D because it binds evidence to already-made classifications. It must not replace human review because it does not decide.

## Responsibilities

### What EvidenceBinder does

- Binds each RDE classification item to its evidence spans, source references, and context.
- Tracks which input material (`source_text`, `changed_text`, `source_intent`, `non_goals`, etc.) supports which classification.
- Separates confidence/uncertainty notes from evidence references.
- Provides a structure that human reviewers can inspect to understand "why this classification."
- Defines an intermediate structure (`EvidenceBindingReport`) that can feed into audit trails, Markdown reports, and JSON reports.

### What EvidenceBinder does NOT do

- Does **not** decide approval or rejection.
- Does **not** generate safety verdicts.
- Does **not** override or change the classification itself.
- Does **not** shortcut SLS-4 category mapping.
- Does **not** fabricate evidence where none exists.
- Does **not** treat missing evidence as factual evidence.
- Does **not** replace human judgment.

## Input/output model (draft)

The following types are design proposals, not implementation commitments. They may change during implementation.

```rust
/// A single evidence binding connecting an RDE classification item to its sources.
pub struct EvidenceBinding {
    /// Identifier of the RDE classification item this binding refers to.
    pub item_id: String,
    /// The classification this evidence supports or contextualizes.
    pub classification: RdeCategory,
    /// Evidence references that support this classification.
    pub evidence_refs: Vec<EvidenceRef>,
    /// Uncertainty notes — distinct from evidence.
    pub uncertainty_notes: Vec<String>,
    /// Human reviewer focus items suggested by the evidence layer.
    pub reviewer_focus: Vec<String>,
}

/// A single evidence reference pointing into source material.
pub struct EvidenceRef {
    /// Identifier of the source (e.g., "source_text", "source_intent", "must_not_lose[0]").
    pub source_id: String,
    /// Optional text span within the source.
    pub span: Option<TextSpan>,
    /// Optional verbatim quote from the source.
    pub quote: Option<String>,
    /// Role this evidence plays in the classification.
    pub role: EvidenceRole,
}

/// Roles that evidence can play relative to a classification.
pub enum EvidenceRole {
    /// Evidence directly supports the classification.
    SupportsClassification,
    /// Evidence supports a claim of uncertainty about the classification.
    SupportsUncertainty,
    /// Evidence indicates that material is missing or unavailable.
    IndicatesMissingEvidence,
    /// Evidence conflicts with the classification.
    IndicatesConflict,
    /// Evidence provides context without directly supporting or opposing.
    ContextOnly,
}

/// A text span in source material.
pub struct TextSpan {
    /// Byte offset from the start of the source text.
    pub start: usize,
    /// Byte offset to the end of the span (exclusive).
    pub end: usize,
}

/// Report produced by the EvidenceBinder.
pub struct EvidenceBindingReport {
    /// Subject reference shared across the pipeline.
    pub subject_ref: String,
    /// Evidence bindings, one per RDE classification item.
    pub bindings: Vec<EvidenceBinding>,
    /// Summary of evidence coverage (e.g., how many items have evidence vs. missing).
    pub coverage_summary: String,
}
```

Design note: `TextSpan` uses byte offsets as the default. Whether to support char offsets or line-column format is deferred to implementation.

## Minimum test plan (for implementation phase)

When implementation begins, the following tests should be added. This section serves as the test design, not as implemented code.

| Test name | What it verifies |
|---|---|
| `evidence_binder_attaches_source_refs_to_rde_items` | Each RDE classification item receives its corresponding `EvidenceRef` entries from the semantic elements and ΔM relations that produced it. |
| `evidence_binder_does_not_change_classification` | The `RdeCategory` on each item is identical before and after evidence binding. The binder must not reclassify. |
| `evidence_binder_marks_missing_evidence_as_uncertainty` | When no evidence references exist for a classification item, the binder records `IndicatesMissingEvidence` and adds an uncertainty note rather than silently omitting evidence. |
| `evidence_binder_preserves_review_focus` | `NextUpdatePolicy` items that serve as human review focus retain their role and are not converted into judgment items. |
| `evidence_binder_does_not_generate_approval_or_rejection` | The output contains no "approved", "rejected", "safe", or "unsafe" strings in any field. |
| `evidence_binder_handles_conflicting_evidence_conservatively` | When evidence roles conflict (e.g., `SupportsClassification` and `IndicatesConflict` for the same item), the binder records the conflict rather than resolving it silently. |

## Non-goals for Phase 1

- Rust struct implementation.
- PostgreSQL persistence of evidence bindings.
- Markdown / JSON report generation.
- Meta-RDE integration.
- CLI / UI / MCP / Orchestrator changes.

## Next phases

- **Phase 2**: Implement `EvidenceBinding`, `EvidenceRef`, `EvidenceBindingReport` structs and minimum unit tests.
- **Phase 3**: Connect to Markdown / JSON report generation.
- **Meta-RDE design gate**: Start after EvidenceBinder responsibilities are fixed.

## RDE Difference Review

### Preserved

- Phase B/C/D/Human Review pipeline boundaries are maintained.
- The constraint that RDE classifier must not generate judgment is preserved.
- Conservative classification policy is unchanged.

### Authorized Transformation

- The pipeline-level test results for classifications are extended with an evidence binding layer.
- RDE output is transformed into a structure more suitable for human review handoff.

### Inferred Extension

- Separation of evidence span, source reference, uncertainty note, and reviewer focus is introduced.
- JSON / Markdown report connectivity is indicated in the design.

### Unresolved

- Whether `TextSpan` uses byte offsets, char offsets, or line-column format.
- Whether `quote` is stored or only `source_ref` is kept.
- How PostgreSQL backends persist evidence bindings.
- How Meta-RDE reads `EvidenceBindingReport`.

### Drift Risk

- EvidenceBinder expanding into a classifier or judger.
- Missing evidence being treated as inferred factual evidence.
- Reviewer handoff appearing as implicit approval/rejection.
- SLS-4 mapping being shortcut by the evidence layer.

### Next Update Policy

- Phase 1: Design document only.
- Phase 2: Minimum structs and unit tests.
- Phase 3: Markdown / JSON report connection.
- Meta-RDE design gate: deferred until EvidenceBinder responsibilities are stable.
