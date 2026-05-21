# M6 schema — principals / projects

**Status:** informative draft DDL. **Not normative** in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec).

**Migration:** [`migrations/20260522120000_m6_principals_projects.sql`](../migrations/20260522120000_m6_principals_projects.sql)

**Parent:** [kotonoha-management#138](https://github.com/zyx-corporation/kotonoha-management/issues/138)

## Tables

| Table | Purpose |
| --- | --- |
| `principals` | Human / service / agent_channel actors |
| `projects` | Workspace slug + display name |
| `project_members` | `(project_id, principal_id, role)` |

## Extended columns

| Table | Column | Notes |
| --- | --- | --- |
| `agent_runs` | `principal_id` NOT NULL | FK → `principals` |
| `meaning_deltas` | `project_id` NOT NULL | FK → `projects` |
| `review_decisions` | `principal_id` NOT NULL | FK → `principals` |

## Legacy backfill (fixed UUIDs)

| Resource | UUID | `external_ref` / `slug` |
| --- | --- | --- |
| Default principal | `00000000-0000-4000-8000-000000000001` | `kotonoha.m6.legacy-default` |
| Default project | `00000000-0000-4000-8000-000000000002` | `default` |

Existing rows are assigned these IDs on migration. New deployments should create explicit principals/projects in M6-b+.

## Rust API

[`src/store/principals.rs`](../src/store/principals.rs):

- `LegacyDefaults` — well-known UUID constants
- `PgStore::m6_schema_present()`
- `get_principal`, `get_project_by_slug`, `principal_has_role`

## Roles (`project_members.role`)

| Role | Intended use |
| --- | --- |
| `owner` | Project administration |
| `reviewer` | Human `review.*` |
| `viewer` | Read / export |
| `agent_runner` | AgentRun + delta + RDE attach |

## RBAC (M6-b · `store::principals`)

When M6 schema is present, [`PgStore`](https://github.com/zyx-corporation/kotonoha-core/blob/main/src/store/postgres.rs) enforces:

| Operation | Required role |
| --- | --- |
| `start_agent_run` | `agent_runner` |
| `create_meaning_delta` (with `agent_run_id`) | `agent_runner` + run principal match |
| `create_meaning_delta` (no agent run) | `owner` \| `reviewer` \| `agent_runner` |
| `attach_rde_assessment` | `agent_runner` on delta's project |
| `record_review_decision` | `reviewer` |

Unset `acting_principal_id` / `project_id` / `principal_id` on inputs default to **legacy** UUIDs. Deny → [`SemanticLineageError::AccessDenied`](../src/semantic_lineage.rs).
