# PostgreSQL schema — M2 RDE metadata (informative)

**Parent:** [kotonoha-management#103](https://github.com/zyx-corporation/kotonoha-management/issues/103)  
**Migration:** [`migrations/20260522000000_m2_rde_meta.sql`](../migrations/20260522000000_m2_rde_meta.sql)  
**M1 baseline:** [`postgresql-schema-m1.md`](postgresql-schema-m1.md)

## `rde_assessments` additive columns

| Column | Type | Notes |
| --- | --- | --- |
| `payload_schema_version` | `TEXT` | Copy of `rde_review_output.spec_version` at attach time |
| `source_kind` | `TEXT` | `cli` \| `llm` \| `import` \| `replay` |
| `validation_report` | `JSONB` | Machine-readable summary (`strict`, `warnings`, …) |

Existing M1 rows remain valid (NULL columns). Apply with `kotonoha db migrate`.

## Rollback (manual)

```sql
ALTER TABLE rde_assessments
  DROP CONSTRAINT IF EXISTS rde_assessments_source_kind_check,
  DROP COLUMN IF EXISTS validation_report,
  DROP COLUMN IF EXISTS source_kind,
  DROP COLUMN IF EXISTS payload_schema_version;
```
