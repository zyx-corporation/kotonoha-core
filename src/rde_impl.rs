//! RDE implementation profile aligned with `kotonoha-spec` **SLS-5**.
//!
//! This module does **not** prescribe a particular LLM, prompt, model, latency
//! target, or deployment topology. It provides a small implementation scaffold
//! that makes SLS-5 responsibilities explicit in `kotonoha-core`:
//!
//! - subject intake (`RdeSubject`),
//! - context assembly (`RdeContext`),
//! - category classification (`RdeCategory`),
//! - output emission (`RdeEvaluation::to_json_value`),
//! - output validation (`RdeEvaluation::validate`), and
//! - traceability (`RdeTraceability`).
//!
//! RDE remains a semantic review record producer. It is not a policy engine,
//! safety filter, approval authority, or replacement for human accountability.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// SLS-4 / SLS-5 category keys used by RDE review outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RdeCategory {
    Preserved,
    Transformed,
    Complemented,
    IntentionallyUnresolved,
    Lost,
    DeviationRisk,
    NextUpdatePolicy,
}

impl RdeCategory {
    /// Returns the normative category key used in SLS-4 interchange records.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Preserved => "preserved",
            Self::Transformed => "transformed",
            Self::Complemented => "complemented",
            Self::IntentionallyUnresolved => "intentionally_unresolved",
            Self::Lost => "lost",
            Self::DeviationRisk => "deviation_risk",
            Self::NextUpdatePolicy => "next_update_policy",
        }
    }

    /// Returns all normative category keys in stable output order.
    pub const fn all() -> [Self; 7] {
        [
            Self::Preserved,
            Self::Transformed,
            Self::Complemented,
            Self::IntentionallyUnresolved,
            Self::Lost,
            Self::DeviationRisk,
            Self::NextUpdatePolicy,
        ]
    }
}

/// Subject adapter output for SLS-5.3.2 / SLS-5.4.1.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RdeSubject {
    /// Stable subject reference used as `subject_ref` in SLS-4 output.
    pub subject_ref: String,
    /// Optional source or prior text/material.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
    /// Optional changed/proposed text/material.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_text: Option<String>,
    /// Optional source references, such as commit, issue, PR, document, or span references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

impl RdeSubject {
    /// Creates a subject with a required non-empty subject reference.
    pub fn new(subject_ref: impl Into<String>) -> Self {
        Self {
            subject_ref: subject_ref.into(),
            ..Self::default()
        }
    }

    /// Validates SLS-5.5.1 minimum input.
    pub fn validate(&self) -> Result<(), String> {
        if self.subject_ref.trim().is_empty() {
            return Err("RdeSubject.subject_ref must be non-empty (SLS-5.5.1)".to_string());
        }
        Ok(())
    }
}

/// Context provider output for SLS-5.3.3 / SLS-5.4.2.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RdeContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_lineage_unit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_rde_output_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_correlation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub human_context_refs: Vec<String>,
}

impl RdeContext {
    /// True when no prior/context references are available.
    pub fn is_empty_context(&self) -> bool {
        self.prior_lineage_unit_id.is_none()
            && self.prior_rde_output_ref.is_none()
            && self.audit_correlation_id.is_none()
            && self.human_context_refs.is_empty()
    }
}

/// A single categorized RDE observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RdeObservation {
    pub category: RdeCategory,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence_note: Option<String>,
}

impl RdeObservation {
    pub fn new(category: RdeCategory, summary: impl Into<String>) -> Self {
        Self {
            category,
            summary: summary.into(),
            evidence_refs: Vec::new(),
            confidence_note: None,
        }
    }

    fn to_item_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("summary".to_string(), Value::String(self.summary.clone()));
        if !self.evidence_refs.is_empty() {
            obj.insert("evidence_refs".to_string(), json!(self.evidence_refs));
        }
        if let Some(note) = &self.confidence_note {
            obj.insert("confidence_note".to_string(), Value::String(note.clone()));
        }
        Value::Object(obj)
    }
}

