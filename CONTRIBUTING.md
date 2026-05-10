# Contributing to kotonoha-core

Follow [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) for normative behaviour. Informative governance for this crate in the ecosystem: [`docs/repository-governance.md`](docs/repository-governance.md).

Update [`docs/spec-traceability.md`](docs/spec-traceability.md) when you add or change public API surface tied to the specification.

## Build

```bash
cargo test
cargo fmt --all --check
cargo check --features postgres
```

### PostgreSQL feature

Integration tests that hit a real database live behind `#[ignore]` and only run when you pass `--include-ignored` and set `DATABASE_URL`.

Example after starting PostgreSQL (see [`docker-compose.yml`](docker-compose.yml); database name there is `kotonoha_dev`):

```bash
export DATABASE_URL=postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev
cargo test --features postgres -- --include-ignored
```

Without `DATABASE_URL`, ignored tests are skipped so `cargo test --features postgres` still completes without a running database.

## Workflow

Organization **Git/Issue/branch/PR** rules (**no direct edits to `main`**): **[`docs/git_operation_rules.md`](docs/git_operation_rules.md)** ([canonical in **`kotonoha-management`**](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/04_git_operation_rules.md); update canon first).

1. Open an **Issue** for design questions that might affect the public specification (resolve in `kotonoha-spec` first when normative).
2. **Pull requests** should include tests and traceability updates.
3. Behaviour that expands **representation of lost elements** beyond the current RDE `lost`-category pathway should cite **[issue #3](https://github.com/zyx-corporation/kotonoha-spec/issues/3)** in the PR body when opening follow-up discussions (successor PRs inherit the linkage).

## License

[Apache License 2.0](LICENSE).
