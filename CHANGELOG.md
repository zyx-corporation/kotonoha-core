# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed

- [`docs/spec-traceability.md`](docs/spec-traceability.md) — [`representation-of-loss.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/representation-of-loss.md) mapping now cites public tracking **[`kotonoha-spec#3`](https://github.com/zyx-corporation/kotonoha-spec/issues/3)** for optional normative follow-up beyond the RDE `lost` pathway.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — PR guidance for linking that issue when expanding loss semantics.

### Added

- [`docs/repository-governance.md`](docs/repository-governance.md) — informative ecosystem and change-flow summary for implementers.
- **M1 draft PostgreSQL schema** ([#21](https://github.com/zyx-corporation/kotonoha-core/issues/21)): migration [`20260520000000_m1_semantic_lineage.sql`](migrations/20260520000000_m1_semantic_lineage.sql) and [`docs/postgresql-schema-m1.md`](docs/postgresql-schema-m1.md) (`document_objects`, `meaning_states`, `meaning_deltas`, `rde_assessments`, `review_decisions`, `agent_runs`). Rust insert APIs deferred to [#22](https://github.com/zyx-corporation/kotonoha-core/issues/22).

## [0.1.6] — 2026-05-10

### Changed

- **`InterchangeDocument` deserialization**: serde **`#[serde(deny_unknown_fields)]`** — JSON envelopes with unknown **top‑level** keys are rejected (`validate_interchange_json` / `from_json_str`), reducing “flexible JSON” drift versus informal spec extension. **Migrating callers** that relied on tolerated extra envelope fields must strip or rename them before validation.
- **`LineageUnit` deserialization** (within interchange `lineage_unit`): **`#[serde(deny_unknown_fields)]`** — only **`id`** and **`prior_unit_id`** are accepted in JSON objects passed through the interchange parser.
- **CI**: job-level **`DATABASE_URL`** documents intent for Postgres integration tests (`--features postgres` **`-- --include-ignored`**); clarified step naming.

### Added

- **Thirty** `#[cfg(test)]` library cases covering interchange **unknown fields** (envelope + `lineage_unit`), RDE **category / `spec_version`** edges, nested RDE in interchange (**strict**), and related Positive paths.

## [0.1.5] — 2026-05-10

### Fixed

- Migration **`20260510120000_v0_interchange_documents.sql`**: GIN index on `payload` uses **`jsonb_path_ops`** (PostgreSQL JSONB operator class). The previous **`json_path_ops`** name was invalid and caused migration failure.

### Added

- PostgreSQL **integration tests** (marked `#[ignore]`; run with `DATABASE_URL` and `cargo test --features postgres -- --include-ignored`).
- **CI**: PostgreSQL 16 service, `DATABASE_URL`, and the postgres integration test command above.

### Changed

- **`uuid`** optional dependency: enable **`v4`** so integration tests can generate unique IDs.

## [0.1.4] — 2026-05-10

### Changed

- **`PgStore::insert_interchange_document_json`**: one PostgreSQL **transaction** — insert into `interchange_documents`, then materialize optional `lineage_unit` / `rde_document` into `lineage_units` / `rde_documents` (same `strict_rde` as interchange validation for nested RDE).
- **`PgStore::insert_rde_document_json`**: internal refactor (`insert_rde_document_value_ex`); behaviour unchanged (`strict` RDE validation remains **`true`**).

### Documentation

- [`docs/postgresql-schema-v0.md`](docs/postgresql-schema-v0.md): document transactional materialization.

## [0.1.3] — 2026-05-10

### Added

- Migration **`migrations/20260510120000_v0_interchange_documents.sql`**: table `interchange_documents` (full `kotonoha.interchange.v1` envelope JSONB).
- **`PgStore::insert_interchange_document_json`** — validates via [`interchange::validate_interchange_json`](src/interchange.rs), then inserts into `interchange_documents`.

### Documentation

- [`docs/postgresql-schema-v0.md`](docs/postgresql-schema-v0.md): `interchange_documents` column notes.

## [0.1.2] — 2026-05-10

### Added

- PostgreSQL DDL **`migrations/20260510000000_v0_init.sql`** (`lineage_units`, `rde_documents`, `audit_events`), [`docs/postgresql-schema-v0.md`](docs/postgresql-schema-v0.md), [`docker-compose.yml`](docker-compose.yml), [`migrations/README.md`](migrations/README.md).
- Optional Cargo feature **`postgres`**: [`store::postgres`](src/store/postgres.rs) (`PgStore`) — SQLx pool, `Migrator`-based migrations from `migrations/`, `insert_lineage_unit`, `insert_rde_document_json` (strict RDE validation before insert), `insert_audit_event`.

### Documentation

- [`docs/persistence.md`](docs/persistence.md): PostgreSQL as the primary deployment database (project decision).

## [0.1.1] — 2026-05-10

### Added

- **`interchange`** module: `InterchangeDocument`, `INTERCHANGE_FORMAT_V1`, `validate_interchange_json` — exchangeable JSON envelope bundling optional `lineage_unit` and/or `rde_document` (not normative in `kotonoha-spec`; deployment interchange helper).

## [0.1.0] — 2026-05-10

### Added

- Phase 2 **minimum core library**: `kotonoha_core` crate with `lineage::LineageUnit` and `rde::validate_json` aligned to [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) Phase 1 interchange.
- [`docs/spec-traceability.md`](docs/spec-traceability.md).
- CI (fmt, test).
