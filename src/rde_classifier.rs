//! RDE classifier scaffold — Phase D of the full RDE implementation roadmap.
//!
//! This module introduces a conservative classifier that maps `DeltaMReport`
//! into existing `RdeEvaluation` / SLS-4-compatible categories. It does **not**
//! claim to understand meaning. The classifier emits review-focused observations
//! only; final judgment belongs to the human review layer.
//!
//! # Non-goals
//!
//! - No LLM integration.
//! - No fuzzy matching or embedding similarity.
//! - No approval/rejection verdicts.
//! - No safety verdicts.
//! - No policy enforcement.
//! - No model dependency.
//!
//! # Human authority boundary
//!
//! The classifier produces structured RDE review focus. It does **not** decide:
//!
//! - what is lost,
//! - what is dangerous,
//! - what is acceptable,
//! - what is approved or rejected.
//!
//! Those decisions are reserved for the human review layer (SLS-5.9).
//!
//! # Pipeline position
//!
//! ```text
//! Phase B:     RdeContextBundle → SemanticExtraction
//! Phase C:     SemanticExtraction × SemanticExtraction → DeltaMReport
//! Phase D:     DeltaMReport → RdeEvaluation / SLS-4 categories  ← this module
//! Human review: approval / rejection / accountability decision
//! ```

use crate::rde_delta::{DeltaMRelationKind, DeltaMReport};
use crate::rde_impl::{RdeCategory, RdeEvaluation, RdeObservation};
use crate::rde_semantic::RdeError;

// ---------------------------------------------------------------------------
// RdeClassifier trait
// ---------------------------------------------------------------------------

/// Contract for components that map `DeltaMReport` into `RdeEvaluation`.
///
/// Implementations may be rule-based, model-assisted, or human-curated. The
/// trait does **not** mandate a specific model provider or algorithm.
///
/// # Human authority boundary
///
/// Implementations **MUST NOT** emit approval, rejection, safety verdicts, or
/// access-control decisions through this trait. Classification produces review
/// focus, not final judgment.
pub trait RdeClassifier {
    /// Classifies ΔM relations into SLS-4-compatible RDE observations.
    ///
    /// Returns `RdeEvaluation` on success. The output must pass existing
    /// `crate::rde::validate_json` when called through `RdeEvaluation::validate`.
    fn classify(&self, report: &DeltaMReport) -> Result<RdeEvaluation, RdeError>;
}

// ---------------------------------------------------------------------------
// ConservativeRdeClassifier
// ---------------------------------------------------------------------------

/// A deterministic, conservative classifier that maps `DeltaMReport` relations
/// into RDE categories following the Phase D design gate constraints.
///
/// # Mapping policy (conservative)
///
/// | ΔM relation | RDE category | Notes |
/// |---|---|---|
/// | `Preserved` | `preserved` | Direct mapping. |
/// | `Transformed` | `transformed` | With uncertainty note if evidence is absent. |
/// | `Complemented` | `complemented` | No value judgment. |
/// | `Removed` | `next_update_policy` | Review focus only; **not** `lost`. |
/// | `Contradicted` | `next_update_policy` | Review focus only; **not** `deviation_risk`. |
/// | `Weakened` | `next_update_policy` | Review focus only; **not** `deviation_risk`. |
/// | `Unresolved` | `next_update_policy` | Review focus unless explicitly marked intentional. |
/// | `Split` / `Merged` | `transformed` | With uncertainty note. |
///
/// # What this classifier does **not** do
///
/// - `Removed` is **never** mapped to `lost`.
/// - `Contradicted` / `Weakened` are **never** mapped to `deviation_risk`.
/// - `Complemented` is **never** treated as valuable.
/// - Empty evidence produces a required `confidence_note`.
/// - No approval/rejection/safety verdict is emitted.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConservativeRdeClassifier;