/// Traceability fields corresponding to SLS-5.4.6, SLS-5.7, and SLS-5.8.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RdeTraceability {
    pub subject_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_lineage_unit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_rde_output_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_correlation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

/// Evaluation result emitted by an RDE implementation profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RdeEvaluation {
    pub subject_ref: String,
    #[serde(default)]
    pub observations: Vec<RdeObservation>,
    #[serde(default)]
    pub traceability: RdeTraceability,
}

impl RdeEvaluation {
    pub fn new(subject_ref: impl Into<String>) -> Self {
        let subject_ref = subject_ref.into();
        Self {
            traceability: RdeTraceability {
                subject_ref: subject_ref.clone(),
                ..RdeTraceability::default()
            },
            subject_ref,
            observations: Vec::new(),
        }
    }

    pub fn push(&mut self, observation: RdeObservation) {
        self.observations.push(observation);
    }

    /// Emits the SLS-4-compatible `rde_review_output` JSON shape.
    pub fn to_json_value(&self) -> Value {
        let mut categories: BTreeMap<&'static str, Vec<Value>> = BTreeMap::new();
        for category in RdeCategory::all() {
            categories.insert(category.key(), Vec::new());
        }

        for observation in &self.observations {
            categories
                .entry(observation.category.key())
                .or_default()
                .push(observation.to_item_json());
        }

        json!({
            "rde_review_output": {
                "spec_version": crate::TARGET_SPEC_BUNDLE,
                "subject_ref": self.subject_ref,
                "categories": categories
            }
        })
    }

    /// Serializes the SLS-4-compatible output to compact JSON text.
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string(&self.to_json_value()).map_err(|e| format!("RDE serialization: {e}"))
    }

    /// Validates the emitted output through the existing SLS-4 validator.
    pub fn validate(&self, strict: bool) -> Result<Vec<String>, String> {
        let text = self.to_json_string()?;
        crate::rde::validate_json(&text, strict)
    }
}

/// RDE evaluator boundary from SLS-5.3.1.
pub trait RdeEvaluator {
    fn evaluate(&self, subject: &RdeSubject, context: &RdeContext)
        -> Result<RdeEvaluation, String>;
}

/// A deterministic minimal evaluator useful for tests, demos, and integration scaffolding.
///
/// This evaluator intentionally does **not** claim deep semantic understanding.
/// It classifies obvious structural situations and records uncertainty so that
/// SLS-5 boundaries can be tested without binding the crate to a model provider.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConservativeRdeEvaluator;

