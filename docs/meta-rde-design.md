# Meta-RDE — Design Gate Phase 1

Status: **non-normative design guidance** — implementation not started.

This document defines the design boundary for the Meta-RDE layer. Meta-RDE inspects the RDE pipeline output itself for drift from the original RDE principles. It does **not** reclassify, approve, or reject. It is a recursive audit layer, not a superior judge.

## Purpose

Meta-RDE examines the full RDE pipeline output — from `RdeContextBundle` through `SemanticExtraction`, `DeltaMReport`, `RdeEvaluation`, and `EvidenceBindingReport` — to detect whether the pipeline has drifted from its design boundaries.

It checks for:
- RDE classifier expanding into a judgment layer.
- EvidenceBinder silently reclassifying or fabricating evidence.
- Report output presenting unverified claims as established fact.
- Missing or suppressed uncertainty notes.
- Human review boundary being eroded.

Meta-RDE output is review assistance, not final judgment.

## Pipeline position

```text
Phase B: RdeContextBundle → SemanticExtraction
Phase C: SemanticExtraction × SemanticExtraction → DeltaMReport
Phase D: DeltaMReport → RdeEvaluation / SLS-4
Phase E: RdeEvaluation × Source Context → EvidenceBindingReport
Phase F: Full Pipeline Output → MetaRdeReport                                    ← this layer
Human Review: EvidenceBindingReport + MetaRdeReport → approval / rejection / revision
```

Meta-RDE is placed **after** all other pipeline stages (B through E) and **before** human review. It reads the full transformation history, not just the final output. It does not replace human review.

## Responsibilities

### What Meta-RDE does

- Inspects whether RDE pipeline output has excessively deformed the original input intent, design thought, or classification boundary.
- Assigns difference audit results to each pipeline stage output.
- Checks whether the RDE classifier has drifted into a judgment layer.
- Checks whether the EvidenceBinder silently substitutes classification, approval, or rejection.
- Checks whether report output presents unverified content as established fact.
- Extracts unresolved, drift risk, and next update policy items for human review.
- Provides a structured `MetaRdeReport` as review assistance.

### What Meta-RDE does NOT do

- Does **not** directly change RDE classifications.
- Does **not** directly change `EvidenceBindingReport`.
- Does **not** decide approval or rejection.
- Does **not** generate safety verdicts.
- Does **not** substitute for human reviewer judgment.
- Does **not** become a policy engine or safety filter.
- Does **not** redefine RDE theoretical boundaries for implementation convenience.

## Input surface

Meta-RDE reads the full pipeline transformation history:

```text
Meta-RDE Input:
- Original RdeContextBundle
- SemanticExtraction
- DeltaMReport
- RdeEvaluation
- EvidenceBindingReport
- Generated Markdown Report (when available)
- Generated JSON Report (when available)
```

Key points:
- Meta-RDE does not read a single output alone; it reads the full transformation history.
- Meta-RDE does not instantly judge "correct" or "wrong."
- Meta-RDE classifies differences as: preserved, authorized transformation, inferred extension, suspicious drift, critical distortion.
- Meta-RDE output is supplementary information for Human Review, not final judgment.

## Classification categories

### Preserved

- **Definition**: The pipeline output faithfully represents the original design intent and classification boundary.
- **Typical example**: A `Preserved` ΔM relation mapped to `preserved` RDE category with evidence intact.
- **Human review condition**: No special review required. Confirm and proceed.
- **Must not be auto-blocking**: Already verified by the pipeline; auto-detected preservation is confirmation, not a decision.
- **Future blocking consideration**: Not applicable.

### AuthorizedTransformation

- **Definition**: The pipeline output has transformed content in ways that respect the RDE design boundary.
- **Typical example**: A `Transformed` relation mapped to `transformed` with an uncertainty note when evidence is absent.
- **Human review condition**: Review the transformation's faithfulness to the original intent.
- **Must not be auto-blocking**: Transformation is expected in RDE; cataloguing it is audit, not alarm.
- **Future blocking consideration**: If a pattern of unauthorized transformations accumulates, flag for process review.

### InferredExtension

