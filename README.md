# kotonoha-core

**Open-source core implementation of the Semantic Lineage System (SLS)** for the Kotonoha ecosystem. This repository hosts source code, developer-oriented documentation, and implementation guidance that align with the public specifications.

**Japanese:** [README_ja.md](README_ja.md)

## Scope

- Core libraries and runtime components for SLS (Phase 2 onward).
- Developer documentation, build instructions, and contribution notes.

Normative, review-facing specifications live in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). This repository implements those specifications; see **[spec traceability](docs/spec-traceability.md)** for section ↔ code mapping.

### Repository governance (informative)

[**`docs/repository-governance.md`**](docs/repository-governance.md) summarizes ecosystem roles (spec vs implementation), recommended change flow, and RDE scope — **not normative**. When in doubt, follow [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec).

### Phase 2 (current minimum)

Rust crate **`kotonoha_core`** (`Cargo.toml`):

- **`lineage::LineageUnit`** — minimal lineage unit (`docs/semantic-lineage-model.md`).
- **`rde::validate_json`** — Phase 1 RDE interchange validation (`docs/rde-review-output.md`).
- **`interchange::validate_interchange_json`** — optional envelope combining lineage and/or RDE JSON for tool pipelines (format `kotonoha.interchange.v1`; not part of normative `kotonoha-spec`).
- **`store::postgres`** (Cargo feature **`postgres`**) — SQLx connection pool, filesystem migrations, and validated inserts into `lineage_units`, `rde_documents`, and `audit_events`.
- Constant **`TARGET_SPEC_BUNDLE`** (`0.1`) — must match interchange `spec_version`.

### Build

Requires [Rust](https://www.rust-lang.org/tools/install) stable (MSRV in `Cargo.toml`).

```bash
cargo test
```

Test layering (unit vs database integration vs acceptance): [**`docs/unit_testing_guidelines.md`**](docs/unit_testing_guidelines.md) (**[`docs/unit_testing_guidelines_ja.md`](docs/unit_testing_guidelines_ja.md)**).

### Persistence (deployments)

Production-oriented storage targets **PostgreSQL** as the single primary database — see [`docs/persistence.md`](docs/persistence.md).

**Local PostgreSQL (development)**

```bash
docker compose up -d
export DATABASE_URL="postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev"
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f migrations/20260510000000_v0_init.sql
```

See [`migrations/README.md`](migrations/README.md).

## Related repositories

Public cross-references only.

| Repository | Role |
| --- | --- |
| [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) | Canonical public specifications |
| **kotonoha-core (this repository)** | OSS core implementation of SLS |
| [`kotonoha-cli`](https://github.com/zyx-corporation/kotonoha-cli) | Official `kotonoha` CLI ([definition](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md)) |
| [`kotonoha-docs`](https://github.com/zyx-corporation/kotonoha-docs) | Non-specification public docs (manuals, tutorials, guides) |

Whenever possible, changes here should track the specifications in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). Resolve open design questions through the public specification before locking behavior in code.

## Language policy

**By default, documents under `kotonoha-core` are written in English.** Provide Japanese alongside the English source when helpful. Keep English primary and use the `*_ja.md` suffix for Japanese files (for example, `README.md` / `README_ja.md`).

## License

Unless otherwise stated in a specific file, repository content is licensed under the [Apache License 2.0](LICENSE).

## Links

- Repository: https://github.com/zyx-corporation/kotonoha-core
- GitHub Projects (organization workflow): [`docs/github_projects_policy.md`](docs/github_projects_policy.md)