impl RdeEvaluator for ConservativeRdeEvaluator {
    fn evaluate(
        &self,
        subject: &RdeSubject,
        context: &RdeContext,
    ) -> Result<RdeEvaluation, String> {
        subject.validate()?;

        let mut evaluation = RdeEvaluation::new(subject.subject_ref.clone());
        evaluation.traceability = RdeTraceability {
            subject_ref: subject.subject_ref.clone(),
            related_lineage_unit_id: context.prior_lineage_unit_id.clone(),
            prior_rde_output_ref: context.prior_rde_output_ref.clone(),
            audit_correlation_id: context.audit_correlation_id.clone(),
            source_refs: subject.source_refs.clone(),
        };

        match (&subject.source_text, &subject.changed_text) {
            (Some(source), Some(changed)) if source == changed => {
                evaluation.push(RdeObservation::new(
                    RdeCategory::Preserved,
                    "source and changed material are textually identical; no material semantic change is inferred by the conservative evaluator",
                ));
            }
            (Some(source), Some(changed))
                if source.trim().is_empty() && !changed.trim().is_empty() =>
            {
                evaluation.push(RdeObservation::new(
                    RdeCategory::Complemented,
                    "changed material adds content where the supplied source material is empty",
                ));
            }
            (Some(source), Some(changed))
                if !source.trim().is_empty() && changed.trim().is_empty() =>
            {
                evaluation.push(RdeObservation::new(
                    RdeCategory::Lost,
                    "changed material is empty while source material was present; potential semantic loss should be reviewed",
                ));
            }
            (Some(_), Some(_)) => {
                evaluation.push(RdeObservation::new(
                    RdeCategory::Transformed,
                    "source and changed material differ; conservative evaluator records this as a transformation requiring review",
                ));
            }
            (None, Some(_)) => {
                evaluation.push(RdeObservation::new(
                    RdeCategory::IntentionallyUnresolved,
                    "prior source material is unavailable; preservation, transformation, and loss cannot be fully characterized",
                ));
            }
            (Some(_), None) => {
                evaluation.push(RdeObservation::new(
                    RdeCategory::IntentionallyUnresolved,
                    "changed material is unavailable; RDE review cannot determine the resulting semantic state",
                ));
            }
            (None, None) => {
                evaluation.push(RdeObservation::new(
                    RdeCategory::IntentionallyUnresolved,
                    "neither source nor changed material is available; only subject traceability can be recorded",
                ));
            }
        }

        if context.is_empty_context() {
            evaluation.push(RdeObservation::new(
                RdeCategory::DeviationRisk,
                "no prior lineage, prior RDE output, audit correlation, or human context reference was supplied; review quality may be limited",
            ));
        }

        evaluation.push(RdeObservation::new(
            RdeCategory::NextUpdatePolicy,
            "human reviewer should confirm whether the conservative classification matches project intent before approval or publication",
        ));

        Ok(evaluation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_keys_match_sls4() {
        let keys: Vec<&str> = RdeCategory::all().iter().map(|c| c.key()).collect();
        assert_eq!(
            keys,
            vec![
                "preserved",
                "transformed",
                "complemented",
                "intentionally_unresolved",
                "lost",
                "deviation_risk",
                "next_update_policy"
            ]
        );
    }

    #[test]
    fn subject_ref_is_required() {
        let subject = RdeSubject::new("   ");
        assert!(subject.validate().is_err());
    }

    #[test]
    fn conservative_evaluator_emits_valid_sls4_output() {
        let mut subject = RdeSubject::new("https://example.invalid/subject/1");
        subject.source_text = Some("original intent".to_string());
        subject.changed_text = Some("changed intent".to_string());
        subject.source_refs.push("issue:1".to_string());

        let context = RdeContext {
            prior_lineage_unit_id: Some("lineage:0".to_string()),
            prior_rde_output_ref: Some("rde:0".to_string()),
            audit_correlation_id: Some("audit:1".to_string()),
            human_context_refs: vec!["review-note:1".to_string()],
        };

        let evaluation = ConservativeRdeEvaluator
            .evaluate(&subject, &context)
            .expect("evaluation should succeed");
        assert!(evaluation.validate(true).unwrap().is_empty());
        assert_eq!(
            evaluation.traceability.audit_correlation_id.as_deref(),
            Some("audit:1")
        );
    }

    #[test]
    fn conservative_evaluator_records_loss_for_empty_changed_material() {
        let mut subject = RdeSubject::new("https://example.invalid/subject/loss");
        subject.source_text = Some("must not be lost".to_string());
        subject.changed_text = Some("".to_string());

        let evaluation = ConservativeRdeEvaluator
            .evaluate(&subject, &RdeContext::default())
            .expect("evaluation should succeed");

        assert!(evaluation
            .observations
            .iter()
            .any(|o| o.category == RdeCategory::Lost));
        assert!(evaluation.validate(true).unwrap().is_empty());
    }

    #[test]
    fn output_contains_all_categories_even_when_empty() {
        let evaluation = RdeEvaluation::new("https://example.invalid/subject/categories");
        let json = evaluation.to_json_value();
        let categories = json["rde_review_output"]["categories"]
            .as_object()
            .expect("categories object");

        for category in RdeCategory::all() {
            assert!(categories.contains_key(category.key()));
        }
    }
}
