//! RDE ΔM report scaffold — Phase C of the full RDE implementation roadmap.
//!
//! This module introduces the structural scaffolding for recording relationships
//! between semantic elements across revisions. It does **not** claim complete
//! semantic ΔM analysis. Instead, it provides typed containers so that later RDE
//! classification (Phase D) can map relations into SLS-4 categories with explicit
//! evidence hooks.
//!
//! # Non-goals
//!
//! - No LLM integration.
//! - No fuzzy semantic matching.
//! - No SLS-4 final category classification.
//! - No policy enforcement.
//! - No final approval/rejection.
//! - No model dependency.
//!
//! # Human authority boundary
//!
//! ΔM relations recorded through this module are structured observations of how
//! semantic elements moved across revisions. They are **not** value judgments.
//!
//! - `Removed` is not automatically `lost`.
//! - `Complemented` is not automatically a valuable addition.
//! - `Transformed` is not automatically a dangerous deviation.
//!
//! All of these require evidence, human review, and explicit classification
//! (Phase D) before they become RDE category judgments.
//!
//! # Pipeline position
//!
//! ```text
//! Phase B: RdeContextBundle → SemanticExtraction
//! Phase C: SemanticExtraction × SemanticExtraction → DeltaMReport
//! Phase D: DeltaMReport → RdeEvaluation / SLS-4 categories
//! ```

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::rde_semantic::{RdeError, SemanticElement, SemanticExtraction};

// ---------------------------------------------------------------------------
// DeltaMRelationKind
// ---------------------------------------------------------------------------

/// Kind of ΔM relation between two semantic elements across revisions.
///
/// This enum describes **how** meaning moved; it does **not** declare whether
/// the movement is good, bad, safe, or dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaMRelationKind {
    /// Element preserved with identical kind and text.
    Preserved,
    /// Element kind or text changed while carrying meaning forward.
    Transformed,
    /// New element present in target but not in source.
    Complemented,
    /// Element kind and text weakened or scope narrowed.
    Weakened,
    /// Element present in source but absent in target.
    Removed,
    /// Two or more source elements merged into one target element.
    Merged,
    /// One source element split into two or more target elements.
    Split,
    /// Element in source appears to be contradicted by a target element.
    Contradicted,
    /// Relation cannot be confidently determined from structured data alone.
    Unresolved,
}

impl DeltaMRelationKind {
    /// Returns a stable lowercase key for each variant.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Preserved => "preserved",
            Self::Transformed => "transformed",
            Self::Complemented => "complemented",
            Self::Weakened => "weakened",
            Self::Removed => "removed",
            Self::Split => "split",
            Self::Merged => "merged",
            Self::Contradicted => "contradicted",
            Self::Unresolved => "unresolved",
        }
    }
}

// ---------------------------------------------------------------------------
// DeltaMRelation
// ---------------------------------------------------------------------------

/// A single ΔM relation between source and target semantic elements.
///
/// This struct does **not** carry an approval/rejection verdict, a safety
/// classification, or an access-control decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaMRelation {
    /// Identifier of the source element, when traceable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_element_id: Option<String>,
    /// Identifier of the target element, when traceable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_element_id: Option<String>,
    /// How the element moved across the revision.
    pub relation: DeltaMRelationKind,
    /// Concise deterministic summary of the observed relation.
    pub summary: String,
    /// Evidence references carried over from `SemanticElement.source_ref`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
}

// ---------------------------------------------------------------------------
// DeltaMReport
// ---------------------------------------------------------------------------

/// Result of analyzing ΔM between two `SemanticExtraction` objects.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DeltaMReport {
    /// Subject reference shared by both extractions.
    pub subject_ref: String,
    /// All observed ΔM relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<DeltaMRelation>,
}

