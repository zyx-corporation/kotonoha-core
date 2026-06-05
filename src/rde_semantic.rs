//! RDE semantic extraction scaffold — Phase B of the full RDE implementation roadmap.
//!
//! This module introduces the structural scaffolding for meaning-bearing elements
//! in RDE evaluation. It does **not** claim complete semantic understanding.
//! Instead, it provides typed containers so that later ΔM analysis can compare how
//! intent, constraints, risks, responsibilities, unresolved questions, and
//! value/factual claims move across revisions.
//!
//! # Non-goals
//!
//! - No LLM integration.
//! - No policy enforcement.
//! - No final approval/rejection.
//! - No full ΔM relation model.
//! - No model dependency.
//!
//! # Human authority boundary
//!
//! RDE output produced through this module remains a structured observation record.
//! It does **not** replace human judgment, approval, rejection, publication
//! responsibility, or institutional accountability.
//!
//! Trait extensibility:
//!
//! - `SemanticExtractor` is a trait boundary so rule-based, model-assisted, and
//!   human-curated extractors can later implement the same contract.
//! - The core remains model-agnostic.

use serde::{Deserialize, Serialize};

use crate::rde_impl::RdeSubject;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by RDE semantic extraction and related pipeline stages.
///
/// This is intentionally minimal for Phase B. Additional variants for ΔM
/// analysis, classification, and evidence binding can be added in later phases
/// without breaking the existing `SemanticExtractor` contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RdeError {
    /// Extraction phase failure — missing input, unreadable material, etc.
    Extraction(String),
    /// Validation phase failure — malformed structure, constraint violation, etc.
    Validation(String),
}

impl std::fmt::Display for RdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Extraction(msg) => write!(f, "extraction error: {msg}"),
            Self::Validation(msg) => write!(f, "validation error: {msg}"),
        }
    }
}

impl std::error::Error for RdeError {}

// ---------------------------------------------------------------------------
// RdeContextBundle — SLS-5.3.3 / SLS-5.4.2 context assembly
// ---------------------------------------------------------------------------

/// Bundled review context assembled before semantic extraction.
///
/// Corresponds to the **Input layer** in `docs/full-rde-implementation-roadmap.md`
/// §5.1. It wraps a review subject together with human-supplied context so that
/// extractors do not collapse the review subject into raw text diff alone.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RdeContextBundle {
    /// The review subject.
    pub subject: RdeSubject,
    /// Stated design or change intent, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_intent: Option<String>,
    /// Explicitly declared non-goals for the change.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_goals: Vec<String>,
    /// Elements that must not be lost or weakened.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub must_not_lose: Vec<String>,
    /// Sections of a relevant specification or governing document.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_spec_sections: Vec<String>,
    /// References to prior RDE review output records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prior_rde_outputs: Vec<String>,
    /// Audit correlation references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit_refs: Vec<String>,
    /// Human-supplied review notes (not pre-approved decisions).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub human_review_notes: Vec<String>,
}

impl RdeContextBundle {
    /// Creates a bundle for the given subject with no additional context.
    pub fn new(subject: RdeSubject) -> Self {
        Self {
            subject,
            ..Self::default()
        }
    }

    /// Validates that the minimum required input is present (SLS-5.5.1).
    pub fn validate_minimum_input(&self) -> Result<(), RdeError> {
        self.subject
            .validate()
            .map_err(|msg| RdeError::Validation(format!("subject validation: {msg}")))?;
        if self.subject.subject_ref.trim().is_empty() {
            return Err(RdeError::Validation(
                "RdeContextBundle.subject.subject_ref must be non-empty (SLS-5.5.1)".to_string(),
            ));
        }
        Ok(())
    }

    /// Returns true when no auxiliary context (intent, non-goals, must-not-lose, …) is supplied.
    pub fn is_minimal_context(&self) -> bool {
        self.source_intent.is_none()
            && self.non_goals.is_empty()
            && self.must_not_lose.is_empty()
            && self.related_spec_sections.is_empty()
            && self.prior_rde_outputs.is_empty()
            && self.audit_refs.is_empty()
            && self.human_review_notes.is_empty()
    }
}

// ---------------------------------------------------------------------------
// SemanticElementKind — SLS-5.4.3 semantic observation categories
// ---------------------------------------------------------------------------

