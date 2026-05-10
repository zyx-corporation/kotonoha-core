# Repository governance (informative)

**Status:** Informative — not a substitute for [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). If this document disagrees with published normative specifications, **specifications win**.

This file summarizes **how `kotonoha-core` fits** in the public Kotonoha / SLS ecosystem. It complements [CONTRIBUTING.md](../CONTRIBUTING.md) and [`docs/spec-traceability.md`](spec-traceability.md).

---

## 1. Source of truth

| Layer | Role |
| --- | --- |
| **[`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec)** | **Normative** definitions for semantics, interchange records, and conformance language (English-first `docs/`). |
| **`kotonoha-core` (this repository)** | **Implementation source of truth** for the OSS Rust crate **`kotonoha_core`**: behaviour developers ship, tests, migrations, and developer docs. Behaviour should **match** the specifications; documented gaps are tracked via issues/PRs. |
| **`kotonoha-cli`** | Thin CLI wrapping core validation and storage; authoritative CLI contract in **[`cli-definition.md`](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md)**. |
| **`kotonoha-docs`** | Non-normative manuals and tutorials — must cite `kotonoha-spec` for exact meanings. |

---

## 2. Change flow (recommended)

1. **Normative ambiguity or new public semantics** → open an **[Issue or PR in `kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec)** first when possible.
2. **Implementation PR** in this repository → link the spec issue/PR; update **[`docs/spec-traceability.md`](spec-traceability.md)** when touching behaviour mapped to specification sections.
3. **Purely internal refactors** (no observable semantics change) → spec update not required; keep tests covering public behaviour.

If implementation temporarily leads specifications (e.g. spike), **backport** the distilled design into `kotonoha-spec` **before treating the behaviour as stable for external callers**.

---

## 3. RDE alignment

Governance statements here are **about process**, not substitutes for **[RDE definitions](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-review-output.md)**. Do **not** narrow RDE to “what `validate_json` happens to enforce today”: validation helpers implement **published rules** that evolve through the specification.

---

## 4. Persistence and operations

Operational choices documented here (PostgreSQL migrations, DDL sketches) supplement normative clauses only where **`kotonoha-spec`** points at correlation or versioning — see **`docs/spec-traceability.md`** persistence tables and **`docs/persistence.md`**.

---

## Changelog (document level)

| Date | Change |
| --- | --- |
| 2026-05-10 | Initial English summary derived from internal repository-governance draft (project PR #6 / `kotonoha-management` `13_repository_governance_draft.md`). |
