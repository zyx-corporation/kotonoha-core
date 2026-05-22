# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- **SLS-9 / Phase 2:** `source_context_status` closed-vocabulary validation in [`src/rde.rs`](src/rde.rs) ([#44](https://github.com/zyx-corporation/kotonoha-core/issues/44)); unit tests for accepted, unknown, and non-string values.

### Documentation

- [`docs/spec-traceability.md`](docs/spec-traceability.md): SLS-9 row and M4 GitHub correlation (non-normative) mapping ([#45](https://github.com/zyx-corporation/kotonoha-core/issues/45)).

## [0.1.16] — 2026-05-21

### Added

- **M6-f** ([#138](https://github.com/zyx-corporation/kotonoha-management/issues/138)): `list_meaning_deltas_by_git_commit` optional `project_id` filter; `list_meaning_deltas_by_project`; `list_meaning_deltas_for_audit_export` (viewer+ RBAC); `MeaningDeltaRow.project_id`.

## [0.1.15] — 2026-05-21

### Changed

- **`validate_and_attach_rde`:** optional `acting_principal_id` for M6 RBAC on attach ([#138](https://github.com/zyx-corporation/kotonoha-management/issues/138) M6-c).

## [0.1.14] — 2026-05-21

### Added

- **M6-b RBAC** ([#138](https://github.com/zyx-corporation/kotonoha-management/issues/138)): `OperationContext`, `require_project_role`, INSERT-time `principal_id` / `project_id`; review requires `reviewer` role.

## [0.1.13] — 2026-05-21

### Added

- **M6 Team Mode (schema)** ([#138](https://github.com/zyx-corporation/kotonoha-management/issues/138)): migration [`20260522120000_m6_principals_projects.sql`](migrations/20260522120000_m6_principals_projects.sql) — `principals`, `projects`, `project_members`; `principal_id` / `project_id` columns with legacy backfill; [`docs/agent-schema-m6.md`](docs/agent-schema-m6.md); `store::principals` (`LegacyDefaults`, `m6_schema_present`, `principal_has_role`).

## [0.1.12] — 2026-05-21

### Added

- **M5 context pack** ([#34](https://github.com/zyx-corporation/kotonoha-core/issues/34)): `context_pack` module — `kotonoha.context_pack.v0.1`, `build_context_pack`, sample [`docs/examples/context_pack_sample.v0.1.json`](docs/examples/context_pack_sample.v0.1.json).

## [0.1.11] — 2026-05-21

### Added

- **M5 AgentRun** ([#33](https://github.com/zyx-corporation/kotonoha-core/issues/33)): `agent_runs` extension migration + `store::agent_runs` PgStore APIs; [`docs/agent-schema-m5.md`](docs/agent-schema-m5.md).

## [0.1.10] — 2026-05-20

### Added

- **M4 GitHub links** ([#32](https://github.com/zyx-corporation/kotonoha-core/issues/32)): `github_*_links` tables + `store::github_links`.

## [0.1.9] — 2026-05-22

### Added (tests)

- M2 unit tests: `rde_attach` strict/warn, `observation_rde` mapping; postgres integration `migrate_applies_m2_*` (see [`docs/unit_testing_guidelines_ja.md`](docs/unit_testing_guidelines_ja.md)).

### Added

- **M2 RDE metadata** ([#29](https://github.com/zyx-corporation/kotonoha-core/issues/29)): migration [`20260522000000_m2_rde_meta.sql`](migrations/20260522000000_m2_rde_meta.sql) — `payload_schema_version`, `source_kind`, `validation_report` on `rde_assessments`.
- **M2 validate + attach** ([#30](https://github.com/zyx-corporation/kotonoha-core/issues/30)): `PgStore::validate_and_attach_rde`, `AttachRdeAssessmentMeta`, `build_validation_report`.
- **Observation ↔ RDE hints** ([#31](https://github.com/zyx-corporation/kotonoha-core/issues/31)): `observation_rde::map_observation_to_rde_hints`.
- [`docs/postgresql-schema-m2.md`](docs/postgresql-schema-m2.md).

### Changed

- `PgStore::attach_rde_assessment` accepts optional M2 metadata (extra parameter).

## [0.1.8] — 2026-05-20

### Added

- `PgStore::list_rde_assessments_for_meaning_delta`, `list_review_decisions_for_meaning_delta` — CLI M1 `export` ([#28](https://github.com/zyx-corporation/kotonoha-core/pull/28), [cli#15](https://github.com/zyx-corporation/kotonoha-cli/issues/15)).

## [0.1.7] — 2026-05-20

### Added

- [`docs/repository-governance.md`](docs/repository-governance.md) — informative ecosystem and change-flow summary for implementers.
- **M1 draft PostgreSQL schema** ([#21](https://github.com/zyx-corporation/kotonoha-core/issues/21)): migration [`20260520000000_m1_semantic_lineage.sql`](migrations/20260520000000_m1_semantic_lineage.sql) and [`docs/postgresql-schema-m1.md`](docs/postgresql-schema-m1.md).
- **M1 PgStore APIs** ([#22](https://github.com/zyx-corporation/kotonoha-core/issues/22)): `create_meaning_delta`, `attach_rde_assessment`, `record_review_decision`, `get_meaning_delta`, `list_meaning_deltas_by_git_commit`.
- [`git`](src/git.rs) module ([#23](https://github.com/zyx-corporation/kotonoha-core/issues/23)): `discover_repo`, `working_tree_status`, `diff_unstaged`, `path_relative_to_root`.

### Changed

- [`docs/spec-traceability.md`](docs/spec-traceability.md) — `representation-of-loss` mapping cites [`kotonoha-spec#3`](https://github.com/zyx-corporation/kotonoha-spec/issues/3).
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — PR guidance for loss-semantics follow-up.

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