impl DeltaMReport {
    /// Creates an empty report for the given subject reference.
    pub fn new(subject_ref: impl Into<String>) -> Self {
        Self {
            subject_ref: subject_ref.into(),
            relations: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// DeltaMAnalyzer trait — SLS-5.4.3 / Layer 3 (ΔM layer)
// ---------------------------------------------------------------------------

/// Contract for components that produce `DeltaMReport` from two extractions.
///
/// Implementations may be rule-based, model-assisted, or human-curated. The
/// trait does **not** mandate a specific model provider or algorithm.
///
/// # Human authority boundary
///
/// Implementations **MUST NOT** emit approval, rejection, safety verdicts, or
/// access-control decisions through this trait. Analysis is structured
/// observation, not judgment.
pub trait DeltaMAnalyzer {
    /// Analyzes ΔM between source and target `SemanticExtraction` objects.
    ///
    /// Returns `DeltaMReport` on success. An empty `relations` list is a valid
    /// result when no material relations could be identified.
    fn analyze(
        &self,
        source: &SemanticExtraction,
        target: &SemanticExtraction,
    ) -> Result<DeltaMReport, RdeError>;
}

// ---------------------------------------------------------------------------
// ConservativeDeltaMAnalyzer — deterministic, no model dependency
// ---------------------------------------------------------------------------

/// A deterministic, conservative analyzer that compares `SemanticExtraction`
/// objects using id-based matching with kind/text comparison.
///
/// Matching strategy (deterministic, no fuzzy logic):
///
/// - Elements with the same `id` in both source and target, same `kind` and
///   `text` → `Preserved`.
/// - Elements with the same `id` in both source and target, different `kind`
///   or `text` → `Transformed`.
/// - Elements present only in source → `Removed`.
/// - Elements present only in target → `Complemented`.
/// - `Weakened`, `Split`, `Merged`, `Contradicted`, `Unresolved` are enum
///   variants available for future analyzers but are **not** generated by
///   this conservative implementation.
///
/// `evidence_refs` are copied from `SemanticElement.source_ref` when present.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConservativeDeltaMAnalyzer;

impl DeltaMAnalyzer for ConservativeDeltaMAnalyzer {
    fn analyze(
        &self,
        source: &SemanticExtraction,
        target: &SemanticExtraction,
    ) -> Result<DeltaMReport, RdeError> {
        // Validate subject_refs
        let src_ref = source.subject_ref.trim();
        let tgt_ref = target.subject_ref.trim();

        if src_ref.is_empty() || tgt_ref.is_empty() {
            return Err(RdeError::Validation(
                "both source and target subject_ref must be non-empty".to_string(),
            ));
        }

        if src_ref != tgt_ref {
            return Err(RdeError::Validation(format!(
                "source subject_ref {src_ref:?} does not match target subject_ref {tgt_ref:?}"
            )));
        }

        let mut report = DeltaMReport::new(src_ref);

        // Build lookup: id → SemanticElement for both sides
        let source_by_id: std::collections::HashMap<&str, &SemanticElement> =
            source.elements.iter().map(|e| (e.id.as_str(), e)).collect();

        let target_by_id: std::collections::HashMap<&str, &SemanticElement> =
            target.elements.iter().map(|e| (e.id.as_str(), e)).collect();

        let source_ids: HashSet<&str> = source_by_id.keys().copied().collect();
        let target_ids: HashSet<&str> = target_by_id.keys().copied().collect();

        // Elements in both source and target → Preserved or Transformed
        for id in source_ids.intersection(&target_ids) {
            let src_elem = source_by_id[id];
            let tgt_elem = target_by_id[id];

            let evidence = src_elem
                .source_ref
                .as_deref()
                .map(|r| vec![r.to_string()])
                .unwrap_or_default();

            if src_elem.kind == tgt_elem.kind && src_elem.text == tgt_elem.text {
                report.relations.push(DeltaMRelation {
                    source_element_id: Some((*id).to_string()),
                    target_element_id: Some((*id).to_string()),
                    relation: DeltaMRelationKind::Preserved,
                    summary: "semantic element preserved by id".to_string(),
                    evidence_refs: evidence,
                });
            } else {
                report.relations.push(DeltaMRelation {
                    source_element_id: Some((*id).to_string()),
                    target_element_id: Some((*id).to_string()),
                    relation: DeltaMRelationKind::Transformed,
                    summary: "semantic element transformed by id".to_string(),
                    evidence_refs: evidence,
                });
            }
        }

        // Elements only in source → Removed
        for id in source_ids.difference(&target_ids) {
            let src_elem = source_by_id[id];
            let evidence = src_elem
                .source_ref
                .as_deref()
                .map(|r| vec![r.to_string()])
                .unwrap_or_default();

            report.relations.push(DeltaMRelation {
                source_element_id: Some((*id).to_string()),
                target_element_id: None,
                relation: DeltaMRelationKind::Removed,
                summary: "semantic element removed from target extraction".to_string(),
                evidence_refs: evidence,
            });
        }

        // Elements only in target → Complemented
        for id in target_ids.difference(&source_ids) {
            let tgt_elem = target_by_id[id];
            let evidence = tgt_elem
                .source_ref
                .as_deref()
                .map(|r| vec![r.to_string()])
                .unwrap_or_default();

            report.relations.push(DeltaMRelation {
                source_element_id: None,
                target_element_id: Some((*id).to_string()),
                relation: DeltaMRelationKind::Complemented,
                summary: "semantic element complemented in target extraction".to_string(),
                evidence_refs: evidence,
            });
        }

        Ok(report)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rde_semantic::{SemanticElement, SemanticElementKind};

    // ── helpers ──────────────────────────────────────────────────────────

    fn make_element(id: &str, kind: SemanticElementKind, text: &str) -> SemanticElement {
        SemanticElement {
            id: id.to_string(),
            kind,
            text: text.to_string(),
            source_ref: Some(format!("ref:{id}")),
            confidence_note: None,
            scope: None,
        }
    }

    fn make_extraction(subject_ref: &str, elements: Vec<SemanticElement>) -> SemanticExtraction {
        SemanticExtraction {
            subject_ref: subject_ref.to_string(),
            elements,
        }
    }

    fn make_analyzer() -> ConservativeDeltaMAnalyzer {
        ConservativeDeltaMAnalyzer
    }

    // ── Preserved: same id, same kind, same text ───────────────────────

    #[test]
    fn preserved_when_id_kind_text_identical() {
        let source = make_extraction(
            "s/1",
            vec![make_element(
                "e1",
                SemanticElementKind::Intent,
                "keep stable",
            )],
        );
        let target = make_extraction(
            "s/1",
            vec![make_element(
                "e1",
                SemanticElementKind::Intent,
                "keep stable",
            )],
        );

        let report = make_analyzer()
            .analyze(&source, &target)
            .expect("analysis succeeds");
        assert_eq!(report.subject_ref, "s/1");
        assert_eq!(report.relations.len(), 1);
        assert_eq!(report.relations[0].relation, DeltaMRelationKind::Preserved);
        assert_eq!(report.relations[0].source_element_id.as_deref(), Some("e1"));
        assert_eq!(report.relations[0].target_element_id.as_deref(), Some("e1"));
    }

    // ── Transformed: same id, same kind, different text ─────────────────

    #[test]
    fn transformed_when_id_same_kind_same_text_differs() {
        let source = make_extraction(
            "s/1",
            vec![make_element(
                "e1",
                SemanticElementKind::Intent,
                "keep stable",
            )],
        );
        let target = make_extraction(
            "s/1",
            vec![make_element(
                "e1",
                SemanticElementKind::Intent,
                "add caching layer",
            )],
        );

        let report = make_analyzer()
            .analyze(&source, &target)
            .expect("analysis succeeds");
        assert_eq!(report.relations.len(), 1);
        assert_eq!(
            report.relations[0].relation,
            DeltaMRelationKind::Transformed
        );
    }

    // ── Transformed: same id, different kind ────────────────────────────

    #[test]
    fn transformed_when_id_same_kind_differs() {
        let source = make_extraction(
            "s/1",
            vec![make_element(
                "e1",
                SemanticElementKind::Intent,
                "keep stable",
            )],
        );
        let target = make_extraction(
            "s/1",
            vec![make_element(
                "e1",
                SemanticElementKind::Constraint,
                "keep stable",
            )],
        );

        let report = make_analyzer()
            .analyze(&source, &target)
            .expect("analysis succeeds");
        assert_eq!(report.relations.len(), 1);
        assert_eq!(
            report.relations[0].relation,
            DeltaMRelationKind::Transformed
        );
    }

    // ── Removed: source element not in target ───────────────────────────

    #[test]
    fn removed_when_element_only_in_source() {
        let source = make_extraction(
            "s/1",
            vec![make_element("e1", SemanticElementKind::Intent, "scope A")],
        );
        let target = make_extraction("s/1", vec![]);

        let report = make_analyzer()
            .analyze(&source, &target)
            .expect("analysis succeeds");
        assert_eq!(report.relations.len(), 1);
        assert_eq!(report.relations[0].relation, DeltaMRelationKind::Removed);
        assert_eq!(report.relations[0].source_element_id.as_deref(), Some("e1"));
        assert_eq!(report.relations[0].target_element_id, None);
    }

    // ── Complemented: target element not in source ──────────────────────

    #[test]
    fn complemented_when_element_only_in_target() {
        let source = make_extraction("s/1", vec![]);
        let target = make_extraction(
            "s/1",
            vec![make_element(
                "e2",
                SemanticElementKind::Assumption,
                "assumes postgres",
            )],
        );

        let report = make_analyzer()
            .analyze(&source, &target)
            .expect("analysis succeeds");
        assert_eq!(report.relations.len(), 1);
        assert_eq!(
            report.relations[0].relation,
            DeltaMRelationKind::Complemented
        );
        assert_eq!(report.relations[0].source_element_id, None);
        assert_eq!(report.relations[0].target_element_id.as_deref(), Some("e2"));
    }

    // ── Empty extractions do not panic ──────────────────────────────────

    #[test]
    fn empty_extractions_do_not_panic() {
        let source = make_extraction("s/1", vec![]);
        let target = make_extraction("s/1", vec![]);

        let report = make_analyzer()
            .analyze(&source, &target)
            .expect("analysis succeeds");
        assert_eq!(report.subject_ref, "s/1");
        assert!(report.relations.is_empty());
    }

    // ── Validation: empty subject_ref ───────────────────────────────────

    #[test]
    fn rejects_empty_source_subject_ref() {
        let source = make_extraction("   ", vec![]);
        let target = make_extraction("s/1", vec![]);

        let result = make_analyzer().analyze(&source, &target);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RdeError::Validation(_)));
    }

    #[test]
    fn rejects_empty_target_subject_ref() {
        let source = make_extraction("s/1", vec![]);
        let target = make_extraction("", vec![]);

        let result = make_analyzer().analyze(&source, &target);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RdeError::Validation(_)));
    }

