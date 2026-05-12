# Roadmap toward a full RDE implementation

Status: **design roadmap / non-normative implementation guidance**

Tracked by: [zyx-corporation/kotonoha-core#14](https://github.com/zyx-corporation/kotonoha-core/issues/14)

Source specification: [`kotonoha-spec` SLS-5 RDE implementation specification](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-implementation-specification.md)

## 1. Purpose

This document describes how `kotonoha-core` should evolve from the current **SLS-5 implementation scaffold** toward a fuller RDE implementation.

The current implementation provides explicit boundaries and minimal deterministic behavior. It does **not** claim complete semantic understanding. A full RDE should be understood as a **semantic-change audit structure**, not merely a smarter evaluator.

RDE must remain:

- a producer and validator of semantic review observations;
- grounded in traceable evidence;
- connected to lineage and audit records;
- separate from policy enforcement;
- separate from final human approval or rejection.

## 2. Current state

The current `kotonoha-core` scaffold provides:

| Component | Current implementation | Role |
| --- | --- | --- |
| Subject adapter | `RdeSubject` | Holds `subject_ref`, optional source/changed text, and source refs. |
| Context provider | `RdeContext` | Holds prior lineage, prior RDE output, audit correlation, and human context refs. |
| Evaluator boundary | `RdeEvaluator` | Trait for evaluation implementations. |
| Minimal evaluator | `ConservativeRdeEvaluator` | Deterministic scaffold for tests and demos; not deep semantic understanding. |
| Output object | `RdeEvaluation` | Emits SLS-4-compatible RDE review output. |
| Traceability | `RdeTraceability` | Carries subject, lineage, prior RDE, audit, and source references. |
| Validator | `RdeEvaluation::validate` | Reuses existing SLS-4 `crate::rde::validate_json`. |

This is enough to make SLS-5 responsibilities visible in code. It is not enough to make RDE complete.

## 3. Design principle

A full RDE is not an LLM wrapper.

LLMs or smaller language models may assist extraction, classification, or explanation, but RDE itself is the architecture that binds together:

- subject identity;
- prior context;
- semantic elements;
- ΔM analysis;
- category classification;
- evidence references;
- uncertainty notes;
- lineage persistence;
- audit correlation;
- human review handoff;
- meta-review of RDE output quality.

The core should remain model-agnostic. Model-backed evaluators should be plugins or adapters behind traits, not hard requirements of `kotonoha-core`.

## 4. Target layered architecture

A fuller RDE implementation should evolve toward the following layers.

[HTML rendering of this structure](full-rde-structure.html) is available for browser review. The source SVG is [`assets/full-rde-structure.svg`](assets/full-rde-structure.svg).

![Full RDE structure concept](assets/full-rde-structure.svg)

The SVG above is a conceptual diagram. It illustrates the intended implementation direction, but the normative source remains `kotonoha-spec` SLS-5 and the Rust contracts that this repository actually exposes.

```text
Input layer
  ├─ Subject Adapter
  ├─ Context Assembler
  └─ Source-of-truth Resolver

Semantic layer
  ├─ Semantic Extractor
  ├─ Claim / Intent / Constraint Parser
  ├─ Responsibility Detector
  └─ Unresolvedness Detector

ΔM layer
  ├─ Semantic Element Matcher
  ├─ Transformation Analyzer
  ├─ Loss Detector
  └─ Drift Risk Detector

RDE layer
  ├─ Category Classifier
  ├─ Evidence Binder
  ├─ Confidence / Uncertainty Notes
  └─ Next Update Policy Generator

Governance layer
  ├─ Human Review Handoff
  ├─ Audit Correlation
  ├─ Lineage Store Integration
  └─ Meta-RDE Quality Review
```

## 5. Layer responsibilities

### 5.1 Input layer

The input layer decides **what is being reviewed**.

It should normalize documents, patches, pull requests, generated text, design decisions, or lineage units into an RDE subject.

Future work:

```rust
pub struct RdeContextBundle {
    pub subject: RdeSubject,
    pub source_intent: Option<String>,
    pub non_goals: Vec<String>,
    pub must_not_lose: Vec<String>,
    pub related_spec_sections: Vec<String>,
    pub prior_rde_outputs: Vec<String>,
    pub audit_refs: Vec<String>,
    pub human_review_notes: Vec<String>,
}
```

The input layer must not collapse the review subject into raw text diff alone.

### 5.2 Semantic layer

The semantic layer extracts meaning-bearing elements from source and changed material.

Future work:

```rust
pub enum SemanticElementKind {
    Intent,
    Constraint,
    Assumption,
    Risk,
    Responsibility,
    UnresolvedQuestion,
    ValueClaim,
    FactualClaim,
}

pub struct SemanticElement {
    pub id: String,
    pub kind: SemanticElementKind,
    pub text: String,
    pub source_ref: Option<String>,
    pub confidence_note: Option<String>,
    pub scope: Option<String>,
}

pub struct SemanticExtraction {
    pub subject_ref: String,
    pub elements: Vec<SemanticElement>,
}
```

The semantic layer may be rule-based, model-assisted, or human-curated. The representation should not depend on one model vendor or prompt style.

### 5.3 ΔM layer

The ΔM layer compares prior and changed semantic states.

Future work:

```rust
pub enum DeltaMRelationKind {
    Preserved,
    Transformed,
    Complemented,
    Weakened,
    Removed,
    Split,
    Merged,
    Contradicted,
    Unresolved,
}

pub struct DeltaMRelation {
    pub source_element_id: Option<String>,
    pub target_element_id: Option<String>,
    pub relation: DeltaMRelationKind,
    pub summary: String,
    pub evidence_refs: Vec<String>,
}

pub struct DeltaMReport {
    pub subject_ref: String,
    pub relations: Vec<DeltaMRelation>,
}
```

This layer is where RDE becomes more than categorized commentary. It should show how meaning moved.

### 5.4 RDE classification layer

The classification layer maps ΔM relations into SLS-4 categories:

- `preserved`
- `transformed`
- `complemented`
- `intentionally_unresolved`
- `lost`
- `deviation_risk`
- `next_update_policy`

Future work:

```rust
pub trait RdeClassifier {
    fn classify(&self, report: &DeltaMReport) -> Result<RdeEvaluation, RdeError>;
}
```

Classification must remain evidence-linked. A category label without evidence is too close to a model opinion.

### 5.5 Evidence binder

The evidence binder connects observations to specific sources.

Evidence may include:

- document spans;
- patch hunks;
- issue comments;
- specification section identifiers;
- prior RDE observations;
- lineage unit identifiers;
- audit records;
- human review notes.

Future work:

```rust
pub struct EvidenceRef {
    pub kind: String,
    pub uri: Option<String>,
    pub label: Option<String>,
    pub span: Option<String>,
}
```

Evidence binding is required for trust. Without it, RDE output becomes impressionistic.

### 5.6 Governance layer

The governance layer hands RDE observations to humans and systems without pretending to decide for them.

It should output review focus, not approval.

Allowed:

- missing context warning;
- drift risk note;
- suggested reviewer focus;
- next update policy;
- audit correlation.

Not allowed as RDE output:

- approved;
- rejected;
- safe;
- unsafe;
- access granted;
- access denied.

Policy engines may consume RDE observations, but that is a separate layer.

### 5.7 Meta-RDE layer

The meta-RDE layer reviews RDE output quality.

It should detect:

- repetitive generic categories;
- weak or missing evidence;
- overclaiming;
- false closure;
- category misuse;
- missing loss analysis;
- missing uncertainty notes;
- failure to preserve human authority boundaries.

This is especially important if LLM-assisted evaluators are introduced.

## 6. Proposed trait architecture

The next substantive implementation stage should introduce traits rather than one monolithic evaluator.

```rust
pub trait SemanticExtractor {
    fn extract(
        &self,
        subject: &RdeSubject,
        context: &RdeContextBundle,
    ) -> Result<SemanticExtraction, RdeError>;
}

pub trait DeltaMAnalyzer {
    fn analyze(
        &self,
        source: &SemanticExtraction,
        target: &SemanticExtraction,
    ) -> Result<DeltaMReport, RdeError>;
}

pub trait RdeClassifier {
    fn classify(&self, report: &DeltaMReport) -> Result<RdeEvaluation, RdeError>;
}

pub trait EvidenceBinder {
    fn bind_evidence(&self, evaluation: &mut RdeEvaluation) -> Result<(), RdeError>;
}
```

This keeps the core extensible:

- rule-based extractors can be implemented first;
- model-backed extractors can be added later;
- human-curated extractors can be supported;
- validators remain independent of model choices.

## 7. Implementation phases

### Phase A — Current scaffold

Delivered:

- `RdeSubject`
- `RdeContext`
- `RdeEvaluator`
- `ConservativeRdeEvaluator`
- `RdeEvaluation`
- SLS-4-compatible output validation
- SLS-5 traceability documentation

### Phase B — Context bundle and semantic elements

Next recommended work.

Add:

- `RdeContextBundle`
- `SemanticElementKind`
- `SemanticElement`
- `SemanticExtraction`
- initial rule-based or manual extractor scaffold

Candidate issue:

```text
impl: add RdeContextBundle and SemanticElement extraction scaffold
```

### Phase C — ΔM report model

Add:

- `DeltaMRelationKind`
- `DeltaMRelation`
- `DeltaMReport`
- simple matcher for exact/near-exact preservation and obvious loss

Candidate issue:

```text
impl: add DeltaM report model and conservative analyzer
```

### Phase D — RDE classifier pipeline

Add:

- `RdeClassifier`
- classifier from `DeltaMReport` to `RdeEvaluation`
- category-specific tests
- evidence propagation

Candidate issue:

```text
impl: add RDE classifier from DeltaM report
```

### Phase E — Evidence and traceability expansion

Add:

- `EvidenceRef`
- source span support
- spec section references
- prior RDE references
- audit correlation expansion

Candidate issue:

```text
impl: add evidence references and traceability enrichment
```

### Phase F — Model-assisted evaluator adapters

Add optional adapters outside the core minimum.

Requirements:

- no model dependency in default core;
- schema validation after model output;
- evidence-bound output;
- uncertainty notes;
- meta-RDE review hooks.

Candidate issue:

```text
impl: define optional model-assisted RDE adapter interface
```

### Phase G — Meta-RDE quality review

Add checks for RDE output quality.

Candidate issue:

```text
impl: add meta-RDE quality checks for evaluator output
```

## 8. Non-goals

The roadmap does not require:

- a specific model provider;
- a specific prompt format;
- a claim that current scaffold is semantically complete;
- automatic approval/rejection;
- a universal safety filter;
- product UI behavior;
- full persistence design.

## 9. RDE drift risks to avoid

| Risk | Why it matters | Mitigation |
| --- | --- | --- |
| Treating RDE as a linter | Reduces meaning audit to syntax or style checks | Preserve semantic element and ΔM layers. |
| Treating an LLM as RDE | Confuses model output with institutional review structure | Keep model assistance behind traits and validators. |
| Treating category labels as evidence | Produces plausible but ungrounded review | Require evidence refs for mature implementations. |
| Treating RDE as approval | Violates SLS-5 human authority boundary | Keep approval outside RDE output. |
| Treating Git diff as semantic diff | Misses responsibility, scope, and unresolvedness | Build semantic extraction and ΔM report layers. |
| Treating memory as authority | Confuses stored observation with decision | Store refs and observations, not final decisions. |

## 10. Immediate next step

The recommended next implementation issue is:

```text
impl: add RdeContextBundle and SemanticElement extraction scaffold
```

This should not make `ConservativeRdeEvaluator` smarter directly. Instead, it should insert the missing semantic extraction structure before classification. Without that layer, RDE risks remaining a category-labeled comment generator.
