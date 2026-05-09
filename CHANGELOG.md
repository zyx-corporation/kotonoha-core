# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
