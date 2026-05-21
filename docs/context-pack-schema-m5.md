# M5 Context Pack schema (kotonoha-core)

**Normative product spec:** [kotonoha-management `31_m5_agent_run_integration_spec_draft.md` §5](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/31_m5_agent_run_integration_spec_draft.md)

**Sample JSON:** [`examples/context_pack_sample.v0.1.json`](examples/context_pack_sample.v0.1.json)

## Format identifier

| Field | Value |
| --- | --- |
| `format` | `kotonoha.context_pack.v0.1` |

## Top-level fields

| Field | Required | Description |
| --- | --- | --- |
| `format` | yes | Constant above |
| `generated_at_unix` | yes | UTC seconds when pack was built |
| `git_anchor` | yes | Same shape as M1 [`GitAnchor`](../src/semantic_lineage.rs) |
| `meaning_delta_draft` | no | `observation` + `source_context` (not persisted until CLI/UI confirms) |
| `rde_hints` | yes | [`ObservationRdeHints`](../src/observation_rde.rs) from draft observation |
| `policy_ref` | no | URL or doc anchor for capability / human-responsibility policy |

## Rust API

| Symbol | Module |
| --- | --- |
| `CONTEXT_PACK_FORMAT` | `context_pack` |
| `build_context_pack` | `context_pack` |
| `validate_context_pack` | `context_pack` |
| `parse_context_pack_json` | `context_pack` |

## CLI (M5-c)

`kotonoha context export` — implemented in `kotonoha-cli` ([#23](https://github.com/zyx-corporation/kotonoha-cli/issues/23)).

## Issue

[kotonoha-core#34](https://github.com/zyx-corporation/kotonoha-core/issues/34) · parent [management#106](https://github.com/zyx-corporation/kotonoha-management/issues/106)
