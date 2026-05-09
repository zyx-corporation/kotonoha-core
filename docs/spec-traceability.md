# Specification traceability (Phase 2)

This document maps **`kotonoha-core`** Rust modules to [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) normative sections. Update it when adding behaviour.

| `kotonoha-spec` document | Rust module / symbol |
| --- | --- |
| [`docs/rde-review-output.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-review-output.md) | [`src/rde.rs`](../src/rde.rs) — `validate_json` |
| [`docs/semantic-lineage-model.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/semantic-lineage-model.md) | [`src/lineage.rs`](../src/lineage.rs) — `LineageUnit` |
| [`docs/representation-of-loss.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/representation-of-loss.md) | Enforced indirectly via RDE `lost` category validation in `rde.rs` |
| [`docs/introduction.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/introduction.md) | [`TARGET_SPEC_BUNDLE`](../src/lib.rs) — must align with interchange `spec_version` |
| *(non-normative envelope)* | [`src/interchange.rs`](../src/interchange.rs) — `validate_interchange_json`, bundles lineage +/or RDE for pipelines |

## Deferred (later Phase 2+ increments)

- Richer lineage graph types beyond minimal `LineageUnit`.

### Persistence (PostgreSQL)

| Concern | Rust module |
| --- | --- |
| [`audit-trail-relationship.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/audit-trail-relationship.md) (abstract audit correlation) | [`src/store/postgres.rs`](../src/store/postgres.rs) — optional feature **`postgres`**: pool, migrations, append-only `audit_events`, validated `rde_documents` / `lineage_units` inserts |
| *(non-normative envelope persistence)* | Same — **`insert_interchange_document_json`** → table **`interchange_documents`** ([`interchange.rs`](../src/interchange.rs) validation rules) |

DDL sketches: [`migrations/`](../migrations/) — baseline [`20260510000000_v0_init.sql`](../migrations/20260510000000_v0_init.sql), interchange [`20260510120000_v0_interchange_documents.sql`](../migrations/20260510120000_v0_interchange_documents.sql); direction: [`persistence.md`](persistence.md).
