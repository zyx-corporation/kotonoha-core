# kotonoha-core Responsibility Boundary

## Status

**Informative — implementation mirror.** The canonical boundary document lives in **`kotonoha-spec`**:

→ **[`docs/core-responsibility-boundary.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/core-responsibility-boundary.md)**

If this summary disagrees with that document or with normative `kotonoha-spec` text, **spec wins**.

## Role in one sentence

`kotonoha-core` is the **shared Rust implementation layer** between normative specifications and runtimes (CLI, orchestrator services). It implements published rules; it does not define semantic authority.

## Quick reference

| Owns | Does not own |
| --- | --- |
| Validation helpers (`rde`, `interchange`, `lineage`) | Normative schema meaning → `kotonoha-spec` |
| RDE scaffold types (`rde_impl`, `observation_rde`) | CLI commands / exit codes → `kotonoha-cli` |
| Context pack / handoff **helpers** | Obsidian UX policy → `obsidian-kotonoha-console` |
| Optional Postgres store (`store`, migrations) | VSCode extension UX → `kotonoha-vscode` |
| Common library error types | Orchestrator HTTP API semantics → `kotonoha-orchestrator` |

**Schema helpers:** allowed. **Schema semantics as source of truth:** forbidden.

## Allowed modules (current)

See canonical doc for the full allow-list. Current public modules in [`src/lib.rs`](../src/lib.rs):

`context_pack`, `git`, `interchange`, `lineage`, `observation_rde`, `rde`, `rde_attach`, `rde_impl`, `semantic_lineage`, and optional `store` (`postgres` feature).

## Before you open a core PR

1. Link a `kotonoha-spec` issue/PR if observable semantics change.
2. Confirm the change is not CLI-only glue or UI policy.
3. Update [`spec-traceability.md`](spec-traceability.md) when behaviour maps to spec sections.
4. Run through the **Decision checklist** in the canonical boundary doc.

## Related

| Document | Role |
| --- | --- |
| [core-responsibility-boundary.md (spec)](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/core-responsibility-boundary.md) | Canonical boundary |
| [spec-traceability.md](spec-traceability.md) | Spec section ↔ Rust module |
| [repository-governance.md](repository-governance.md) | Change flow |
| [current-official-architecture.md (spec)](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/current-official-architecture.md) | Ecosystem roles |

Governance: [kotonoha-management #163](https://github.com/zyx-corporation/kotonoha-management/issues/163)
