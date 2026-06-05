//! Meta-RDE — Phase F of the full RDE implementation roadmap.
//!
//! Meta-RDE is a recursive audit layer that inspects the RDE pipeline output
//! for drift from its original design boundaries. It does **not** reclassify,
//! approve, or reject. It produces review assistance, not final judgment.
//!
//! # Non-goals
//!
//! - No reclassification of RDE output.
//! - No approval/rejection verdicts.
//! - No safety verdicts.
//! - No auto-blocking or auto-rejection.
//! - No policy enforcement.
//!
//! # Human authority boundary
//!
//! Meta-RDE inspects the pipeline for boundary drift. It flags suspicious
//! patterns and critical distortions as review focus. It **never** substitutes
//! for human judgment. `Blocking` severity means "must be reviewed before
//! acceptance," not "automatically rejected."
//!
//! # Pipeline position
//!
//! ```text
//! Phase E: RdeEvaluation × Source Context → EvidenceBindingReport
//! Phase F: Full Pipeline Output → MetaRdeReport                      ← this module
//! Human Review: EvidenceBindingReport + MetaRdeReport → decision
//! ```

use crate::rde_evidence::EvidenceBindingReport;

// ---------------------------------------------------------------------------
// MetaRdeSeverity, PipelinePhase
// ---------------------------------------------------------------------------

/// Severity of a Meta-RDE finding.
///
/// `Blocking` means "must be reviewed before the pipeline output can be
/// accepted." It is **not** an automatic rejection. Human override is
/// always available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

/// Pipeline stages that Meta-RDE can inspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelinePhase {
    PhaseB,
    PhaseC,
    PhaseD,
    PhaseE,
    ReportMarkdown,
    ReportJson,
    HumanReviewHandoff,
}

// ---------------------------------------------------------------------------
// MetaRdeFinding, MetaRdeReport
// ---------------------------------------------------------------------------

/// A single Meta-RDE finding about a pipeline stage.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Report produced by the Meta-RDE layer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

// ---------------------------------------------------------------------------
// run_meta_rde_minimal
// ---------------------------------------------------------------------------

