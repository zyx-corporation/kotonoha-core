//! RDE Combined Review Report — Phase G of the full RDE implementation roadmap.
//!
//! This module integrates `EvidenceBindingReport` and `MetaRdeReport` into a
//! single review artifact for human evaluation. It generates Markdown reports
//! but does **not** generate approval, rejection, or safety verdicts.
//!
//! # Non-goals
//!
//! - No approval/rejection verdicts.
//! - No safety verdicts.
//! - No decision logic.
//! - No policy enforcement.
//! - No UI or database integration.
//!
//! # Human authority boundary
//!
//! The combined report is a review artifact, not a decision object. It collects
//! evidence bindings, Meta-RDE findings, and reviewer focus items so a human
//! can evaluate them. It does not decide for the human.
//!
//! # Pipeline position
//!
//! ```text
//! Phase E: EvidenceBindingReport
//! Phase F: MetaRdeReport
//! Phase G: Combined Review Report (Markdown)   ← this module
//! Human Review: report → approval / rejection / revision
//! ```

use crate::meta_rde::MetaRdeReport;
use crate::rde_evidence::EvidenceBindingReport;

// ---------------------------------------------------------------------------
// RdeCombinedReviewReport
// ---------------------------------------------------------------------------

/// Combined review artifact for human evaluation.
///
/// This struct collects `EvidenceBindingReport`, `MetaRdeReport`, and
/// aggregated reviewer focus items. It is **not** a decision object.
/// It does not carry approval, rejection, or safety verdict fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdeCombinedReviewReport {
    /// Unique identifier for this review report.
    pub report_id: String,
    /// Evidence bindings from Phase E.
    pub evidence_report: EvidenceBindingReport,
    /// Meta-RDE findings from Phase F.
    pub meta_report: MetaRdeReport,
    /// Aggregated reviewer focus items.
    pub reviewer_summary: Vec<String>,
    /// Aggregated unresolved items.
    pub unresolved: Vec<String>,
    /// Aggregated next-update policy items.
    pub next_update_policy: Vec<String>,
}

// ---------------------------------------------------------------------------
// build_combined_review_report
// ---------------------------------------------------------------------------

/// Builds a combined review report from evidence and Meta-RDE reports.
///
/// This function:
/// - Preserves both input reports without modification.
/// - Aggregates `reviewer_focus` from evidence bindings.
/// - Collects `unresolved` findings from Meta-RDE.
/// - Collects `next_update_policy` from Meta-RDE.
/// - Does **not** generate approval, rejection, or safety verdicts.
pub fn build_combined_review_report(
    report_id: impl Into<String>,
    evidence_report: EvidenceBindingReport,
    meta_report: MetaRdeReport,
) -> RdeCombinedReviewReport {
    let mut reviewer_summary: Vec<String> = Vec::new();

    // Collect reviewer focus from evidence bindings
    for binding in &evidence_report.bindings {
        if !binding.reviewer_focus.is_empty() {
            reviewer_summary.extend(binding.reviewer_focus.clone());
        }
        if !binding.uncertainty_notes.is_empty() {
            reviewer_summary.extend(binding.uncertainty_notes.clone());
        }
    }
    reviewer_summary.extend(evidence_report.reviewer_focus.clone());

    // Collect unresolved from Meta-RDE
    let unresolved: Vec<String> = meta_report
        .unresolved
        .iter()
        .map(|f| f.summary.clone())
        .collect();

    // Next update policy from Meta-RDE
    let next_update_policy = meta_report.next_update_policy.clone();

    RdeCombinedReviewReport {
        report_id: report_id.into(),
        evidence_report,
        meta_report,
        reviewer_summary,
        unresolved,
        next_update_policy,
    }
}

// ---------------------------------------------------------------------------
// Markdown rendering
// ---------------------------------------------------------------------------

