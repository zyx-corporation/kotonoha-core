# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
