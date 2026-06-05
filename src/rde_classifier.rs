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

    // ── End-to-end pipeline tests (Phase B → C → D) ────────────────────

    use crate::rde_delta::{ConservativeDeltaMAnalyzer, DeltaMAnalyzer};
    use crate::rde_impl::RdeSubject;
    use crate::rde_semantic::{ConservativeSemanticExtractor, RdeContextBundle, SemanticExtractor};

    /// Runs the full Phase B → C → D pipeline and returns the resulting `RdeEvaluation`.
    fn run_pipeline(
        source_context: &RdeContextBundle,
        target_context: &RdeContextBundle,
    ) -> RdeEvaluation {
        let extractor = ConservativeSemanticExtractor;
        let source_extraction = extractor
            .extract(source_context)
            .expect("Phase B: extract source");
        let target_extraction = extractor
            .extract(target_context)
            .expect("Phase B: extract target");

        let analyzer = ConservativeDeltaMAnalyzer;
        let report = analyzer
            .analyze(&source_extraction, &target_extraction)
            .expect("Phase C: analyze delta");

        let classifier = ConservativeRdeClassifier;
        let eval = classifier
            .classify(&report)
            .expect("Phase D: classify to RDE");

        // Validate SLS-4 output shape
        let warnings = eval.validate(true).expect("validate_json should succeed");
        assert!(
            warnings.is_empty(),
            "unexpected validation warnings: {warnings:?}"
        );

        eval
    }

    fn subject(ref_id: &str) -> RdeSubject {
        RdeSubject::new(ref_id)
    }

    /// Helper: creates a context bundle with source_intent filled.
    fn context_with_intent(subject_ref: &str, intent: &str) -> RdeContextBundle {
        let mut ctx = RdeContextBundle::new(subject(subject_ref));
        ctx.source_intent = Some(intent.to_string());
        ctx
    }

    /// Helper: creates a context bundle with must_not_lose items.
    fn context_with_must_not_lose(subject_ref: &str, items: &[&str]) -> RdeContextBundle {
        let mut ctx = RdeContextBundle::new(subject(subject_ref));
        ctx.must_not_lose = items.iter().map(|s| s.to_string()).collect();
        ctx
    }

    /// Helper: collects all text (summary + confidence_note) from an evaluation.
    fn all_text(eval: &RdeEvaluation) -> String {
        eval.observations
            .iter()
            .flat_map(|o| {
                [o.summary.as_str()]
                    .into_iter()
                    .chain(o.confidence_note.as_deref().into_iter())
            })
            .fold(String::new(), |mut acc, t| {
                if !acc.is_empty() {
                    acc.push(' ');
                }
                acc.push_str(t);
                acc
            })
    }

    // ── Pipeline: preserved / transformed / complemented basic case ─────

    #[test]
    fn pipeline_preserves_phase_boundaries() {
        let source = context_with_intent("s/pipeline-1", "Keep public API stable");
        let target = context_with_intent("s/pipeline-1", "Add caching layer");

        let eval = run_pipeline(&source, &target);

        // source_intent elements should appear as Preserved or Transformed
        let has_category = |cat| eval.observations.iter().any(|o| o.category == cat);
        // With the pipeline, the extractor generates Intent elements with auto-IDs.
        // The analyzer compares by id. Since both contexts generate the same auto-id
        // (subject_ref/element/1) but different text, they become Transformed.
        assert!(has_category(RdeCategory::Transformed));

        // No approval/rejection in the pipeline output
        let text = all_text(&eval);
        let forbidden = ["approved", "rejected", "safe", "unsafe"];
        for word in forbidden {
            assert!(
                !text.to_lowercase().contains(word),
                "pipeline must not contain verdict word {word:?}"
            );
        }

        // Output must validate against SLS-4
        assert!(eval.validate(true).unwrap().is_empty());
    }

    // ── Pipeline: no approval or safety verdicts in full pipeline ───────

    #[test]
    fn pipeline_does_not_generate_approval_or_safety_verdicts() {
        let source = context_with_intent("s/pipeline-safe", "Original intent");
        let target = context_with_intent("s/pipeline-safe", "Revised intent");

        let eval = run_pipeline(&source, &target);
        let text = all_text(&eval);

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
                !text.to_lowercase().contains(word),
                "pipeline must not contain verdict word {word:?}"
            );
        }
    }

    // ── Pipeline: uncertain changes routed to next_update_policy ────────

    #[test]
    fn pipeline_routes_uncertain_changes_to_next_update_policy() {
        // source has must_not_lose items, target removes one
        let source = context_with_must_not_lose(
            "s/pipeline-uncertain",
            &[
                "backward compatibility with v1 clients",
                "error message format",
            ],
        );
        let target = context_with_must_not_lose(
            "s/pipeline-uncertain",
            &["backward compatibility with v1 clients"],
        );
        // "error message format" is removed in target

        let eval = run_pipeline(&source, &target);

        // The removed must_not_lose should appear in next_update_policy,
        // NOT in lost
        let lost_items: Vec<_> = eval
            .observations
            .iter()
            .filter(|o| o.category == RdeCategory::Lost)
            .collect();
        assert!(
            lost_items.is_empty(),
            "removed must_not_lose must not appear in lost category"
        );

        let nup_items: Vec<_> = eval
            .observations
            .iter()
            .filter(|o| o.category == RdeCategory::NextUpdatePolicy)
            .collect();
        assert!(
            !nup_items.is_empty(),
            "removed must_not_lose should appear as review focus in next_update_policy"
        );
    }

    // ── Pipeline: empty evidence handled conservatively ─────────────────

    #[test]
    fn pipeline_handles_empty_evidence_conservatively() {
        // Both source and target are empty → no structured evidence at all
        let source = RdeContextBundle::new(subject("s/pipeline-empty"));
        let target = RdeContextBundle::new(subject("s/pipeline-empty"));

        let eval = run_pipeline(&source, &target);

        // Should not panic, and should produce some observation
        assert!(!eval.observations.is_empty());

        // All observations with empty evidence should have confidence notes
        let items_without_evidence: Vec<_> = eval
            .observations
            .iter()
            .filter(|o| o.evidence_refs.is_empty())
            .collect();
        for item in &items_without_evidence {
            assert!(
                item.confidence_note.is_some(),
                "items without evidence must have confidence_note"
            );
        }

        // No approval/rejection/safety verdicts
        let text = all_text(&eval);
        let forbidden = ["approved", "rejected", "safe", "dangerous", "loss"];
        for word in forbidden {
            assert!(!text.to_lowercase().contains(word));
        }
    }

    // ── Pipeline: no SLS-4 shortcut mapping ─────────────────────────────

    #[test]
    fn pipeline_does_not_shortcut_sls4_mapping() {
        // Source has intent + constraint; target has different intent, new risk, removed constraint
        let source = context_with_intent("s/pipeline-shortcut", "Keep API stable");
        let target = context_with_intent("s/pipeline-shortcut", "Add caching layer");

        let eval = run_pipeline(&source, &target);

        // Transformed → should be in transformed, NOT deviation_risk
        let transformed_items: Vec<_> = eval
            .observations
            .iter()
            .filter(|o| o.category == RdeCategory::Transformed)
            .collect();
        for item in &transformed_items {
            let text = format!(
                "{} {}",
                item.summary,
                item.confidence_note.as_deref().unwrap_or("")
            );
            assert!(
                !text.to_lowercase().contains("dangerous"),
                "transformed must not be labeled dangerous"
            );
            assert!(
                !text.to_lowercase().contains("deviation"),
                "transformed must not be shortcut to deviation"
            );
        }

        // Complemented items → should NOT assert value
        let complemented_items: Vec<_> = eval
            .observations
            .iter()
            .filter(|o| o.category == RdeCategory::Complemented)
            .collect();
        for item in &complemented_items {
            assert!(
                !item.summary.to_lowercase().contains("valuable"),
                "complemented must not assert value"
            );
            assert!(
                !item.summary.to_lowercase().contains("good"),
                "complemented must not assert goodness"
            );
        }

        // Output must validate against SLS-4
        assert!(eval.validate(true).unwrap().is_empty());
    }

    // ── Pipeline: Phase D output is review focus, not judgment ─────────

    #[test]
    fn pipeline_output_is_review_focus_not_judgment() {
        // source has must_not_lose items, target drops one → generates Removed → NextUpdatePolicy
        let source = context_with_must_not_lose(
            "s/pipeline-review",
            &[
                "backward compatibility with v1 clients",
                "error message format",
            ],
        );
        let target = context_with_must_not_lose(
            "s/pipeline-review",
            &["backward compatibility with v1 clients"],
        );

        let eval = run_pipeline(&source, &target);

        // The output must be valid SLS-4
        assert!(eval.validate(true).unwrap().is_empty());

        // The classifier output must not pretend to be final
        let text = all_text(&eval);
        assert!(
            !text.to_lowercase().contains("final") || text.to_lowercase().contains("not final"),
            "classifier output must not claim finality"
        );

        // next_update_policy must exist as review focus handoff to human review
        let has_nup = eval
            .observations
            .iter()
            .any(|o| o.category == RdeCategory::NextUpdatePolicy);
        assert!(
            has_nup,
            "pipeline must include next_update_policy for human review handoff"
        );
    }
}
