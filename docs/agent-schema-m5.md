# M5 AgentRun schema (kotonoha-core)

**Normative product spec:** [kotonoha-management `31_m5_agent_run_integration_spec_draft.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/31_m5_agent_run_integration_spec_draft.md)

**Migration:** `migrations/20260521100000_m5_agent_runs_extend.sql`

## Tables

| Table | M5 change |
| --- | --- |
| `agent_runs` | Adds `capability_profile`, `parent_run_id`, `status`, `output_artifact_refs`, `denied_actions` |

M1 columns retained: `agent_kind`, `external_ref`, `payload`, `created_at`.

`meaning_deltas.agent_run_id` → `agent_runs.id` (M1 FK, unchanged).

## Status values

| `status` | Meaning |
| --- | --- |
| `started` | Run opened (default on insert) |
| `completed` | Normal completion |
| `failed` | Terminal failure |
| `denied` | Capability denied at least once (`denied_actions` non-empty) |

## PgStore API

See `src/store/agent_runs.rs`:

- `m5_schema_present`
- `start_agent_run`, `get_agent_run`
- `update_agent_run_status`
- `append_agent_run_denied_action`
- `set_agent_run_output_artifacts`

## Related

- Context pack: [`context-pack-schema-m5.md`](context-pack-schema-m5.md) ([#34](https://github.com/zyx-corporation/kotonoha-core/issues/34))

## Issue

[kotonoha-core#33](https://github.com/zyx-corporation/kotonoha-core/issues/33) · parent [management#106](https://github.com/zyx-corporation/kotonoha-management/issues/106)