/// Kinds of meaning-bearing elements that a semantic extractor may surface.
///
/// This enum is **not** a policy verdict. It labels what kind of element was
/// observed; it does not declare the element safe, approved, or rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticElementKind {
    /// Stated design or change intent.
    Intent,
    /// Constraint or boundary that must be respected.
    Constraint,
    /// Assumption that underpins the current state or change.
    Assumption,
    /// Identified risk or exposure.
    Risk,
    /// Responsibility or accountability boundary.
    Responsibility,
    /// Open question or unresolved tension.
    UnresolvedQuestion,
    /// Claim about value, priority, or institutional importance.
    ValueClaim,
    /// Claim of fact about the system, domain, or context.
    FactualClaim,
}

impl SemanticElementKind {
    /// Returns a stable lowercase key for each variant, suitable for JSON serialization.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Intent => "intent",
            Self::Constraint => "constraint",
            Self::Assumption => "assumption",
            Self::Risk => "risk",
            Self::Responsibility => "responsibility",
            Self::UnresolvedQuestion => "unresolved_question",
            Self::ValueClaim => "value_claim",
            Self::FactualClaim => "factual_claim",
        }
    }
}

// ---------------------------------------------------------------------------
// SemanticElement — a single extracted meaning-bearing element
// ---------------------------------------------------------------------------

/// A single meaning-bearing element extracted from review context.
///
/// This struct does **not** carry an approval/rejection verdict, a safety
/// classification, or an access-control decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticElement {
    /// Unique identifier within the extraction (not globally unique).
    pub id: String,
    /// What kind of element was observed.
    pub kind: SemanticElementKind,
    /// The element text, as it appears or is summarized.
    pub text: String,
    /// Reference to the source material where this element was observed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    /// Human-readable note about extraction confidence or uncertainty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence_note: Option<String>,
    /// Scope qualifier, e.g. "this change", "module X", "project-wide".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

// ---------------------------------------------------------------------------
// SemanticExtraction — a collection of elements for one subject
// ---------------------------------------------------------------------------

/// Result of extracting semantic elements from a review context bundle.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SemanticExtraction {
    /// The subject reference this extraction is about.
    pub subject_ref: String,
    /// All extracted meaning-bearing elements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<SemanticElement>,
}

