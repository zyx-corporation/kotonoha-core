# Phase D — RDE classifier design gate

Status: **non-normative design guidance** — implementation not started.

This document establishes the design constraints for Phase D before any code is written. Phase D is the first stage where `DeltaMReport` relations are mapped into SLS-4 RDE categories. This is a higher-risk transition than Phase B or Phase C because it approaches normative RDE output.

## Purpose

Phase D introduces the `RdeClassifier` trait and a `ConservativeRdeClassifier` that maps `DeltaMReport` → `RdeEvaluation` (SLS-4 categories).

The classifier **must not** pretend to understand meaning. It maps structured ΔM relations into RDE categories while preserving human authority, evidence traceability, and uncertainty notes.

## Pipeline boundary

```text
Phase B:     RdeContextBundle → SemanticExtraction
Phase C:     SemanticExtraction × SemanticExtraction → DeltaMReport
Phase D:     DeltaMReport → RdeEvaluation / SLS-4 categories
Human review: approval / rejection / accountability decision
```

Phase C records relations. Phase D classifies them into RDE categories. Human approval, rejection, and accountability decisions are a separate layer after Phase D.

## Prohibited shortcuts

| Prohibited shortcut | Reason |
|---|---|
| Mechanical `Removed` → `lost` mapping | `Removed` is an observation. `lost` is a review judgment and requires human/contextual confirmation. |
| Mechanical `Transformed` → `deviation_risk` mapping | Change is not danger. Drift/risk requires evidence and context. |
| Treating `Complemented` as a good addition | Value judgment about additions is outside the ΔM relation layer. |
| Classifier returning `approval`/`rejection` | Violates the human accountability boundary (SLS-5.9). |
| Classifier returning a safety verdict | RDE is not a safety filter (SLS-5.10). |
| Producing judgment with empty `evidence_refs` and no `confidence_note` | Absence of evidence must result in confidence/uncertainty notes. |
| Presenting RDE output as final judgment | RDE output is review focus. Final judgment belongs to the human review layer. |

## Conservative mapping policy (draft)

This is a design draft, not a specification. Actual implementation may refine these mappings as evidence handling matures.

| ΔM relation | SLS-4 category | Notes |
|---|---|---|
| `Preserved` | `preserved` | Direct mapping when kind/text are identical. |
| `Transformed` | `transformed` | Map with uncertainty note when evidence is weak or missing. |
| `Complemented` | `complemented` | Map as structural addition; no value judgment. |
| `Removed` | *review focus* | **Not** automatically `lost`. Flag for human review. May be classified as `lost` only with explicit human confirmation. |
| `Unresolved` | `intentionally_unresolved` | Only when the relation is explicitly marked as intentional. Otherwise, treat as review focus. |
| `Contradicted` | `deviation_risk` *candidate* | Not automatic. Requires evidence. Conservative classifier may leave as review focus. |
| `Weakened` | `deviation_risk` *candidate* | Not automatic. Requires evidence and scope context. |
| `Split` / `Merged` | `transformed` or review focus | Depends on evidence of intent preservation across the split/merge. |

## Future implementation candidate

When implementation begins, the candidate types are:

- `RdeClassifier` trait
- `ConservativeRdeClassifier`
- `DeltaMReport` → `RdeEvaluation` mapping logic

The classifier must accept `DeltaMReport` and produce `RdeEvaluation` without:
- LLM calls
- fuzzy matching
- embedding similarity
- approval/rejection verdicts
- safety classifications

## Non-goals

- `RdeClassifier` implementation
- `ConservativeRdeClassifier` implementation
- SLS-4 category mapping code
- approval / rejection logic
- safety verdict logic
- policy enforcement
- LLM integration
- UI / CLI / MCP / Orchestrator changes
- DB schema changes

## RDE boundary note

This design gate is **non-normative**. The authoritative specification remains `kotonoha-spec` SLS-4 and SLS-5.

Phase D is the first layer that touches SLS-4 categories. The risk is that a classifier that appears "obvious" (e.g., `Removed` → `lost`) may prematurely convert structured observation into normative judgment. The design gate exists to prevent that drift.
