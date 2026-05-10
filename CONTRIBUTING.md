# Contributing to kotonoha-core

Follow [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) for normative behaviour. Informative governance for this crate in the ecosystem: [`docs/repository-governance.md`](docs/repository-governance.md).

Sister-project **engineering RDE norms** adapted for organizational Japanese drafts **§§24–25**: **[Awai `rde_development_guidelines`](https://github.com/zyx-corporation/awai-commons/blob/main/docs/engineering/rde_development_guidelines.md)** (PR **ΔM** / triggers); **[Awai `rde_testing_policy`](https://github.com/zyx-corporation/awai-commons/blob/main/docs/engineering/rde_testing_policy.md)** (Positive / Negative layers, sequencing). Behavioural mapping for this codebase: **[`docs/unit_testing_guidelines.md`](docs/unit_testing_guidelines.md)** — keep OSS PR descriptions free of links to non-public trackers when possible.

Update [`docs/spec-traceability.md`](docs/spec-traceability.md) when you add or change public API surface tied to the specification.

## Design hygiene

Use object-oriented **design patterns**—separation of concerns, explicit extension points, sane dependency direction—when they clearly improve **maintainability** or **efficiency**, not as naming ceremony. Prefer Rust-native structure (**traits**, **enums**, **composition**, small `mod` boundaries) over deep inheritance-style hierarchies. Avoid speculative layering that obscures interchange/RDE/traceability paths or fights the **`cargo test`** invariants documented in **`docs/unit_testing_guidelines.md`**; explain trade-offs under PR **ΔM** (see **[Awai `rde_development_guidelines`](https://github.com/zyx-corporation/awai-commons/blob/main/docs/engineering/rde_development_guidelines.md) §5**).

## Build & tests

```bash
cargo test
cargo fmt --all --check
cargo check --features postgres
```

Layering (**unit vs integration vs acceptance**): **[`docs/unit_testing_guidelines.md`](docs/unit_testing_guidelines.md)** (Japanese sibling: **[`docs/unit_testing_guidelines_ja.md`](docs/unit_testing_guidelines_ja.md)**).

### PostgreSQL feature

Integration tests that hit a real database live behind `#[ignore]` and only run when you pass `--include-ignored` and set `DATABASE_URL`.

Example after starting PostgreSQL (see [`docker-compose.yml`](docker-compose.yml); database name there is `kotonoha_dev`):

```bash
export DATABASE_URL=postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev
cargo test --features postgres -- --include-ignored
```

Without `DATABASE_URL`, ignored tests are skipped so `cargo test --features postgres` still completes without a running database.

**CI:** [.github/workflows/ci.yml](.github/workflows/ci.yml) starts PostgreSQL 16 (`kotonoha_test`), sets **`DATABASE_URL`**, and runs `cargo test --features postgres -- --include-ignored` after the default **`cargo test`**, so **`#[ignore]`** integration tests (`src/store/postgres.rs`) run on **`main`** and pull requests automatically.
## Workflow

Organization **Git/Issue/branch/PR** rules (**no direct edits to `main`**): **[`docs/git_operation_rules.md`](docs/git_operation_rules.md)** (Japanese; self-contained in this repo).

1. Open an **Issue** for design questions that might affect the public specification (resolve in `kotonoha-spec` first when normative).
2. **Pull requests** should include tests and traceability updates.
3. For substantive **interchange**, **RDE catalogue**, **Postgres DDL**, or **spec-bundle** churn, summarise **ΔM and guards** in the PR body (see [**Awai `rde_development_guidelines`](https://github.com/zyx-corporation/awai-commons/blob/main/docs/engineering/rde_development_guidelines.md) §5 and **[`docs/unit_testing_guidelines.md`](docs/unit_testing_guidelines.md)**).
4. Behaviour that expands **representation of lost elements** beyond the current RDE `lost`-category pathway should cite **[issue #3](https://github.com/zyx-corporation/kotonoha-spec/issues/3)** in the PR body when opening follow-up discussions (successor PRs inherit the linkage).

## License

[Apache License 2.0](LICENSE).