- **Definition**: The pipeline output has added content or structure not directly present in the input.
- **Typical example**: A classifier-generated `confidence_note` summarizing uncertainty from absent evidence.
- **Human review condition**: Check whether the inferred extension is reasonably grounded or overreaching.
- **Must not be auto-blocking**: Inference is part of conservative classification; auditing the inference is Meta-RDE's role, not blocking it.
- **Future blocking consideration**: If inferred extensions routinely substitute for missing evidence, flag the evidence binder or classifier.

### SuspiciousDrift

- **Definition**: The pipeline output shows signs of boundary drift that warrant review.
- **Typical example**: A `Transformed` item whose summary uses language suggesting danger or value judgment ("this change is dangerous," "this addition is valuable").
- **Human review condition**: Required. The drift may indicate a classifier or binder exceeding its boundary.
- **Must not be auto-blocking**: Drift is a review signal, not a proven violation. The human reviewer decides.
- **Future blocking consideration**: If a specific suspicion pattern is confirmed across multiple reports, the responsible pipeline stage should be corrected.

### CriticalDistortion

- **Definition**: The pipeline output contains a concrete, demonstrable violation of RDE design boundaries.
- **Typical example**: A `Removed` ΔM relation mechanically mapped to `lost` with no human review.
- **Human review condition**: **High priority**. The distortion must be confirmed or refuted by a human before the output is accepted.
- **Must not be auto-blocking**: The name "Critical" is strong, but in this phase it signals **review priority**, not automatic rejection. The human reviewer retains authority to accept, revise, or reject.
- **Future blocking consideration**: After sufficient review evidence confirms the detection pattern, `CriticalDistortion` may be elevated to a blocking condition with explicit human override.

### Unresolved

- **Definition**: Elements that the Meta-RDE layer cannot classify confidently from the available pipeline output.
- **Typical example**: A pipeline stage output that is structurally valid but semantically ambiguous.
- **Human review condition**: Human reviewer must decide the disposition.
- **Must not be auto-blocking**: Unresolved means "needs human attention," not "presumed violation."

### NextUpdatePolicy

- **Definition**: Carry-forward items for the next pipeline iteration or process update.
- **Typical example**: "Meta-RDE detection of classifier summary language drift should be expanded in the next version."
- **Human review condition**: Informational only; no immediate action required.

## Output model (implementation candidate)

The following types are design proposals, not implementation commitments. They may change during implementation.

```rust
/// Report produced by the Meta-RDE layer.
pub struct MetaRdeReport {
    /// Identifier of the target pipeline run being audited.
    pub target_id: String,
    /// Elements faithfully preserved through the pipeline.
    pub preserved: Vec<MetaRdeFinding>,
    /// Authorized transformations detected.
    pub authorized_transformations: Vec<MetaRdeFinding>,
    /// Inferred extensions detected.
    pub inferred_extensions: Vec<MetaRdeFinding>,
    /// Suspicious drifts requiring human review.
    pub suspicious_drifts: Vec<MetaRdeFinding>,
    /// Critical distortions requiring high-priority human review.
    pub critical_distortions: Vec<MetaRdeFinding>,
    /// Elements Meta-RDE cannot classify confidently.
    pub unresolved: Vec<MetaRdeFinding>,
    /// Carry-forward policy items for the next update.
    pub next_update_policy: Vec<String>,
}

/// A single Meta-RDE finding about a pipeline stage.
pub struct MetaRdeFinding {
    /// Unique identifier for this finding.
    pub finding_id: String,
    /// Which pipeline phase the finding refers to.
    pub target_phase: PipelinePhase,
    /// Concise summary of the observation.
    pub summary: String,
    /// Evidence references supporting this finding.
    pub evidence_refs: Vec<String>,
    /// Severity of the finding.
    pub severity: MetaRdeSeverity,
    /// Human reviewer focus items.
    pub reviewer_focus: Vec<String>,
}

/// Pipeline stages that Meta-RDE can inspect.
pub enum PipelinePhase {
    PhaseB,
    PhaseC,
    PhaseD,
    PhaseE,
    ReportMarkdown,
    ReportJson,
    HumanReviewHandoff,
}

/// Severity of a Meta-RDE finding.
pub enum MetaRdeSeverity {
    /// Informational only; no action required.
    Informational,
    /// Human review recommended.
    ReviewNeeded,
    /// Higher risk; human review strongly recommended.
    HighRisk,
    /// Must be reviewed; failure to review should be documented.
    Blocking,
}
```