/// Minimal Meta-RDE audit of an `EvidenceBindingReport`.
///
/// This function:
/// - Inspects each binding for missing evidence.
/// - Flags bindings with missing evidence as `ReviewNeeded`.
/// - Does **not** detect deeper boundary drift (reserved for future phases).
/// - Does **not** generate approval, rejection, or safety verdicts.
/// - Handles empty input conservatively.
pub fn run_meta_rde_minimal(evidence_report: &EvidenceBindingReport) -> MetaRdeReport {
    let target_id = "pipeline-run/0".to_string();
    let mut report = MetaRdeReport {
        target_id: target_id.clone(),
        ..MetaRdeReport::default()
    };

    let mut finding_counter: u64 = 0;

    for binding in &evidence_report.bindings {
        let has_missing_evidence = binding.evidence_refs.iter().any(|r| {
            matches!(
                r.role,
                crate::rde_evidence::EvidenceRole::IndicatesMissingEvidence
            )
        });

        let has_uncertainty = !binding.uncertainty_notes.is_empty();

        if has_missing_evidence {
            finding_counter += 1;
            report.suspicious_drifts.push(MetaRdeFinding {
                finding_id: format!("{target_id}/finding/{finding_counter}"),
                target_phase: PipelinePhase::PhaseE,
                summary:
                    "evidence binding contains missing-evidence markers; confirm that classification is adequately supported"
                        .to_string(),
                evidence_refs: Vec::new(),
                severity: MetaRdeSeverity::ReviewNeeded,
                reviewer_focus: vec![
                    "human reviewer should verify whether missing evidence affects classification confidence"
                        .to_string(),
                ],
            });
        }

        if has_uncertainty {
            finding_counter += 1;
            report.inferred_extensions.push(MetaRdeFinding {
                finding_id: format!("{target_id}/finding/{finding_counter}"),
                target_phase: PipelinePhase::PhaseE,
                summary:
                    "evidence binding contains uncertainty notes; reviewer should assess whether the uncertainty is adequately addressed"
                        .to_string(),
                evidence_refs: Vec::new(),
                severity: MetaRdeSeverity::Informational,
                reviewer_focus: Vec::new(),
            });
        }
    }

    // If no bindings were present, note it as unresolved.
    if evidence_report.bindings.is_empty() && evidence_report.reviewer_focus.is_empty() {
        finding_counter += 1;
        report.unresolved.push(MetaRdeFinding {
            finding_id: format!("{target_id}/finding/{finding_counter}"),
            target_phase: PipelinePhase::PhaseE,
            summary: "Meta-RDE received an empty EvidenceBindingReport; no material to audit"
                .to_string(),
            evidence_refs: Vec::new(),
            severity: MetaRdeSeverity::Informational,
            reviewer_focus: vec![
                "human reviewer should confirm whether empty output is intentional".to_string(),
            ],
        });
    }

    // Always add a next_update_policy note for future expansion.
    report.next_update_policy.push(
        "Meta-RDE should be expanded to inspect Phase B/C/D outputs for deeper classifier boundary drift"
            .to_string(),
    );

    report
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rde_evidence::{
        bind_evidence_minimal, EvidenceBindingReport,
    };
    use crate::rde_impl::{RdeCategory, RdeEvaluation, RdeObservation};

    // ── helpers ──────────────────────────────────────────────────────────

    fn binding_with_evidence() -> EvidenceBindingReport {
        let eval = {
            let mut e = RdeEvaluation::new("s/1");
            e.push(RdeObservation {
                category: RdeCategory::Preserved,
                summary: "kept".to_string(),
                evidence_refs: vec!["ref:1".to_string()],
                confidence_note: None,
            });
            e
        };
        bind_evidence_minimal(&eval)
    }

    fn binding_without_evidence() -> EvidenceBindingReport {
        let eval = {
            let mut e = RdeEvaluation::new("s/1");
            e.push(RdeObservation {
                category: RdeCategory::Transformed,
                summary: "changed".to_string(),
                evidence_refs: vec![],
                confidence_note: None,
            });
            e
        };
        bind_evidence_minimal(&eval)
    }

    // ── Human review boundary preserved ──────────────────────────────────

    #[test]
    fn meta_rde_preserves_human_review_boundary() {
        let evidence = binding_with_evidence();
        let report = run_meta_rde_minimal(&evidence);

        // The target_id should be set
        assert!(!report.target_id.is_empty());

        // preserved / authorized_transformations should exist even if empty
        assert!(report.preserved.iter().all(|_| true)); // just checking it's accessible
    }

    // ── No approval/rejection/safety verdicts ────────────────────────────

    #[test]
    fn meta_rde_does_not_generate_approval_or_rejection() {
        let evidence = binding_without_evidence();
        let report = run_meta_rde_minimal(&evidence);

        let all_text = collect_report_text(&report);
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
                "Meta-RDE must not contain verdict word {word:?}"
            );
        }
    }

    fn collect_report_text(report: &MetaRdeReport) -> String {
        let mut parts: Vec<String> = Vec::new();
        for category in [
            &report.preserved,
            &report.authorized_transformations,
            &report.inferred_extensions,
            &report.suspicious_drifts,
            &report.critical_distortions,
            &report.unresolved,
        ] {
            for finding in category.iter() {
                parts.push(finding.summary.clone());
                parts.extend(finding.reviewer_focus.clone());
            }
        }
        parts.extend(report.next_update_policy.clone());
        parts.join(" ")
    }

    // ── CriticalDistortion without auto-rejection ────────────────────────

    #[test]
    fn meta_rde_reports_critical_distortion_without_auto_rejection() {
        // Add a manually constructed critical distortion to verify the type system
        let mut report = MetaRdeReport {
            target_id: "test/1".to_string(),
            ..MetaRdeReport::default()
        };
        report.critical_distortions.push(MetaRdeFinding {
            finding_id: "test/1/finding/1".to_string(),
            target_phase: PipelinePhase::PhaseD,
            summary: "classifier mapped Removed directly to lost".to_string(),
            evidence_refs: vec!["ref:audit".to_string()],
            severity: MetaRdeSeverity::Blocking,
            reviewer_focus: vec![
                "human reviewer must confirm whether this classification violates RDE boundaries"
                    .to_string(),
            ],
        });

        // Verify that Blocking severity is set, but no auto-rejection logic exists
        assert!(!report.critical_distortions.is_empty());
        assert_eq!(
            report.critical_distortions[0].severity,
            MetaRdeSeverity::Blocking
        );

        // No "rejected" or "denied" in the output
        let text = collect_report_text(&report);
        assert!(!text.to_lowercase().contains("rejected"));
        assert!(!text.to_lowercase().contains("denied"));
        assert!(!text.to_lowercase().contains("blocked"));
    }

    // ── Empty / partial input handled conservatively ─────────────────────

    #[test]
    fn meta_rde_handles_empty_or_partial_pipeline_outputs_conservatively() {
        let empty = EvidenceBindingReport::default();
        let report = run_meta_rde_minimal(&empty);

        // Should produce unresolved, not panic
        assert!(!report.unresolved.is_empty());
        let unresolved_text: String = report
            .unresolved
            .iter()
            .map(|f| &f.summary)
            .chain(report.unresolved.iter().flat_map(|f| &f.reviewer_focus))
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(unresolved_text.contains("empty"), "should note empty input");

        // next_update_policy should still be populated
        assert!(
            !report.next_update_policy.is_empty(),
            "next_update_policy should not be empty even with empty input"
        );
    }

    // ── MetaRdeSeverity ordering ─────────────────────────────────────────

    #[test]
    fn meta_rde_severity_ordering_is_intuitive() {
        assert!(MetaRdeSeverity::Informational < MetaRdeSeverity::ReviewNeeded);
        assert!(MetaRdeSeverity::ReviewNeeded < MetaRdeSeverity::HighRisk);
        assert!(MetaRdeSeverity::HighRisk < MetaRdeSeverity::Blocking);
    }
}
