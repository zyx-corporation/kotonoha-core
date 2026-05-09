# PostgreSQL migrations

SQL files are applied **manually or via your migration runner** (SQLx, Flyway, Liquibase, etc.). This repository ships **raw DDL** as a contract sketch — not yet wired to a Rust migration crate.

## Apply (example)

With [`docker-compose.yml`](../docker-compose.yml) running locally:

```bash
export DATABASE_URL="postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev"
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f migrations/001_init.sql
```

Use a dedicated database user with least privilege in real deployments.

## Files

| File | Purpose |
| --- | --- |
| `001_init.sql` | Initial tables: `lineage_units`, `rde_documents`, `audit_events`. |

See [`docs/postgresql-schema-v0.md`](../docs/postgresql-schema-v0.md) for column semantics and spec references.
