# M4 GitHub link schema (kotonoha-core)

**Normative product spec:** [kotonoha-management `30_m4_github_integration_spec_draft.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/30_m4_github_integration_spec_draft.md)

**Migration:** `migrations/20260520100000_m4_github_links.sql`

## Tables

| Table | FK / purpose |
| --- | --- |
| `github_repository_links` | `owner` + `repo` (unique); optional `project_id` (M6) |
| `github_issue_links` | `repository_link_id` → `meaning_delta_id` + `issue_number` |
| `github_pull_request_links` | `meaning_delta_id` and/or `rde_assessment_id` + `pr_number`; optional `head_sha` |
| `github_review_comment_refs` | `review_decision_id` + `github_comment_id` |

GitHub holds canonical issue/PR/comment text; Kotonoha DB stores **IDs, numbers, URLs, and correlation** only.

## PgStore API

See `src/store/github_links.rs`:

- `upsert_github_repository`, `link_meaning_delta_to_github_issue`, `link_meaning_delta_to_github_pr`
- `link_rde_assessment_to_github_pr`, `list_meaning_deltas_for_github_pr`
- `link_review_decision_to_github_comment`
- `m4_schema_present`

## Issue

[kotonoha-core#32](https://github.com/zyx-corporation/kotonoha-core/issues/32) · parent [management#105](https://github.com/zyx-corporation/kotonoha-management/issues/105)