/// Renders a combined review report as Markdown.
///
/// The output includes a mandatory **Non-Judgment Boundary** section stating
/// that this report does not approve, reject, or issue safety verdicts.
pub fn render_combined_review_markdown(report: &RdeCombinedReviewReport) -> String {
    let mut md = String::new();

    // Header
    md.push_str(&format!(
        "# RDE Combined Review Report\n\n**Report ID:** `{}`\n\n",
        report.report_id
    ));

    // Evidence Bindings
    md.push_str("## Evidence Bindings\n\n");
    if report.evidence_report.bindings.is_empty() {
        md.push_str("*No evidence bindings were produced.*\n\n");
    } else {
        for binding in &report.evidence_report.bindings {
            md.push_str(&format!(
                "- **`{}`** — `{}`\n",
                binding.item_id,
                binding.classification.key()
            ));
            if !binding.evidence_refs.is_empty() {
                md.push_str("  - Evidence:\n");
                for eref in &binding.evidence_refs {
                    md.push_str(&format!("    - `{}`", eref.source_id));
                    if let Some(q) = &eref.quote {
                        md.push_str(&format!(" — \"{}\"", q));
                    }
                    md.push('\n');
                }
            }
            if !binding.uncertainty_notes.is_empty() {
                for note in &binding.uncertainty_notes {
                    md.push_str(&format!("  - ⚠ {}\n", note));
                }
            }
            md.push('\n');
        }
    }

    // Meta-RDE Findings
    md.push_str("## Meta-RDE Findings\n\n");
    let finding_categories = [
        ("Preserved", &report.meta_report.preserved),
        (
            "Authorized Transformations",
            &report.meta_report.authorized_transformations,
        ),
        (
            "Inferred Extensions",
            &report.meta_report.inferred_extensions,
        ),
        ("Suspicious Drifts", &report.meta_report.suspicious_drifts),
        (
            "Critical Distortions",
            &report.meta_report.critical_distortions,
        ),
        ("Unresolved", &report.meta_report.unresolved),
    ];

    let mut has_findings = false;
    for (label, findings) in &finding_categories {
        if !findings.is_empty() {
            has_findings = true;
            md.push_str(&format!("### {}\n\n", label));
            for f in *findings {
                let severity_label = match f.severity {
                    crate::meta_rde::MetaRdeSeverity::Informational => "ℹ",
                    crate::meta_rde::MetaRdeSeverity::ReviewNeeded => "👁",
                    crate::meta_rde::MetaRdeSeverity::HighRisk => "⚠",
                    crate::meta_rde::MetaRdeSeverity::Blocking => "⛔",
                };
                md.push_str(&format!(
                    "- {} **{}**: {}\n",
                    severity_label, f.finding_id, f.summary
                ));
                if !f.reviewer_focus.is_empty() {
                    for focus in &f.reviewer_focus {
                        md.push_str(&format!("  - → {}\n", focus));
                    }
                }
            }
            md.push('\n');
        }
    }
    if !has_findings {
        md.push_str("*No Meta-RDE findings were produced.*\n\n");
    }

    // Reviewer Focus
    md.push_str("## Reviewer Focus\n\n");
    if report.reviewer_summary.is_empty() {
        md.push_str("*No reviewer focus items.*\n\n");
    } else {
        for item in &report.reviewer_summary {
            md.push_str(&format!("- {}\n", item));
        }
        md.push('\n');
    }

    // Unresolved
    md.push_str("## Unresolved\n\n");
    if report.unresolved.is_empty() {
        md.push_str("*No unresolved items.*\n\n");
    } else {
        for item in &report.unresolved {
            md.push_str(&format!("- {}\n", item));
        }
        md.push('\n');
    }

    // Next Update Policy
    md.push_str("## Next Update Policy\n\n");
    if report.next_update_policy.is_empty() {
        md.push_str("*No next update policy items.*\n\n");
    } else {
        for item in &report.next_update_policy {
            md.push_str(&format!("- {}\n", item));
        }
        md.push('\n');
    }

    // Non-Judgment Boundary (mandatory)
    md.push_str("## Non-Judgment Boundary\n\n");
    md.push_str("**This report does not approve, reject, or issue a safety verdict.**\n\n");
    md.push_str("It is a review artifact for human evaluation. All classifications,\n");
    md.push_str("findings, and reviewer focus items are structured observations, not final\n");
    md.push_str(
        "decisions. Final judgment — approval, rejection, revision, or acceptance — belongs\n",
    );
    md.push_str("to the human reviewer.\n\n");
    md.push_str(
        "*このレポートは承認・却下・安全判定を行わない。人間の確認のための review artifact である。*\n",
    );

    md
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta_rde::{run_meta_rde_minimal, MetaRdeReport};
    use crate::rde_evidence::{bind_evidence_minimal, EvidenceBindingReport};
    use crate::rde_impl::{RdeCategory, RdeEvaluation, RdeObservation};

    // ── helpers ──────────────────────────────────────────────────────────

    fn sample_evidence_report() -> EvidenceBindingReport {
        let mut eval = RdeEvaluation::new("s/1");
        eval.push(RdeObservation {
            category: RdeCategory::Preserved,
            summary: "kept".to_string(),
            evidence_refs: vec!["ref:1".to_string()],
            confidence_note: None,
        });
        eval.push(RdeObservation {
            category: RdeCategory::Transformed,
            summary: "changed".to_string(),
            evidence_refs: vec![],
            confidence_note: None,
        });
        bind_evidence_minimal(&eval)
    }

    fn sample_meta_report(evidence: &EvidenceBindingReport) -> MetaRdeReport {
        run_meta_rde_minimal(evidence)
    }

    // ── Preserves evidence and meta reports ─────────────────────────────

    #[test]
    fn combined_report_preserves_evidence_and_meta_reports() {
        let evidence = sample_evidence_report();
        let meta = sample_meta_report(&evidence);

        let combined = build_combined_review_report("test/1", evidence.clone(), meta.clone());

        // Evidence bindings preserved
        assert_eq!(
            combined.evidence_report.bindings.len(),
            evidence.bindings.len()
        );
        assert_eq!(
            combined.evidence_report.bindings[0].classification,
            evidence.bindings[0].classification
        );

        // Meta-RDE target_id preserved
        assert_eq!(combined.meta_report.target_id, meta.target_id);
    }

    // ── No approval/rejection ───────────────────────────────────────────

    #[test]
    fn combined_report_does_not_generate_approval_or_rejection() {
        let evidence = sample_evidence_report();
        let meta = sample_meta_report(&evidence);

        let combined = build_combined_review_report("test/2", evidence, meta);

        // Check struct fields: no approval/rejection fields exist
        let fields = [
            combined.report_id.clone(),
            combined.reviewer_summary.join(" "),
            combined.unresolved.join(" "),
            combined.next_update_policy.join(" "),
        ]
        .join(" ");

        let forbidden = [
            "approved", "rejected", "accepted", "denied", "safe", "unsafe",
        ];
        for word in forbidden {
            assert!(
                !fields.to_lowercase().contains(word),
                "combined report must not contain {word:?}"
            );
        }
    }

    // ── Markdown contains non-judgment boundary ────────────────────────

    #[test]
    fn combined_report_markdown_contains_non_judgment_boundary() {
        let evidence = sample_evidence_report();
        let meta = sample_meta_report(&evidence);
        let combined = build_combined_review_report("test/3", evidence, meta);

        let md = render_combined_review_markdown(&combined);

        assert!(
            md.contains("Non-Judgment Boundary"),
            "markdown must contain Non-Judgment Boundary section"
        );
        assert!(
            md.contains("does not approve, reject, or issue a safety verdict"),
            "markdown must state non-judgment nature"
        );
        assert!(
            md.contains("review artifact"),
            "markdown must identify itself as review artifact"
        );
    }

    // ── Collects unresolved and next_update_policy ─────────────────────

    #[test]
    fn combined_report_collects_unresolved_and_next_update_policy() {
        let evidence = sample_evidence_report();
        let meta = sample_meta_report(&evidence);
        let combined = build_combined_review_report("test/4", evidence, meta);

        // next_update_policy from Meta-RDE should be collected
        assert!(
            !combined.next_update_policy.is_empty(),
            "must collect next_update_policy from meta report"
        );

        // Markdown should include both sections
        let md = render_combined_review_markdown(&combined);
        assert!(md.contains("Unresolved"));
        assert!(md.contains("Next Update Policy"));
    }

    // ── Handles empty reports conservatively ───────────────────────────

    #[test]
    fn combined_report_handles_empty_reports_conservatively() {
        let evidence = EvidenceBindingReport::default();
        let meta = MetaRdeReport::default();
        let combined = build_combined_review_report("test/5", evidence, meta);

        // Should not panic; should produce a valid report
        assert_eq!(combined.report_id, "test/5");
        assert!(combined.reviewer_summary.is_empty());

        // Markdown should render without panic
        let md = render_combined_review_markdown(&combined);
        assert!(md.contains("RDE Combined Review Report"));
        assert!(md.contains("Non-Judgment Boundary"));
    }
}
