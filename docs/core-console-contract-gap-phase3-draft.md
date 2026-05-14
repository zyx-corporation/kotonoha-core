# Core ⇄ console contract — Phase 3 gap memo (informative)

Informative memo comparing **draft** expectations for a future core ⇄ console / UI boundary (maintained in organization internal planning) to **`kotonoha-core`** as it exists today. No normative requirement here — code is the arbiter unless behaviour is promoted through **`kotonoha-spec`** in the usual public escalation path.

## Event names (outline §2) vs current **`kotonoha-core`**

| Outline event (draft) | Current correlate | Gap (2026-05-11) |
| --- | --- | --- |
| **`document.intent.updated`** | None as a typed ingest | No dedicated persistence or Rust API surface for intent edits from a UI channel. |
| **`rde.review.requested`** | [`src/rde.rs`](../src/rde.rs) `validate_json`; **CLI:** **`kotonoha interchange ingest`** with **`kind`** **`rde.review.requested`** (≥ **0.2.0**, [`cli-definition` §4.1](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md)) | No **queued** «review job» model; validation-only path matches direct **`rde validate`** on **`body`**. |
| **`interchange.ingest.submitted`** | [`src/interchange.rs`](../src/interchange.rs) `validate_*` (+ optional Postgres `insert_interchange_document_json` when feature `postgres` is enabled); **CLI:** [`kotonoha interchange ingest`](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md) **`--persist`** (≥ **0.2.0**) wraps envelope in **`kotonoha.console_event.v0`** | Parity exists for envelope validation/storage; UI-specific submission metadata (submission idempotency callbacks, UX-facing status stream) remains undefined. |
| **`lineage.probe.requested`** | [`src/lineage.rs`](../src/lineage.rs) structs / validation helpers | Console-oriented read probes (authz, paging, selective fields) beyond Phase 2 minimal structs are unspecified. |

## Payload ([`20` §3](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/20_phase3_core_console_contract_outline_draft.md) — ペイロードと interchange) vs [`src/interchange.rs`](../src/interchange.rs)

| Outline §3 expectation | Current core behaviour | Gap (2026-05-14) |
| --- | --- | --- |
| **`kotonoha.interchange.v1`** envelope without contradicting schema / serde | `InterchangeDocument` with top-level serde **`deny_unknown_fields`** (`format`, `spec_bundle`, `lineage_unit`, `rde_document` only); nested `lineage_unit` uses `id` / `prior_unit_id` only | **Aligned** with strict JSON rules surfaced in CLI (`kotonoha_core` ≥ **0.1.6** per `cli-definition`). |
| At least one of `lineage_unit` / `rde_document` | `validate_interchange_json` enforces presence of ≥1 | **Aligned**. |
| Schema evolution coordinated with [`17`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/17_spec_escalation_workflow.md) + CLI | Code changes still require manual spec / CLI / traceability updates | **Process gap only** (no additional runtime guard here). |

## Error meaning classes (outline §4) vs code

Validators return structured errors internally; **`cli-definition`** user-facing wording is authoritative for CLI exits. Explicit **enumeration** aligned with **`validation.shape`**, **`validation.semantic`**, **`escalation.spec_gap`** (outline §4) for a future core ↔ console API boundary is **not yet implemented** in typed form. **Informative mapping (v0 candidate):** [kotonoha-management#34 comment (P3-2b)](https://github.com/zyx-corporation/kotonoha-management/issues/34#issuecomment-4449294038).

## M3.3 trace — `console_event` → CLI exit → outline §4 (informative)

**P3-3d** spine for [`22` M3.3 acceptance](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/22_phase3_implementation_plan_draft.md): one developer path uses **`kotonoha interchange ingest`** on **`kotonoha.console_event.v0`** JSON (see [`cli-definition` §4.1](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md)). **Wrapper malformed** (missing keys, wrong `version`, unknown `kind`) → CLI exit **`1`**. **`body`** fails `kotonoha_core` validation (shape / semantic / RDE rules) → exit **`2`** — operationally read as **`validation.shape`** / **`validation.semantic`** in [`20` §4](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/20_phase3_core_console_contract_outline_draft.md) until a typed API exists. **`--persist`** with DB failures → exit **`3`** (**`persistence.*`**). **`escalation.spec_gap`** remains a **process** label (no dedicated exit code today).

## Maintenance

When behaviour changes, refresh this memo in the **same branch** as `docs/spec-traceability.md`, or substitute with a tighter link-only row there.

---

## Changelog

| Date | Summary |
| --- | --- |
| 2026-05-10 | Initial memo; OSS mirror omits hyperlinks to private planning repositories (**#37**-style). |
| 2026-05-12 | Event rows: **`kotonoha` CLI ≥ 0.2.0** `interchange ingest` + **`kotonoha.console_event.v0`**. |
| 2026-05-14 | **P3-2c:** [`20` §3](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/20_phase3_core_console_contract_outline_draft.md) vs `interchange.rs` table; P3-2b exit-code mapping link ([#46](https://github.com/zyx-corporation/kotonoha-management/issues/46)). |
| 2026-05-14 | **P3-3d:** M3.3 trace subsection (`console_event` → CLI exit → `20` §4 labels). |
