//! RDE EvidenceBinder — Phase E of the full RDE implementation roadmap.
//!
//! This module provides minimal structs for binding evidence references,
//! source spans, and uncertainty notes to RDE classification items.
//! It does **not** classify, approve, or reject. It provides structural
//! linkage so a human reviewer can trace why a classification was made.
//!
//! # Non-goals
//!
//! - No classification logic.
//! - No approval/rejection verdicts.
//! - No safety verdicts.
//! - No source context deep analysis.
//! - No report generation.
//!
//! # Human authority boundary
//!
//! EvidenceBinder connects classifications to their sources. It does not
//! decide. The resulting `EvidenceBindingReport` is review assistance,
//! not final judgment.
//!
//! # Pipeline position
//!
//! ```text
//! Phase D: DeltaMReport → RdeEvaluation / SLS-4
//! Phase E: RdeEvaluation × Source Context → EvidenceBindingReport  ← this module
//! Human Review: EvidenceBindingReport + MetaRdeReport → decision
//! ```

use crate::rde_impl::{RdeCategory, RdeEvaluation};

// ---------------------------------------------------------------------------
// EvidenceRef, EvidenceRole, TextSpan
// ---------------------------------------------------------------------------

/// A reference to evidence in source material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRef {
    /// Identifier of the source (e.g., "source_text", "source_intent", "must_not_lose[0]").
    pub source_id: String,
    /// Optional text span within the source.
    pub span: Option<TextSpan>,
    /// Optional verbatim quote from the source.
    pub quote: Option<String>,
    /// Role this evidence plays relative to the classification.
    pub role: EvidenceRole,
}

/// Role that evidence plays relative to a classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// A text span in source material (byte offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSpan {
    /// Byte offset from the start of the source text.
    pub start: usize,
    /// Byte offset to the end of the span (exclusive).
    pub end: usize,
}

// ---------------------------------------------------------------------------
// EvidenceBinding, EvidenceBindingReport
// ---------------------------------------------------------------------------

/// A single evidence binding connecting a classification item to its sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceBinding {
    /// Identifier derived from the RDE observation index.
    pub item_id: String,
    /// The classification this evidence supports or contextualizes.
    pub classification: RdeCategory,
    /// Evidence references associated with this classification.
    pub evidence_refs: Vec<EvidenceRef>,
    /// Uncertainty notes — distinct from evidence.
    pub uncertainty_notes: Vec<String>,
    /// Human reviewer focus items.
    pub reviewer_focus: Vec<String>,
}

/// Report produced by the EvidenceBinder.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EvidenceBindingReport {
    /// Evidence bindings, one per RDE classification item.
    pub bindings: Vec<EvidenceBinding>,
    /// Aggregate reviewer focus items.
    pub reviewer_focus: Vec<String>,
    /// Aggregate uncertainty notes.
    pub uncertainty_notes: Vec<String>,
}

// ---------------------------------------------------------------------------
// bind_evidence_minimal
// ---------------------------------------------------------------------------

