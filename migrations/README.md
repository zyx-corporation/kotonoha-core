# PostgreSQL migrations

SQL files are applied **manually**, via **`kotonoha_core::store::postgres::PgStore::migrate`** (Rust / SQLx `Migrator`), or via third-party runners (Flyway, Liquibase, etc.).

## Apply (example)

With [`docker-compose.yml`](../docker-compose.yml) running locally:

```bash
export DATABASE_URL="postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev"
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f migrations/20260510000000_v0_init.sql
```

From the **`kotonoha`** CLI (Git dependency on `kotonoha-core` with feature `postgres`):

```bash
export DATABASE_URL="postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev"
kotonoha db migrate
```

Use a dedicated database user with least privilege in real deployments.

## Files

| File | Purpose |
| --- | --- |
| `20260510000000_v0_init.sql` | Initial tables: `lineage_units`, `rde_documents`, `audit_events`. |
| `20260510120000_v0_interchange_documents.sql` | Table `interchange_documents` (`kotonoha.interchange.v1` envelope JSONB). |
| `20260520000000_m1_semantic_lineage.sql` | **M1 draft:** `document_objects`, `meaning_states`, `meaning_deltas`, `rde_assessments`, `review_decisions`, `agent_runs` ([#21](https://github.com/zyx-corporation/kotonoha-core/issues/21)). |
| `20260520100000_m4_github_links.sql` | **M4:** `github_*_links` tables ([#32](https://github.com/zyx-corporation/kotonoha-core/issues/32)). |
| `20260521100000_m5_agent_runs_extend.sql` | **M5:** `agent_runs` extension ([#33](https://github.com/zyx-corporation/kotonoha-core/issues/33)). |

See [`docs/postgresql-schema-v0.md`](../docs/postgresql-schema-v0.md), [`docs/postgresql-schema-m1.md`](../docs/postgresql-schema-m1.md), [`docs/github-schema-m4.md`](../docs/github-schema-m4.md), and [`docs/agent-schema-m5.md`](../docs/agent-schema-m5.md) for column semantics.
