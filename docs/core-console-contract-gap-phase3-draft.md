# Core ⇄ console contract — Phase 3 gap memo (informative)

Informative memo comparing **draft** expectations for a future core ⇄ console / UI boundary (maintained in organization internal planning) to **`kotonoha-core`** as it exists today. No normative requirement here — code is the arbiter unless behaviour is promoted through **`kotonoha-spec`** in the usual public escalation path.

## Event names (outline §2) vs current **`kotonoha-core`**

| Outline event (draft) | Current correlate | Gap (2026-05-11) |
| --- | --- | --- |
| **`document.intent.updated`** | None as a typed ingest | No dedicated persistence or Rust API surface for intent edits from a UI channel. |
| **`rde.review.requested`** | [`src/rde.rs`](../src/rde.rs) `validate_json` (invoked via library / tooling patterns) | No **queued** «review job» ingestion distinct from calling validation directly from CLI-style flows. |
| **`interchange.ingest.submitted`** | [`src/interchange.rs`](../src/interchange.rs) `validate_*` (+ optional Postgres `insert_interchange_document_json` when feature `postgres` is enabled) | Parity exists for envelope validation/storage; UI-specific submission metadata (submission idempotency callbacks, UX-facing status stream) remains undefined. |
| **`lineage.probe.requested`** | [`src/lineage.rs`](../src/lineage.rs) structs / validation helpers | Console-oriented read probes (authz, paging, selective fields) beyond Phase 2 minimal structs are unspecified. |

## Error meaning classes (outline §4) vs code

Validators return structured errors internally; **`cli-definition`** user-facing wording is authoritative for CLI exits. Explicit **enumeration** aligned with **`validation.shape`**, **`validation.semantic`**, **`escalation.spec_gap`** (outline §4) for a future core ↔ console API boundary is **not yet implemented**.

## Maintenance

When behaviour changes, refresh this memo in the **same branch** as `docs/spec-traceability.md`, or substitute with a tighter link-only row there.

---

## Changelog

| Date | Summary |
| --- | --- |
| 2026-05-10 | Initial memo; OSS mirror omits hyperlinks to private planning repositories (**#37**-style). |