impl RdeClassifier for ConservativeRdeClassifier {
    fn classify(&self, report: &DeltaMReport) -> Result<RdeEvaluation, RdeError> {
        // Validate minimum input
        if report.subject_ref.trim().is_empty() {
            return Err(RdeError::Validation(
                "DeltaMReport.subject_ref must be non-empty".to_string(),
            ));
        }

        let mut evaluation = RdeEvaluation::new(&report.subject_ref);

        for relation in &report.relations {
            let evidence_refs = relation.evidence_refs.clone();
            let (category, summary, confidence_note) = classify_relation(relation, &evidence_refs);

            evaluation.push(RdeObservation {
                category,
                summary,
                evidence_refs,
                confidence_note,
            });
        }

        // When the report has no relations, emit a single observation noting it.
        if report.relations.is_empty() {
            evaluation.push(RdeObservation {
                category: RdeCategory::IntentionallyUnresolved,
                summary:
                    "no DeltaM relations were present in the report; review context is minimal"
                        .to_string(),
                evidence_refs: Vec::new(),
                confidence_note: Some(
                    "empty DeltaMReport; no structured relations to classify".to_string(),
                ),
            });
        }

        Ok(evaluation)
    }
}

/// Classifies a single `DeltaMRelation` into a (category, summary, confidence_note) triple.
fn classify_relation(
    relation: &crate::rde_delta::DeltaMRelation,
    evidence_refs: &[String],
) -> (RdeCategory, String, Option<String>) {
    let no_evidence = evidence_refs.is_empty();

    match relation.relation {
        DeltaMRelationKind::Preserved => {
            let mut note = None;
            if no_evidence {
                note = Some(
                    "no evidence reference was attached to this relation; treat as review focus, not confirmed preservation"
                        .to_string(),
                );
            }
            (
                RdeCategory::Preserved,
                "semantic element preserved by DeltaM relation".to_string(),
                note,
            )
        }

        DeltaMRelationKind::Transformed => {
            let mut note = None;
            if no_evidence {
                note = Some(
                    "no evidence reference was attached to this relation; treat as review focus, not confirmed transformation"
                        .to_string(),
                );
            }
            (
                RdeCategory::Transformed,
                "semantic element transformed; review context before evaluating drift"
                    .to_string(),
                note,
            )
        }

        DeltaMRelationKind::Complemented => (
            RdeCategory::Complemented,
            "semantic element complemented; value is not asserted by this classifier"
                .to_string(),
            None,
        ),

        // Removed → NOT lost. Review focus via NextUpdatePolicy.
        DeltaMRelationKind::Removed => (
            RdeCategory::NextUpdatePolicy,
            "semantic element removed from target extraction; human review is required before classifying as lost semantic content"
                .to_string(),
            Some(
                if no_evidence {
                    "no evidence reference; removal observation relies on structural absence only"
                } else {
                    "removal is an observation, not a loss judgment"
                }
                .to_string(),
            ),
        ),

        // Weakened → NOT deviation_risk. Review focus.
        DeltaMRelationKind::Weakened => (
            RdeCategory::NextUpdatePolicy,
            "semantic element weakened; human review is required before assessing deviation risk"
                .to_string(),
            Some(
                "weakening is an observation; deviation risk requires evidence and context"
                    .to_string(),
            ),
        ),

        // Contradicted → NOT deviation_risk. Review focus.
        DeltaMRelationKind::Contradicted => (
            RdeCategory::NextUpdatePolicy,
            "semantic element contradicted; human review is required before assessing deviation risk"
                .to_string(),
            Some(
                "contradiction is an observation; deviation risk requires evidence and context"
                    .to_string(),
            ),
        ),

        // Unresolved → NOT automatically intentionally_unresolved. Review focus.
        DeltaMRelationKind::Unresolved => (
            RdeCategory::NextUpdatePolicy,
            "semantic element relation is unresolved; human review is required to determine whether this is intentional"
                .to_string(),
            Some(
                "unresolved is not automatically intentionally_unresolved; requires human confirmation"
                    .to_string(),
            ),
        ),

        // Split → transformed with uncertainty
        DeltaMRelationKind::Split => (
            RdeCategory::Transformed,
            "semantic element appears to have split; review context before evaluating meaning preservation across fragments"
                .to_string(),
            Some(
                "split relation observed; meaning preservation across fragments requires human review"
                    .to_string(),
            ),
        ),

        // Merged → transformed with uncertainty
        DeltaMRelationKind::Merged => (
            RdeCategory::Transformed,
            "semantic elements appear to have merged; review context before evaluating meaning preservation across the merge"
                .to_string(),
            Some(
                "merged relation observed; meaning preservation across the merge requires human review"
                    .to_string(),
            ),
        ),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rde_delta::{DeltaMRelation, DeltaMRelationKind, DeltaMReport};

    // ── helpers ──────────────────────────────────────────────────────────

    fn make_report(relations: Vec<DeltaMRelation>) -> DeltaMReport {
        DeltaMReport {
            subject_ref: "s/1".to_string(),
            relations,
        }
    }

    fn make_relation(kind: DeltaMRelationKind) -> DeltaMRelation {
        DeltaMRelation {
            source_element_id: Some("e1".to_string()),
            target_element_id: Some("e1".to_string()),
            relation: kind,
            summary: String::new(), // summary in DeltaMRelation is not used by classifier
            evidence_refs: vec!["ref:e1".to_string()],
        }
    }

    fn make_classifier() -> ConservativeRdeClassifier {
        ConservativeRdeClassifier
    }

    fn category_of(eval: &RdeEvaluation, cat: RdeCategory) -> Vec<&RdeObservation> {
        eval.observations
            .iter()
            .filter(|o| o.category == cat)
            .collect()
    }

    // ── Preserved → preserved ────────────────────────────────────────────

    #[test]
    fn preserved_maps_to_preserved_category() {
        let report = make_report(vec![make_relation(DeltaMRelationKind::Preserved)]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let items = category_of(&eval, RdeCategory::Preserved);
        assert_eq!(items.len(), 1);
        assert!(items[0].summary.contains("preserved"));
    }

    // ── Transformed → transformed ────────────────────────────────────────

    #[test]
    fn transformed_maps_to_transformed_category() {
        let report = make_report(vec![make_relation(DeltaMRelationKind::Transformed)]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let items = category_of(&eval, RdeCategory::Transformed);
        assert_eq!(items.len(), 1);
        assert!(items[0].summary.contains("transformed"));
    }

    // ── Complemented → complemented ──────────────────────────────────────

    #[test]
    fn complemented_maps_to_complemented_category() {
        let report = make_report(vec![make_relation(DeltaMRelationKind::Complemented)]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let items = category_of(&eval, RdeCategory::Complemented);
        assert_eq!(items.len(), 1);
        assert!(items[0].summary.contains("complemented"));
        assert!(items[0].summary.contains("value is not asserted"));
    }

    // ── Removed → NOT lost ───────────────────────────────────────────────

    #[test]
    fn removed_does_not_map_to_lost() {
        let report = make_report(vec![make_relation(DeltaMRelationKind::Removed)]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let lost_items = category_of(&eval, RdeCategory::Lost);
        assert!(lost_items.is_empty(), "Removed must not be mapped to lost");
    }

    #[test]
    fn removed_is_review_focus_in_next_update_policy() {
        let report = make_report(vec![make_relation(DeltaMRelationKind::Removed)]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let items = category_of(&eval, RdeCategory::NextUpdatePolicy);
        assert_eq!(items.len(), 1);
        assert!(items[0].summary.contains("removed"));
        assert!(items[0].summary.contains("human review"));
        assert!(items[0].summary.contains("before classifying as lost"));
    }

    // ── Contradicted → NOT deviation_risk ────────────────────────────────

    #[test]
    fn contradicted_does_not_map_to_deviation_risk() {
        let report = make_report(vec![make_relation(DeltaMRelationKind::Contradicted)]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let dr_items = category_of(&eval, RdeCategory::DeviationRisk);
        assert!(
            dr_items.is_empty(),
            "Contradicted must not be mapped to deviation_risk"
        );
    }

    // ── Weakened → NOT deviation_risk ────────────────────────────────────

    #[test]
    fn weakened_does_not_map_to_deviation_risk() {
        let report = make_report(vec![make_relation(DeltaMRelationKind::Weakened)]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let dr_items = category_of(&eval, RdeCategory::DeviationRisk);
        assert!(
            dr_items.is_empty(),
            "Weakened must not be mapped to deviation_risk"
        );
    }

    // ── Unresolved → NOT intentionally_unresolved automatically ─────────

    #[test]
    fn unresolved_does_not_map_to_intentionally_unresolved_automatically() {
        let report = make_report(vec![make_relation(DeltaMRelationKind::Unresolved)]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let iu_items = category_of(&eval, RdeCategory::IntentionallyUnresolved);
        assert!(
            iu_items.is_empty(),
            "Unresolved must not be mapped to intentionally_unresolved automatically"
        );
    }

    // ── Empty evidence → confidence note ─────────────────────────────────

    #[test]
    fn empty_evidence_refs_produces_confidence_note() {
        let mut rel = make_relation(DeltaMRelationKind::Preserved);
        rel.evidence_refs = vec![];
        let report = make_report(vec![rel]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let items = category_of(&eval, RdeCategory::Preserved);
        assert_eq!(items.len(), 1);
        let note = items[0]
            .confidence_note
            .as_deref()
            .expect("should have confidence_note");
        assert!(note.contains("no evidence"));
        assert!(note.contains("review focus"));
    }

    // ── No approval/rejection/safety verdicts ────────────────────────────

    #[test]
    fn classifier_contains_no_approval_or_safety_verdicts() {
        let report = make_report(vec![
            make_relation(DeltaMRelationKind::Preserved),
            make_relation(DeltaMRelationKind::Removed),
            make_relation(DeltaMRelationKind::Transformed),
        ]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");

        let all_text: String = eval
            .observations
            .iter()
            .flat_map(|o| {
                [o.summary.as_str()]
                    .into_iter()
                    .chain(o.confidence_note.as_deref().into_iter())
            })
            .fold(String::new(), |mut acc, t| {
                acc.push_str(t);
                acc.push(' ');
                acc
            });

        let forbidden = [
            "approved",
            "rejected",
            "safe",
            "unsafe",
            "access granted",
            "access denied",
            "this removal is a loss",
            "this transformation is dangerous",
            "this addition is valuable",
        ];
        for word in forbidden {
            assert!(
                !all_text.to_lowercase().contains(word),
                "classifier must not contain verdict word {word:?}"
            );
        }
    }

    // ── Classifier output passes existing validate_json ──────────────────

    #[test]
    fn classifier_output_validates_against_sls4() {
        let report = make_report(vec![
            make_relation(DeltaMRelationKind::Preserved),
            make_relation(DeltaMRelationKind::Transformed),
            make_relation(DeltaMRelationKind::Complemented),
            make_relation(DeltaMRelationKind::Removed),
        ]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let warnings = eval.validate(true).expect("validate_json should succeed");
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
    }

    // ── Empty DeltaMReport does not panic ────────────────────────────────

    #[test]
    fn empty_report_does_not_panic() {
        let report = make_report(vec![]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        // Should produce at least one observation noting empty context
        assert!(!eval.observations.is_empty());
    }

    // ── Empty subject_ref → validation error ─────────────────────────────

    #[test]
    fn empty_subject_ref_produces_validation_error() {
        let report = DeltaMReport {
            subject_ref: "   ".to_string(),
            relations: vec![],
        };
        let result = make_classifier().classify(&report);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RdeError::Validation(_)));
    }

    // ── All relation kinds are handled (no panic on new variants) ────────

    #[test]
    fn all_relation_kinds_classified_without_panic() {
        let kinds = [
            DeltaMRelationKind::Preserved,
            DeltaMRelationKind::Transformed,
            DeltaMRelationKind::Complemented,
            DeltaMRelationKind::Weakened,
            DeltaMRelationKind::Removed,
            DeltaMRelationKind::Split,
            DeltaMRelationKind::Merged,
            DeltaMRelationKind::Contradicted,
            DeltaMRelationKind::Unresolved,
        ];

        for kind in kinds {
            let report = make_report(vec![make_relation(kind)]);
            let eval = make_classifier().classify(&report);
            assert!(
                eval.is_ok(),
                "classifier should handle {kind:?} without panic"
            );
        }
    }

    // ── Complemented is not treated as valuable ──────────────────────────

    #[test]
    fn complemented_text_does_not_assert_value() {
        let report = make_report(vec![make_relation(DeltaMRelationKind::Complemented)]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let items = category_of(&eval, RdeCategory::Complemented);
        let text = &items[0].summary.to_lowercase();
        assert!(!text.contains("valuable"));
        assert!(!text.contains("good"));
        assert!(!text.contains("improvement"));
    }

    // ── evidence_refs are preserved in output ────────────────────────────

    #[test]
    fn evidence_refs_preserved_in_output() {
        let mut rel = make_relation(DeltaMRelationKind::Preserved);
        rel.evidence_refs = vec!["ref:issue-1".to_string(), "ref:commit-abc".to_string()];
        let report = make_report(vec![rel]);
        let eval = make_classifier()
            .classify(&report)
            .expect("classify succeeds");
        let items = category_of(&eval, RdeCategory::Preserved);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].evidence_refs.len(), 2);
        assert!(items[0].evidence_refs.contains(&"ref:issue-1".to_string()));
        assert!(items[0]
            .evidence_refs
            .contains(&"ref:commit-abc".to_string()));
    }
}
