# PostgreSQL schema notes (M1 — draft)

**Status:** informative draft DDL only. **Not normative** in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec).

Tracked by: [kotonoha-core#21](https://github.com/zyx-corporation/kotonoha-core/issues/21), parent [kotonoha-management#97](https://github.com/zyx-corporation/kotonoha-management/issues/97).

Concept source: [`kotonoha-docs` `kotonoha_concept.md` §3.1](https://github.com/zyx-corporation/kotonoha-docs/blob/main/ja/paper/kotonoha_concept.md#31-コア型の暫定定義m0).

DDL: [`migrations/20260520000000_m1_semantic_lineage.sql`](../migrations/20260520000000_m1_semantic_lineage.sql).

Prior v0 tables: [`postgresql-schema-v0.md`](postgresql-schema-v0.md).

---

## Design decisions (v0.1-provisional)

| Topic | Choice in this draft |
| --- | --- |
| **SourceContext** | Embedded JSONB on `meaning_states` and `meaning_deltas` (`source_context` column). Separate table deferred. |
| **Git anchor** | `meaning_deltas.git_commit` + `file_path` are **NOT NULL**. Either (`line_range_start`, `line_range_end`) **or** `diff_ref` must be set (`CHECK`). |
| **Observation categories** | Stored in `meaning_deltas.observation` JSONB until M1-b fixes a Rust shape. |
| **RDEAssessment vs `rde_documents`** | **Both** may exist: `rde_assessments` is M1 semantic-lineage bound; `rde_documents` remains spec Phase-1 interchange storage. Optional FK `rde_assessments.rde_document_id` bridges a validated interchange row. |
| **`lineage_units` vs `meaning_states`** | **Coexist.** `lineage_units.id` is spec URI-shaped lineage; `meaning_states` is richer semantic snapshot. Future work may add `meaning_states.lineage_unit_id TEXT REFERENCES lineage_units(id)` — not in this migration. |

---

## Tables

### `document_objects`

| Column | Notes |
| --- | --- |
| `id` | UUID primary key. |
| `external_ref` | Optional stable URI/slug (unique when set). |
| `title` | Human label. |

### `meaning_states`

| Column | Notes |
| --- | --- |
| `document_object_id` | Optional FK. |
| `git_commit` | Optional anchor when snapshot ties to a commit. |
| `snapshot` | JSONB — claims, intent, constraints, unresolved (schema evolution tolerant). |
| `source_context` | JSONB — issue links, conversation refs, etc. |

### `meaning_deltas`

| Column | Notes |
| --- | --- |
| `prior_meaning_state_id` / `new_meaning_state_id` | Optional ends of ΔM. |
| `git_commit` | Full commit SHA (required). |
| `file_path` | Repo-relative path (required). |
| `line_range_start` / `line_range_end` | Inclusive line range when not using `diff_ref`. |
| `diff_ref` | Alternative anchor (e.g. staged diff id / hash). |
| `observation` | RDE-oriented observation payload (preserved, transformed, lost, …). |
| `agent_run_id` | Optional FK to `agent_runs`. |

### `rde_assessments`

| Column | Notes |
| --- | --- |
| `meaning_delta_id` | Required parent. |
| `payload` | JSONB evaluation body. |
| `audit_correlation_id` | Aligns with `audit_events.correlation_ref` / SLS-7 practice. |
| `rde_document_id` | Optional link to existing `rde_documents` row. |

### `review_decisions`

| Column | Notes |
| --- | --- |
| `decision` | `approve` \| `hold` \| `reject` \| `needs_revision`. |
| `decided_by` | Actor id (deployment-defined; not auth). |
| `rationale` | JSONB notes. |

### `agent_runs`

| Column | Notes |
| --- | --- |
| `agent_kind` | e.g. `chatgpt_app`, `claude_code`. |
| `payload` | Run metadata / prompts summary (non-normative). |

---

## Relationship to v0 tables

```text
document_objects
  → meaning_states → meaning_deltas → rde_assessments → review_decisions
                                        ↘ (optional) rde_documents
                                        ↘ audit_events (via audit_correlation_id)

lineage_units / interchange_documents  (Phase 2 v0 — unchanged)
```

---

## Apply

```bash
export DATABASE_URL="postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev"
kotonoha db migrate
```

Or `PgStore::migrate()` from Rust (`postgres` feature).

---

## Next

- **M1-b ([#22](https://github.com/zyx-corporation/kotonoha-core/issues/22)):** `PgStore::create_meaning_delta`, `attach_rde_assessment`, `record_review_decision`, `get_meaning_delta`, `list_meaning_deltas_by_git_commit` — see [`semantic_lineage.rs`](../src/semantic_lineage.rs).
- **M1-c ([#23](https://github.com/zyx-corporation/kotonoha-core/issues/23)):** Git adapter populates [`GitAnchor`](../src/semantic_lineage.rs) for CLI.
