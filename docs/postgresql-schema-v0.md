# PostgreSQL schema notes (v0)

Experimental DDL lives in [`migrations/20260510000000_v0_init.sql`](../migrations/20260510000000_v0_init.sql). **Not normative** in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec).

## Tables

### `lineage_units`

| Column | Notes |
| --- | --- |
| `id` | Primary key; corresponds to [`LineageUnit.id`](https://github.com/zyx-corporation/kotonoha-core/blob/main/src/lineage.rs) / semantic-lineage-model. |
| `prior_unit_id` | Optional FK to predecessor unit (`ON DELETE RESTRICT`). |
| `created_at` | Insert timestamp. |

### `rde_documents`

Stores full JSON documents compatible with [`rde-review-output.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-review-output.md) (wrapper included in `payload`).

| Column | Notes |
| --- | --- |
| `subject_ref` | Denormalized for indexing; must match `rde_review_output.subject_ref` inside `payload`. |
| `spec_version` | Phase 1 constraint to `0.1`. |
| `payload` | JSONB; Gin index for containment queries. |

### `audit_events`

Append-only style table for correlation with RDE / lineage ([`audit-trail-relationship.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/audit-trail-relationship.md)).

| Column | Notes |
| --- | --- |
| `correlation_ref` | Typically aligns with `subject_ref` or deployment-defined IDs. |

### `interchange_documents`

Stores one JSON document per row: the full core **interchange envelope** (`format` = `kotonoha.interchange.v1`). Introduced in [`migrations/20260510120000_v0_interchange_documents.sql`](../migrations/20260510120000_v0_interchange_documents.sql). Not normative in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec); validated in Rust via [`interchange::validate_interchange_json`](https://github.com/zyx-corporation/kotonoha-core/blob/main/src/interchange.rs) before insert.

| Column | Notes |
| --- | --- |
| `payload` | JSONB; entire envelope (may embed `lineage_unit` and/or `rde_document`). Gin index for containment queries. |

Denormalized copies of lineage / RDE may still be stored separately in `lineage_units` / `rde_documents` when deployments want direct querying; this table preserves the **exact interchange artifact** as exchanged.

## Future increments

- Materialized views or triggers linking `interchange_documents.payload` to `lineage_units` / `rde_documents`.
- Multi-tenant partition keys.
- Immutability triggers on `audit_events`.