impl SemanticExtraction {
    /// Creates an empty extraction for the given subject reference.
    pub fn new(subject_ref: impl Into<String>) -> Self {
        Self {
            subject_ref: subject_ref.into(),
            elements: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// SemanticExtractor trait — SLS-5.4.3 / Layer 2 (semantic layer)
// ---------------------------------------------------------------------------

/// Contract for components that produce `SemanticExtraction` from a context bundle.
///
/// Implementations may be rule-based, model-assisted, or human-curated. The trait
/// does **not** mandate a specific model provider, prompt format, or algorithm.
///
/// # Human authority boundary
///
/// Implementations **MUST NOT** emit approval, rejection, safety verdicts, or
/// access-control decisions through this trait. Extraction is observation, not
/// decision.
pub trait SemanticExtractor {
    /// Extracts meaning-bearing elements from the supplied context bundle.
    ///
    /// Returns `SemanticExtraction` on success. An empty `elements` list is a
    /// valid result when no material elements could be identified.
    fn extract(&self, context: &RdeContextBundle) -> Result<SemanticExtraction, RdeError>;
}

// ---------------------------------------------------------------------------
// ConservativeSemanticExtractor — rule-based, no model dependency
// ---------------------------------------------------------------------------

/// A deterministic, conservative extractor that maps supplied context fields
/// into `SemanticElement` observations without deep semantic analysis.
///
/// This extractor is intentionally lightweight:
///
/// - `source_intent` → `Intent` elements.
/// - `non_goals` → `Constraint` elements.
/// - `must_not_lose` → `Constraint` elements (conservative; may later be
///   reclassified as `Responsibility`).
/// - `human_review_notes` → `UnresolvedQuestion` elements — but the extractor
///   does **not** treat them as closed or resolved.
///
/// It performs **no** text analysis, **no** LLM calls, and **no** approval or
/// rejection. It merely lifts already-structured context into typed containers
/// so that later pipeline stages (ΔM analysis, RDE classification) have
/// inspectable, versionable input.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConservativeSemanticExtractor;

impl SemanticExtractor for ConservativeSemanticExtractor {
    fn extract(&self, context: &RdeContextBundle) -> Result<SemanticExtraction, RdeError> {
        context.validate_minimum_input()?;

        let subject_ref = context.subject.subject_ref.clone();
        let mut extraction = SemanticExtraction::new(&subject_ref);
        let mut element_counter: u64 = 0;

        let mut next_id = || {
            element_counter += 1;
            format!("{subject_ref}/element/{element_counter}")
        };

        // source_intent → Intent
        if let Some(intent) = &context.source_intent {
            extraction.elements.push(SemanticElement {
                id: next_id(),
                kind: SemanticElementKind::Intent,
                text: intent.clone(),
                source_ref: Some("context.source_intent".to_string()),
                confidence_note: Some("supplied directly; not verified by extractor".to_string()),
                scope: None,
            });
        }

        // non_goals → Constraint
        for (i, goal) in context.non_goals.iter().enumerate() {
            extraction.elements.push(SemanticElement {
                id: next_id(),
                kind: SemanticElementKind::Constraint,
                text: goal.clone(),
                source_ref: Some(format!("context.non_goals[{}]", i)),
                confidence_note: Some(
                    "declared non-goal; constraint boundary is author-supplied".to_string(),
                ),
                scope: None,
            });
        }

        // must_not_lose → Constraint (conservative; may later be Responsibility)
        for (i, item) in context.must_not_lose.iter().enumerate() {
            extraction.elements.push(SemanticElement {
                id: next_id(),
                kind: SemanticElementKind::Constraint,
                text: item.clone(),
                source_ref: Some(format!("context.must_not_lose[{}]", i)),
                confidence_note: Some(
                    "declared must-not-lose; treated conservatively as constraint".to_string(),
                ),
                scope: None,
            });
        }

        // human_review_notes → UnresolvedQuestion
        // Deliberately NOT treated as resolved or approved.
        for (i, note) in context.human_review_notes.iter().enumerate() {
            extraction.elements.push(SemanticElement {
                id: next_id(),
                kind: SemanticElementKind::UnresolvedQuestion,
                text: note.clone(),
                source_ref: Some(format!("context.human_review_notes[{}]", i)),
                confidence_note: Some(
                    "human-supplied note; not closed or resolved by extractor".to_string(),
                ),
                scope: None,
            });
        }

        // When context is minimal and no elements were produced, add a
        // single element noting the lack of material.
        if extraction.elements.is_empty() {
            extraction.elements.push(SemanticElement {
                id: next_id(),
                kind: SemanticElementKind::UnresolvedQuestion,
                text: "no structured context (intent, constraints, must-not-lose, or review notes) was supplied; semantic extraction is minimal".to_string(),
                source_ref: None,
                confidence_note: Some("empty context; no material elements observed".to_string()),
                scope: None,
            });
        }

        Ok(extraction)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────

    fn subject_with_ref(r: &str) -> RdeSubject {
        RdeSubject::new(r.to_string())
    }

    fn make_extractor() -> ConservativeSemanticExtractor {
        ConservativeSemanticExtractor
    }

    // ── RdeContextBundle construction ────────────────────────────────────

    #[test]
    fn can_construct_context_bundle() {
        let s = subject_with_ref("https://example.invalid/subject/1");
        let bundle = RdeContextBundle::new(s);
        assert_eq!(
            bundle.subject.subject_ref,
            "https://example.invalid/subject/1"
        );
        assert!(bundle.is_minimal_context());
    }

    // ── SemanticElement (de)serialization ────────────────────────────────

    #[test]
    fn semantic_element_roundtrips_json() {
        let elem = SemanticElement {
            id: "s/1/e/1".to_string(),
            kind: SemanticElementKind::Intent,
            text: "preserve original scope".to_string(),
            source_ref: Some("context.source_intent".to_string()),
            confidence_note: Some("supplied directly".to_string()),
            scope: Some("this change".to_string()),
        };

        let json = serde_json::to_string(&elem).expect("serialize");
        let back: SemanticElement = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(back.id, "s/1/e/1");
        assert_eq!(back.kind, SemanticElementKind::Intent);
        assert_eq!(back.text, "preserve original scope");
        assert_eq!(back.source_ref.as_deref(), Some("context.source_intent"));
        assert_eq!(back.confidence_note.as_deref(), Some("supplied directly"));
        assert_eq!(back.scope.as_deref(), Some("this change"));
    }

    // ── Extractor: source_intent → Intent ────────────────────────────────

    #[test]
    fn extractor_maps_source_intent_to_intent_element() {
        let mut bundle =
            RdeContextBundle::new(subject_with_ref("https://example.invalid/subject/intent"));
        bundle.source_intent = Some("Keep the public API stable".to_string());

        let extraction = make_extractor()
            .extract(&bundle)
            .expect("extraction succeeds");

        let intent_elems: Vec<&SemanticElement> = extraction
            .elements
            .iter()
            .filter(|e| e.kind == SemanticElementKind::Intent)
            .collect();
        assert_eq!(intent_elems.len(), 1);
        assert_eq!(intent_elems[0].text, "Keep the public API stable");
        assert_eq!(
            intent_elems[0].source_ref.as_deref(),
            Some("context.source_intent")
        );
    }

    // ── Extractor: must_not_lose is preserved ───────────────────────────

    #[test]
    fn extractor_preserves_must_not_lose_as_constraint() {
        let mut bundle =
            RdeContextBundle::new(subject_with_ref("https://example.invalid/subject/mnl"));
        bundle.must_not_lose = vec![
            "backward compatibility with v1 clients".to_string(),
            "error message format".to_string(),
        ];

        let extraction = make_extractor()
            .extract(&bundle)
            .expect("extraction succeeds");

        let constraint_elems: Vec<&SemanticElement> = extraction
            .elements
            .iter()
            .filter(|e| e.kind == SemanticElementKind::Constraint)
            .collect();
        assert_eq!(constraint_elems.len(), 2);
        let texts: Vec<&str> = constraint_elems.iter().map(|e| e.text.as_str()).collect();
        assert!(texts.contains(&"backward compatibility with v1 clients"));
        assert!(texts.contains(&"error message format"));
    }

    // ── Extractor: empty context does not panic ─────────────────────────

    #[test]
    fn extractor_handles_empty_context_without_panic() {
        let bundle =
            RdeContextBundle::new(subject_with_ref("https://example.invalid/subject/empty"));
        let extraction = make_extractor()
            .extract(&bundle)
            .expect("extraction succeeds");

        // An empty-context extraction should produce at least the
        // minimal "no structured context" element and no panic.
        assert!(!extraction.elements.is_empty());
        assert_eq!(
            extraction.elements[0].kind,
            SemanticElementKind::UnresolvedQuestion
        );
    }

    // ── Extractor: no approval/rejection/safety verdicts ────────────────

    #[test]
    fn extraction_contains_no_approval_or_safety_verdicts() {
        let mut bundle =
            RdeContextBundle::new(subject_with_ref("https://example.invalid/subject/safe"));
        bundle.source_intent = Some("Add caching layer".to_string());
        bundle.must_not_lose = vec!["data integrity".to_string()];

        let extraction = make_extractor()
            .extract(&bundle)
            .expect("extraction succeeds");

        let all_text: String = extraction
            .elements
            .iter()
            .map(|e| &e.text)
            .chain(
                extraction
                    .elements
                    .iter()
                    .filter_map(|e| e.confidence_note.as_ref()),
            )
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
        ];
        for word in forbidden {
            assert!(
                !all_text.to_lowercase().contains(word),
                "extraction must not contain verdict word {word:?}"
            );
        }
    }

    // ── Extractor: missing subject_ref is rejected ──────────────────────

    #[test]
    fn extractor_rejects_empty_subject_ref() {
        let mut bundle = RdeContextBundle::new(subject_with_ref("   "));
        bundle.source_intent = Some("something".to_string());

        let result = make_extractor().extract(&bundle);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(&err, RdeError::Validation(_)),
            "expected Validation error, got {err:?}"
        );
    }

    // ── RdeError Display ────────────────────────────────────────────────

    #[test]
    fn rde_error_display_is_human_readable() {
        let e = RdeError::Extraction("no material".to_string());
        assert!(e.to_string().contains("extraction error"));
        assert!(e.to_string().contains("no material"));

        let e = RdeError::Validation("bad subject".to_string());
        assert!(e.to_string().contains("validation error"));
        assert!(e.to_string().contains("bad subject"));
    }
}