Design note: `Blocking` severity is a review obligation signal, not an automatic rejection. A `Blocking` finding means "this must be reviewed before the pipeline output can be accepted," not "this output is rejected."

## Minimum test plan (for implementation phase)

When implementation begins, the following tests should be added. This section is test design, not implemented code.

| Test name | What it verifies |
|---|---|
| `meta_rde_detects_classifier_boundary_drift` | When a classifier output contains judgment language ("dangerous," "valuable"), Meta-RDE flags it as `SuspiciousDrift`. |
| `meta_rde_detects_evidence_binder_judgment_drift` | When evidence binder output silently substitutes classification labels, Meta-RDE detects the substitution. |
| `meta_rde_marks_unverified_claims_as_suspicious_drift` | When report output presents unverified content as confirmed fact (e.g., "this is a loss"), Meta-RDE marks it for review. |
| `meta_rde_preserves_human_review_boundary` | Meta-RDE output does not contain approval/rejection verdicts or override human review decisions. |
| `meta_rde_does_not_generate_approval_or_rejection` | No "approved," "rejected," "safe," or "unsafe" appears in any Meta-RDE output field. |
| `meta_rde_reports_critical_distortion_without_auto_rejection` | A `CriticalDistortion` finding is generated with severity `Blocking` but does not automatically reject the pipeline output. |
| `meta_rde_handles_empty_or_partial_pipeline_outputs_conservatively` | When pipeline stages produce empty or partial output, Meta-RDE marks them as `Unresolved` rather than fabricating findings. |

## Non-goals for Phase 1

- Rust struct implementation.
- Meta-RDE classification logic.
- PostgreSQL persistence.
- CLI / UI / MCP / Orchestrator changes.
- Auto-blocking or auto-rejection logic.

## Next phases

- **Phase 2**: Implement `MetaRdeReport`, `MetaRdeFinding`, `MetaRdeSeverity` structs and minimum unit tests.
- **Phase 3**: Connect to Markdown / JSON report inspection.
- **Phase 4**: Combined review report with EvidenceBinder + Meta-RDE.

## RDE Difference Review

### Preserved

- Phase B/C/D/E/Human Review pipeline boundaries are maintained.
- The constraint that RDE classifier must not generate judgment is preserved.
- The constraint that EvidenceBinder must not change classification is preserved.
- The structure in which Human Review bears final judgment is preserved.

### Authorized Transformation

- Phase F is added to audit the full pipeline output.
- A recursive audit layer is introduced that applies RDE principles to RDE output itself.
- Suspicious drift and critical distortion are structured as human review assistance.

### Inferred Extension

- Unverified claims and over-assertion in report output are included in the audit surface.
- `EvidenceBindingReport` and `MetaRdeReport` are positioned as parallel inputs to Human Review.
- Meta-RDE severity serves as review priority, not as rejection signal.

### Unresolved

- At what stage Meta-RDE severity can be elevated to a blocking condition.
- Whether `CriticalDistortion` should become an automatic stop condition or remain a mandatory human review condition.
- Whether Meta-RDE treats Markdown or JSON report as the authoritative form.
- How PostgreSQL backends persist `MetaRdeReport`.
- Whether Meta-RDE output itself needs further Meta-RDE audit (infinite regress).

### Drift Risk

- Meta-RDE expanding into a hidden policy engine.
- `CriticalDistortion` being treated as de facto auto-rejection.
- Meta-RDE hollowing out Human Review.
- RDE theoretical boundaries being retroactively rewritten for implementation convenience.
- Meta-RDE silently overriding RDE classifier results.

### Next Update Policy

- Phase 1: Design document only.
- Phase 2: Minimum structs and unit tests.
- Phase 3: Markdown / JSON report connectivity.
- Phase 4: Combined review report with EvidenceBinder.