    // ── Validation: subject_ref mismatch ────────────────────────────────

    #[test]
    fn rejects_subject_ref_mismatch() {
        let source = make_extraction("s/1", vec![]);
        let target = make_extraction("s/2", vec![]);

        let result = make_analyzer().analyze(&source, &target);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(&err, RdeError::Validation(_)),
            "expected Validation error, got {err:?}"
        );
        assert!(err.to_string().contains("does not match"));
    }

    // ── JSON roundtrip ──────────────────────────────────────────────────

    #[test]
    fn delta_m_report_roundtrips_json() {
        let report = DeltaMReport {
            subject_ref: "s/1".to_string(),
            relations: vec![
                DeltaMRelation {
                    source_element_id: Some("e1".to_string()),
                    target_element_id: Some("e1".to_string()),
                    relation: DeltaMRelationKind::Preserved,
                    summary: "preserved".to_string(),
                    evidence_refs: vec!["ref:e1".to_string()],
                },
                DeltaMRelation {
                    source_element_id: Some("e2".to_string()),
                    target_element_id: None,
                    relation: DeltaMRelationKind::Removed,
                    summary: "removed".to_string(),
                    evidence_refs: vec![],
                },
            ],
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let back: DeltaMReport = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(back.subject_ref, "s/1");
        assert_eq!(back.relations.len(), 2);
        assert_eq!(back.relations[0].relation, DeltaMRelationKind::Preserved);
        assert_eq!(back.relations[1].relation, DeltaMRelationKind::Removed);
    }

    // ── No approval/rejection/safety verdicts ───────────────────────────

    #[test]
    fn analysis_contains_no_approval_or_safety_verdicts() {
        let source = make_extraction(
            "s/1",
            vec![
                make_element("e1", SemanticElementKind::Intent, "scope A"),
                make_element("e2", SemanticElementKind::Constraint, "must not break"),
            ],
        );
        let target = make_extraction(
            "s/1",
            vec![
                make_element("e1", SemanticElementKind::Intent, "scope B"),
                make_element("e3", SemanticElementKind::Assumption, "postgres"),
            ],
        );

        let report = make_analyzer()
            .analyze(&source, &target)
            .expect("analysis succeeds");

        let all_text: String = report
            .relations
            .iter()
            .flat_map(|r| [r.summary.as_str()].into_iter())
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
            "loss",
            "dangerous",
            "valuable",
        ];
        for word in forbidden {
            assert!(
                !all_text.to_lowercase().contains(word),
                "analysis must not contain verdict word {word:?}"
            );
        }
    }

    // ── Evidence refs inherited from source_ref ─────────────────────────

    #[test]
    fn evidence_refs_inherited_from_source_element() {
        let source = make_extraction(
            "s/1",
            vec![make_element("e1", SemanticElementKind::Intent, "scope A")],
        );
        let target = make_extraction(
            "s/1",
            vec![make_element(
                "e2",
                SemanticElementKind::Constraint,
                "limit scope",
            )],
        );

        let report = make_analyzer()
            .analyze(&source, &target)
            .expect("analysis succeeds");

        // e1 → Removed, should carry ref:e1
        let removed: Vec<_> = report
            .relations
            .iter()
            .filter(|r| r.relation == DeltaMRelationKind::Removed)
            .collect();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].evidence_refs, vec!["ref:e1"]);

        // e2 → Complemented, should carry ref:e2
        let complemented: Vec<_> = report
            .relations
            .iter()
            .filter(|r| r.relation == DeltaMRelationKind::Complemented)
            .collect();
        assert_eq!(complemented.len(), 1);
        assert_eq!(complemented[0].evidence_refs, vec!["ref:e2"]);
    }

    // ── Multiple preserved + transformed + removed + complemented ───────

    #[test]
    fn mixed_relations_in_single_report() {
        let source = make_extraction(
            "s/mixed",
            vec![
                make_element("e1", SemanticElementKind::Intent, "keep stable"),
                make_element("e2", SemanticElementKind::Constraint, "backward compat"),
            ],
        );
        let target = make_extraction(
            "s/mixed",
            vec![
                make_element("e1", SemanticElementKind::Intent, "keep stable"),
                make_element("e2", SemanticElementKind::Constraint, "forward compat"),
                make_element("e3", SemanticElementKind::Risk, "db migration risk"),
            ],
        );

        let report = make_analyzer()
            .analyze(&source, &target)
            .expect("analysis succeeds");

        let counts =
            report
                .relations
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, r| {
                    *acc.entry(r.relation).or_insert(0) += 1;
                    acc
                });

        assert_eq!(counts.get(&DeltaMRelationKind::Preserved), Some(&1)); // e1
        assert_eq!(counts.get(&DeltaMRelationKind::Transformed), Some(&1)); // e2
        assert_eq!(counts.get(&DeltaMRelationKind::Complemented), Some(&1));
        // e3
        assert_eq!(report.relations.len(), 3);
    }
}
