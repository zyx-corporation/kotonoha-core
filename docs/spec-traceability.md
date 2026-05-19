# Specification traceability (Phase 2)

This document maps **`kotonoha-core`** Rust modules to [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) normative sections. Update it when adding behaviour.

| `kotonoha-spec` section | Rust module / symbol |
| --- | --- |
| [`SLS-1 Introduction`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/introduction.md) | [`TARGET_SPEC_BUNDLE`](../src/lib.rs) — must align with interchange `spec_version` / `spec_bundle`. |
| [`SLS-3 Semantic lineage model`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/semantic-lineage-model.md) | [`src/lineage.rs`](../src/lineage.rs) — `LineageUnit` (serde **`deny_unknown_fields`**: only `id`, `prior_unit_id` in interchange JSON). |
| [`SLS-4 RDE review output`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-review-output.md) | [`src/rde.rs`](../src/rde.rs) — `validate_json`. |
| [`SLS-5 RDE implementation specification`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-implementation-specification.md) | [`src/rde_impl.rs`](../src/rde_impl.rs) — `RdeSubject`, `RdeContext`, `RdeEvaluator`, `ConservativeRdeEvaluator`, `RdeEvaluation`, `RdeTraceability`. |
| [`SLS-6 Representation of loss`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/representation-of-loss.md) | Enforced indirectly via RDE `lost` category validation in `rde.rs`; conservative implementation scaffold can emit `RdeCategory::Lost` in `rde_impl.rs`. |
| [`SLS-7 Relationship to audit trails`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/audit-trail-relationship.md) | [`src/rde_impl.rs`](../src/rde_impl.rs) — `RdeTraceability.audit_correlation_id`; [`src/store/postgres.rs`](../src/store/postgres.rs) under optional `postgres` feature. |
| [`SLS-8 Versioning`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/versioning.md) | README / changelog / this traceability file should update when section identifiers or obligations move. |
| *(non-normative envelope)* | [`src/interchange.rs`](../src/interchange.rs) — `validate_interchange_json`; [`InterchangeDocument`](../src/interchange.rs) uses serde **`deny_unknown_fields`** at the **top level** (`format`, `spec_bundle`, `lineage_unit`, `rde_document` only). Nested **`lineage_unit`** objects use the same rule via [`LineageUnit`](../src/lineage.rs) (`id`, `prior_unit_id` only). |

## RDE implementation scaffold (SLS-5)

`src/rde_impl.rs` is an implementation scaffold, not a claim of complete semantic understanding. It provides typed boundaries corresponding to SLS-5:

| SLS-5 section | Core type / function | Notes |
| --- | --- | --- |
| `SLS-5.3.1` RDE evaluator | `RdeEvaluator`, `ConservativeRdeEvaluator` | Minimal deterministic evaluator for tests and scaffolding. |
| `SLS-5.3.2` Subject adapter | `RdeSubject` | Holds `subject_ref`, optional source/changed text, source references. |
| `SLS-5.3.3` Context provider | `RdeContext` | Holds prior lineage, prior RDE output, audit correlation, human context refs. |
| `SLS-5.3.4` Output validator | `RdeEvaluation::validate` | Reuses existing `crate::rde::validate_json`. |
| `SLS-5.4` Minimum pipeline | `ConservativeRdeEvaluator::evaluate` | Performs subject validation, context use, observation classification, output validation support. |
| `SLS-5.7` Memory interaction | `RdeTraceability` fields | Stores IDs/refs only; does not authorize decisions. |
| `SLS-5.8` Audit correlation | `audit_correlation_id` | Correlation only, not approval. |
| `SLS-5.9` Human authority boundary | Module docs and `NextUpdatePolicy` observation | Output requires human confirmation before approval/publication. |
| `SLS-5.10` Policy/safety boundary | Module docs | RDE output is not enforcement/refusal/access control. |

## Phase 2 minimal scope (this repository)

Development expectations for **`cargo test`** layering vs CLI acceptance demos: **[`docs/unit_testing_guidelines.md`](unit_testing_guidelines.md)**.

