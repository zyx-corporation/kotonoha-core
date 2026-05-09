# Persistence direction

This repository implements **Kotonoha / SLS** behaviour. **Deployment persistence** (lineage storage, RDE/interchange records, audit correlation) is targeted at **PostgreSQL** as the **single primary database** for production-aligned deployments.

## Implications for contributors

- Prefer **PostgreSQL** when adding storage adapters, migrations, or examples intended for shared environments.
- **SQLite** (or in-memory stores) remains acceptable for **tests and local developer workflows**, not as a substitute for the agreed production shape when validating integration assumptions.
- **Normative interchange formats** remain defined in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec); this file does **not** add schema requirements to the public specification.

## Relationship to `spec-traceability.md`

See [`spec-traceability.md`](spec-traceability.md) for mapping Rust modules to specification sections. Persistence adapters will be listed there when introduced.

---

## Changelog

| Date | Change |
| --- | --- |
| 2026-05-10 | Record PostgreSQL-as-primary decision (project governance; mirrors internal decision doc). |