/// Minimal evidence binding that preserves classification without changing it.
///
/// This function:
/// - Copies each `RdeObservation` into an `EvidenceBinding`.
/// - Preserves the `RdeCategory` without reclassifying.
/// - Converts `evidence_refs` strings into `EvidenceRef` entries with
///   `ContextOnly` role when present.
/// - Marks missing evidence with `IndicatesMissingEvidence` and uncertainty notes.
/// - Does **not** generate approval, rejection, or safety verdicts.
pub fn bind_evidence_minimal(evaluation: &RdeEvaluation) -> EvidenceBindingReport {
    let mut report = EvidenceBindingReport::default();

    for (i, obs) in evaluation.observations.iter().enumerate() {
        let item_id = format!("obs/{}", i);
        let mut evidence_refs: Vec<EvidenceRef> = Vec::new();
        let mut uncertainty_notes: Vec<String> = Vec::new();

        if obs.evidence_refs.is_empty() {
            evidence_refs.push(EvidenceRef {
                source_id: "unknown".to_string(),
                span: None,
                quote: None,
                role: EvidenceRole::IndicatesMissingEvidence,
            });
            uncertainty_notes.push(
                "no evidence reference was attached to this classification; treat as review focus"
                    .to_string(),
            );
        } else {
            for eref in &obs.evidence_refs {
                evidence_refs.push(EvidenceRef {
                    source_id: eref.clone(),
                    span: None,
                    quote: None,
                    role: EvidenceRole::ContextOnly,
                });
            }
        }

        if let Some(note) = &obs.confidence_note {
            uncertainty_notes.push(note.clone());
        }

        report.bindings.push(EvidenceBinding {
            item_id,
            classification: obs.category,
            evidence_refs,
            uncertainty_notes,
            reviewer_focus: Vec::new(),
        });
    }

    // If no bindings were produced, add a single note.
    if report.bindings.is_empty() {
        report.uncertainty_notes.push(
            "no RDE classification items were present; evidence binding is minimal".to_string(),
        );
        report.reviewer_focus.push(
            "human reviewer should confirm whether the empty evaluation is intentional".to_string(),
        );
    }

    report
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rde_impl::{RdeCategory, RdeEvaluation, RdeObservation};

    // ── helpers ──────────────────────────────────────────────────────────

    fn make_eval(observations: Vec<RdeObservation>) -> RdeEvaluation {
        let mut eval = RdeEvaluation::new("s/1");
        for obs in observations {
            eval.push(obs);
        }
        eval
    }

    fn obs_with_evidence(cat: RdeCategory, summary: &str, evidence: Vec<&str>) -> RdeObservation {
        RdeObservation {
            category: cat,
            summary: summary.to_string(),
            evidence_refs: evidence.into_iter().map(|s| s.to_string()).collect(),
            confidence_note: None,
        }
    }

    fn obs_without_evidence(cat: RdeCategory, summary: &str) -> RdeObservation {
        RdeObservation {
            category: cat,
            summary: summary.to_string(),
            evidence_refs: vec![],
            confidence_note: None,
        }
    }

    // ── Classification is preserved ──────────────────────────────────────

    #[test]
    fn evidence_binder_does_not_change_classification() {
        let eval = make_eval(vec![obs_with_evidence(
            RdeCategory::Preserved,
            "kept",
            vec!["ref:1"],
        )]);
        let report = bind_evidence_minimal(&eval);
        assert_eq!(report.bindings.len(), 1);
        assert_eq!(
            report.bindings[0].classification,
            RdeCategory::Preserved,
            "classification must be preserved"
        );
    }

    // ── Missing evidence → uncertainty ───────────────────────────────────

    #[test]
    fn evidence_binder_marks_missing_evidence_as_uncertainty() {
        let eval = make_eval(vec![obs_without_evidence(
            RdeCategory::Transformed,
            "changed",
        )]);
        let report = bind_evidence_minimal(&eval);
        assert_eq!(report.bindings.len(), 1);

        let binding = &report.bindings[0];
        assert!(
            binding
                .evidence_refs
                .iter()
                .any(|r| matches!(r.role, EvidenceRole::IndicatesMissingEvidence)),
            "missing evidence must be marked"
        );
        assert!(
            binding
                .uncertainty_notes
                .iter()
                .any(|n| n.contains("no evidence")),
            "uncertainty note must mention missing evidence"
        );
    }

    // ── Reviewer focus is preserved ──────────────────────────────────────

    #[test]
    fn evidence_binder_preserves_review_focus() {
        let eval = make_eval(vec![
            obs_with_evidence(RdeCategory::Preserved, "kept", vec![]),
            obs_with_evidence(
                RdeCategory::NextUpdatePolicy,
                "requires human review",
                vec![],
            ),
        ]);
        let report = bind_evidence_minimal(&eval);
        assert_eq!(report.bindings.len(), 2);

        // NextUpdatePolicy items should retain their classification
        let nup: Vec<_> = report
            .bindings
            .iter()
            .filter(|b| b.classification == RdeCategory::NextUpdatePolicy)
            .collect();
        assert!(!nup.is_empty(), "next_update_policy must not be lost");
    }

    // ── No approval/rejection/safety verdicts ────────────────────────────

    #[test]
    fn evidence_binder_does_not_generate_approval_or_rejection() {
        let eval = make_eval(vec![
            obs_with_evidence(RdeCategory::Preserved, "kept", vec!["ref:1"]),
            obs_with_evidence(RdeCategory::NextUpdatePolicy, "review needed", vec![]),
        ]);
        let report = bind_evidence_minimal(&eval);

        let all_text: String = report
            .bindings
            .iter()
            .flat_map(|b| {
                b.uncertainty_notes
                    .iter()
                    .chain(b.reviewer_focus.iter())
                    .cloned()
            })
            .chain(report.uncertainty_notes.iter().cloned())
            .chain(report.reviewer_focus.iter().cloned())
            .fold(String::new(), |mut acc, t| {
                if !acc.is_empty() {
                    acc.push(' ');
                }
                acc.push_str(&t);
                acc
            });

        let forbidden = [
            "approved",
            "rejected",
            "safe",
            "unsafe",
            "access granted",
            "access denied",
        ];
        for word in forbidden {
            assert!(
                !all_text.to_lowercase().contains(word),
                "evidence binder must not contain verdict word {word:?}"
            );
        }
    }

    // ── Empty evaluation handled ─────────────────────────────────────────

    #[test]
    fn evidence_binder_handles_empty_evaluation() {
        let eval = make_eval(vec![]);
        let report = bind_evidence_minimal(&eval);
        assert!(report.bindings.is_empty());
        assert!(!report.reviewer_focus.is_empty());
    }
}