The following are **in scope for Phase 2 “minimal implementation”** here, anchored on public sources:
 **`kotonoha-docs`** ([tutorials index](https://github.com/zyx-corporation/kotonoha-docs/blob/main/en/tutorials/README.md); see also [Phase 2 CLI acceptance demo](https://github.com/zyx-corporation/kotonoha-docs/blob/main/en/acceptance/phase2_cli_acceptance_demo.md)) and **`kotonoha-cli`** CI / [`scripts/phase2_acceptance_demo.sh`](https://github.com/zyx-corporation/kotonoha-cli/blob/main/scripts/phase2_acceptance_demo.sh):

| Delivered in Phase 2 | Notes |
| --- | --- |
| JSON validation for **RDE review output** aligned with `SLS-4` | [`src/rde.rs`](../src/rde.rs) |
| Minimal **`LineageUnit`** aligned with `SLS-3` | [`src/lineage.rs`](../src/lineage.rs) |
| **RDE implementation scaffold** aligned with `SLS-5` | [`src/rde_impl.rs`](../src/rde_impl.rs) |
| **Loss** obligations surfaced via RDE **`lost`** category checks (see `SLS-6`) | Indirect enforcement in `rde.rs`; scaffold emission in `rde_impl.rs` |
| **`kotonoha.interchange.v1`** envelope (non-normative in spec) for pipelines | [`src/interchange.rs`](../src/interchange.rs) |
| Optional **PostgreSQL** persistence (`postgres` feature): migrations, `interchange_documents`, derived rows | [`src/store/postgres.rs`](../src/store/postgres.rs), [`migrations/`](../migrations/) |

CLI surface contracts: [`kotonoha-cli` `cli-definition.md`](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md).

### PostgreSQL mapping (Phase 2 detail)

| Concern | Rust module |
| --- | --- |
| `SLS-7` abstract audit correlation | [`src/store/postgres.rs`](../src/store/postgres.rs) — optional feature **`postgres`**: pool, migrations, append-only `audit_events`, validated `rde_documents` / `lineage_units` inserts |
| *(non-normative envelope persistence)* | Same — **`insert_interchange_document_json`** → **`interchange_documents`** plus optional **`lineage_units`** / **`rde_documents`** in one transaction ([`interchange.rs`](../src/interchange.rs) validation rules) |

DDL sketches: [`migrations/`](../migrations/) — baseline [`20260510000000_v0_init.sql`](../migrations/20260510000000_v0_init.sql), interchange [`20260510120000_v0_interchange_documents.sql`](../migrations/20260510120000_v0_interchange_documents.sql), M1 draft [`20260520000000_m1_semantic_lineage.sql`](../migrations/20260520000000_m1_semantic_lineage.sql) ([`postgresql-schema-m1.md`](postgresql-schema-m1.md)); direction: [`persistence.md`](persistence.md).

### PostgreSQL mapping (M1 draft — informative)

| Concept (M0 provisional) | Table | Rust API |
| --- | --- | --- |
| Document Object | `document_objects` | *Deferred — [#22](https://github.com/zyx-corporation/kotonoha-core/issues/22)* |
| MeaningState | `meaning_states` | *Deferred — #22* |
| MeaningDelta (ΔM) | `meaning_deltas` | *Deferred — #22* |
| RDEAssessment | `rde_assessments` | *Deferred — #22* |
| ReviewDecision | `review_decisions` | *Deferred — #22* |
| AgentRun (minimal) | `agent_runs` | *Deferred — #22* |

## Phase 3 bridge (CLI ingest — informative)

The official **`kotonoha` CLI** (≥ **0.2.0**) exposes **`interchange ingest`** for a **`kotonoha.console_event.v0`** wrapper that dispatches to the same **`rde`** / **`interchange`** validation (and optional Postgres insert) as this crate’s public APIs. **Not normative in `kotonoha-spec`.** **W-2 outline** payload rules: [`20` §3](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/20_phase3_core_console_contract_outline_draft.md). **Parity / gap tables:** [`core-console-contract-gap-phase3-draft.md`](core-console-contract-gap-phase3-draft.md). Public CLI contract: [`kotonoha-cli` `cli-definition.md` §4.1](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md).

## Deferred (Phase 3 and later)

Work **not** required to declare Phase 2 minimal implementation complete:

- **Richer lineage graph** types and queries beyond the minimal `LineageUnit` struct (multi-hop graphs, richer identities, etc.).
- **Product-scale** audit pipelines, authorization, and operational policy beyond the DDL sketches—tracked against Phase 3+ gates in the phase plan.
- **Persistence evolution** (new tables, replication, retention) beyond the Phase 2 DDL sketches—coordinate with `kotonoha-spec` and [`persistence.md`](persistence.md) before expanding normative claims.
- **Model-backed semantic evaluation** or prompt profiles beyond the conservative scaffold in `rde_impl.rs`.

**Informative:** draft **core ⇄ console** contract expectations versus current modules — [`core-console-contract-gap-phase3-draft.md`](core-console-contract-gap-phase3-draft.md) (no links to private planning repositories in this mirror).
