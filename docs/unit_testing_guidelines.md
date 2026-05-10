# Unit, integration, and acceptance testing (`kotonoha-core`)

This repository distinguishes three layers:

| Layer | What it proves | Where it runs here |
| --- | --- | --- |
| **Unit / library** | validators and pure logic accept, reject, or warn as intended | **`#[cfg(test)] mod tests` inside `src/**/*.rs`** |
| **Integration (env-dependent)** | persistence round-trips via PostgreSQL migrations and queries | **`#[tokio::test]` with `feature = "postgres"` and `#[ignore]`** until `DATABASE_URL` is provided — see [**`CONTRIBUTING.md`**](../CONTRIBUTING.md) |
| **Acceptance (phase gates)** | end-to-end expectations for adopters (“Phase 2 minimum”) | **CLI tutorials and scripted demos** (`kotonoha-docs` Phase 2 walkthrough, **`kotonoha-cli` `phase2_acceptance_demo.sh`**), not duplicated in `cargo test` alone |

Behaviour relative to **`kotonoha-spec`** is tracked in [`spec-traceability.md`](spec-traceability.md). Do not widen normative meaning only through tests; resolve spec deltas in **`kotonoha-spec`** issues/PRs first when required.

Japanese guidance maintained in tandem with partner project drafts: **[`docs/unit_testing_guidelines_ja.md`](unit_testing_guidelines_ja.md)**.

---

## Current inventory (snapshot 2026-05-11)

- **nine** synchronous unit tests combined across **`lineage`**, **`rde`**, and **`interchange`** modules.
- **two** ignored async PostgreSQL smoke tests gated on **`DATABASE_URL`** and **`postgres`** feature (`src/store/postgres.rs`).
- **no** top-level **`tests/`** Rust integration binaries yet—optional Phase 3 harness per internal planning (**P3-3e** stresses keeping `cargo test` and Phase 2 acceptance scripts compatible).
