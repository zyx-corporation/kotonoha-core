# Unit, integration, and acceptance testing (`kotonoha-core`)

This repository distinguishes three layers:

| Layer | What it proves | Where it runs here |
| --- | --- | --- |
| **Unit / library** | validators and pure logic accept, reject, or warn as intended | **`#[cfg(test)] mod tests` inside `src/**/*.rs`** |
| **Integration (env-dependent)** | persistence round-trips via PostgreSQL migrations and queries | **`#[tokio::test]` with `feature = "postgres"` and `#[ignore]`** until `DATABASE_URL` is provided — see [**`CONTRIBUTING.md`**](../CONTRIBUTING.md) |
| **Acceptance (phase gates)** | end-to-end expectations for adopters (“Phase 2 minimum”) | **CLI tutorials and scripted demos** (`kotonoha-docs` Phase 2 walkthrough, **`kotonoha-cli` `phase2_acceptance_demo.sh`**), not duplicated in `cargo test` alone |

Behaviour relative to **`kotonoha-spec`** is tracked in [`spec-traceability.md`](spec-traceability.md). Do not widen normative meaning only through tests; resolve spec deltas in **`kotonoha-spec`** issues/PRs first when required.

This file states how **Positive** and **Negative** testing are used **in this repository** (aligned in spirit with organization-side RDE methodology drafts, without linking private material).

---

## Positive / Negative (semantic invariants)

**Positive** tests prove spec‑aligned validators and CLI-visible outcomes regress safely; **Negative** tests freeze **disallowed shapes** (bad `spec_version` / `spec_bundle`, empty envelopes, malformed RDE envelopes, etc.) so “flexible JSON” cannot silently become a norm expansion. Treat failing Negative suites as semantic drift risks, not only “bugs”.

**Do not widen normative meaning via tests alone:** follow **`spec` Issue / PR → `spec-traceability` + unit tests → `cli-definition` exit codes** when behaviour must change.

For changes that resemble **cross-repo interchange / catalogue / DDL** edits, use PR notes (ΔM / guards) as described here: prefer **Red → Green → Refactor**. When touching invariants **write or extend Negative assertions first**. If implementation landed ahead of tests, commit a failing repro first, then fix in a follow‑up commit. Keep unrelated ΔMs out of one PR where practical; state in the PR which drift each guard blocks.

## CI vs scripted acceptance (“E2E-like”)

**`cargo test` on CI** should keep agreed **reject / strict-regression paths** green without extra flags (`postgres`-gated **`#[ignore]`** jobs separately with `DATABASE_URL`). **Phase 2 gate‑level reassurance** stays with **[`phase2_acceptance_demo.sh`](https://github.com/zyx-corporation/kotonoha-cli/blob/main/scripts/phase2_acceptance_demo.sh)** and **`kotonoha-docs`** walkthrough (**do not** replace them with library tests alone). Browser/UI E2E is out of OSS gate scope today unless future milestones add harnesses.

Japanese guidance maintained in tandem with partner project drafts: **[`docs/unit_testing_guidelines_ja.md`](unit_testing_guidelines_ja.md)**.

---

## Current inventory (snapshot 2026-05-11)

- **thirty** synchronous library tests: **three** `lineage`, **twelve** `interchange`, **fifteen** `rde` (Negative-heavy envelope and RDE-shape guards per testing policy).
- **two** ignored async PostgreSQL smoke tests gated on **`DATABASE_URL`** and **`postgres`** feature (`src/store/postgres.rs`).
- **no** top-level **`tests/`** Rust integration binaries yet—optional Phase 3 harness per internal planning (**P3-3e** stresses keeping `cargo test` and Phase 2 acceptance scripts compatible).
