# Specification traceability (Phase 2)

This document maps **`kotonoha-core`** Rust modules to [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) normative sections. Update it when adding behaviour.

| `kotonoha-spec` document | Rust module / symbol |
| --- | --- |
| [`docs/rde-review-output.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-review-output.md) | [`src/rde.rs`](../src/rde.rs) — `validate_json` |
| [`docs/semantic-lineage-model.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/semantic-lineage-model.md) | [`src/lineage.rs`](../src/lineage.rs) — `LineageUnit` |
| [`docs/representation-of-loss.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/representation-of-loss.md) | Enforced indirectly via RDE `lost` category validation in `rde.rs`; **normative expansion** tracked publicly in [`kotonoha-spec#3`](https://github.com/zyx-corporation/kotonoha-spec/issues/3). |
| [`docs/introduction.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/introduction.md) | [`TARGET_SPEC_BUNDLE`](../src/lib.rs) — must align with interchange `spec_version` |
| *(non-normative envelope)* | [`src/interchange.rs`](../src/interchange.rs) — `validate_interchange_json`, bundles lineage +/or RDE for pipelines |

## Phase 2 minimal scope (this repository)

The following are **in scope for Phase 2 “minimal implementation”** here, anchored on public sources: **`kotonoha-docs`** ([Phase 2 CLI walkthrough](https://github.com/zyx-corporation/kotonoha-docs/blob/main/docs/tutorials/phase2_cli_walkthrough.md)) and **`kotonoha-cli`** CI / [`scripts/phase2_acceptance_demo.sh`](https://github.com/zyx-corporation/kotonoha-cli/blob/main/scripts/phase2_acceptance_demo.sh):

| Delivered in Phase 2 | Notes |
| --- | --- |
| JSON validation for **RDE review output** aligned with `docs/rde-review-output.md` | [`src/rde.rs`](../src/rde.rs) |
| Minimal **`LineageUnit`** aligned with `docs/semantic-lineage-model.md` | [`src/lineage.rs`](../src/lineage.rs) |
| **Loss** obligations surfaced via RDE **`lost`** category checks (see `docs/representation-of-loss.md`) | Indirect enforcement in `rde.rs` |
| **`kotonoha.interchange.v1`** envelope (non-normative in spec) for pipelines | [`src/interchange.rs`](../src/interchange.rs) |
| Optional **PostgreSQL** persistence (`postgres` feature): migrations, `interchange_documents`, derived rows | [`src/store/postgres.rs`](../src/store/postgres.rs), [`migrations/`](../migrations/) |

CLI surface contracts: [`kotonoha-cli` `cli-definition.md`](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md).

### PostgreSQL mapping (Phase 2 detail)

| Concern | Rust module |
| --- | --- |
| [`audit-trail-relationship.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/audit-trail-relationship.md) (abstract audit correlation) | [`src/store/postgres.rs`](../src/store/postgres.rs) — optional feature **`postgres`**: pool, migrations, append-only `audit_events`, validated `rde_documents` / `lineage_units` inserts |
| *(non-normative envelope persistence)* | Same — **`insert_interchange_document_json`** → **`interchange_documents`** plus optional **`lineage_units`** / **`rde_documents`** in one transaction ([`interchange.rs`](../src/interchange.rs) validation rules) |

DDL sketches: [`migrations/`](../migrations/) — baseline [`20260510000000_v0_init.sql`](../migrations/20260510000000_v0_init.sql), interchange [`20260510120000_v0_interchange_documents.sql`](../migrations/20260510120000_v0_interchange_documents.sql); direction: [`persistence.md`](persistence.md).

## Deferred (Phase 3 and later)

Work **not** required to declare Phase 2 minimal implementation complete:

- **Richer lineage graph** types and queries beyond the minimal `LineageUnit` struct (multi-hop graphs, richer identities, etc.).
- **Product-scale** audit pipelines, authorization, and operational policy beyond the DDL sketches—tracked against Phase 3+ gates in the phase plan.
- **Persistence evolution** (new tables, replication, retention) beyond the Phase 2 DDL sketches—coordinate with `kotonoha-spec` and [`persistence.md`](persistence.md) before expanding normative claims.

**Informative:** internal management outline **`docs/20` (core ⇄ console events)** versus current modules — [`core-console-contract-gap-phase3-draft.md`](core-console-contract-gap-phase3-draft.md).

