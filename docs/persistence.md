# Persistence direction

This repository implements **Kotonoha / SLS** behaviour. **Deployment persistence** (lineage storage, RDE/interchange records, audit correlation) is targeted at **PostgreSQL** as the **single primary database** for production-aligned deployments.

## Implications for contributors

- Prefer **PostgreSQL** when adding storage adapters, migrations, or examples intended for shared environments.
- **SQLite** (or in-memory stores) remains acceptable for **tests and local developer workflows**, not as a substitute for the agreed production shape when validating integration assumptions.
- **Normative interchange formats** remain defined in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec); this file does **not** add schema requirements to the public specification.

## Relationship to `spec-traceability.md`

See [`spec-traceability.md`](spec-traceability.md) for mapping Rust modules to specification sections. Persistence adapters will be listed there when introduced.

## Schema sketch (v0)

Initial DDL and column notes:

- [`migrations/20260510000000_v0_init.sql`](../migrations/20260510000000_v0_init.sql)
- [`docs/postgresql-schema-v0.md`](postgresql-schema-v0.md)
- Local database: [`docker-compose.yml`](../docker-compose.yml) + [`migrations/README.md`](../migrations/README.md)

---

## Changelog

| Date | Change |
| --- | --- |
| 2026-05-10 | Record PostgreSQL-as-primary decision (project governance; mirrors internal decision doc). |
| 2026-05-10 | Add initial DDL (`20260510000000_v0_init.sql`), schema notes, `docker-compose.yml`. |
| 2026-05-10 | Optional Rust adapter: `kotonoha_core` feature **`postgres`** (`store::postgres`), migrations + validated inserts. |
| 2026-05-10 | Migration **`20260510120000_v0_interchange_documents.sql`** and **`insert_interchange_document_json`** for `kotonoha.interchange.v1` envelopes. |
