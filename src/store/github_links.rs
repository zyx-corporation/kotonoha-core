//! M4 GitHub correlation persistence ([`PgStore`] extensions).
//!
//! Issue: <https://github.com/zyx-corporation/kotonoha-core/issues/32>

use sqlx::Row;
use uuid::Uuid;

use super::postgres::{PgStore, StoreError};

/// `github_repository_links` row.
#[derive(Debug, Clone)]
pub struct GithubRepositoryLinkRow {
    pub id: Uuid,
    pub owner: String,
    pub repo: String,
    pub project_id: Option<Uuid>,
    pub default_branch: Option<String>,
}

/// Input to register or resolve a GitHub repository binding.
#[derive(Debug, Clone)]
pub struct GithubRepoRef {
    pub owner: String,
    pub repo: String,
    pub project_id: Option<Uuid>,
    pub default_branch: Option<String>,
}

/// `github_issue_links` row.
#[derive(Debug, Clone)]
pub struct GithubIssueLinkRow {
    pub id: Uuid,
    pub repository_link_id: Uuid,
    pub meaning_delta_id: Uuid,
    pub issue_number: i32,
    pub issue_url: Option<String>,
}

/// `github_pull_request_links` row.
#[derive(Debug, Clone)]
pub struct GithubPullRequestLinkRow {
    pub id: Uuid,
    pub repository_link_id: Uuid,
    pub meaning_delta_id: Option<Uuid>,
    pub rde_assessment_id: Option<Uuid>,
    pub pr_number: i32,
    pub pr_url: Option<String>,
    pub head_sha: Option<String>,
}

/// `github_review_comment_refs` row.
#[derive(Debug, Clone)]
pub struct GithubReviewCommentRefRow {
    pub id: Uuid,
    pub repository_link_id: Uuid,
    pub review_decision_id: Uuid,
    pub github_comment_id: i64,
    pub comment_url: Option<String>,
}

impl PgStore {
    /// Whether M4 `github_repository_links` table exists.
    pub async fn m4_schema_present(&self) -> Result<bool, StoreError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = 'github_repository_links'
            )",
        )
        .fetch_one(self.pool())
        .await?;
        Ok(exists)
    }

    /// Inserts or returns existing repository link for `owner`/`repo`.
    pub async fn upsert_github_repository(
        &self,
        repo: &GithubRepoRef,
    ) -> Result<GithubRepositoryLinkRow, StoreError> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO github_repository_links (owner, repo, project_id, default_branch)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (owner, repo) DO UPDATE SET
                 project_id = COALESCE(EXCLUDED.project_id, github_repository_links.project_id),
                 default_branch = COALESCE(EXCLUDED.default_branch, github_repository_links.default_branch)
               RETURNING id"#,
        )
        .bind(&repo.owner)
        .bind(&repo.repo)
        .bind(repo.project_id)
        .bind(&repo.default_branch)
        .fetch_one(self.pool())
        .await?;
        self.get_github_repository_link(id)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }

    /// Fetches a repository link by id.
    pub async fn get_github_repository_link(
        &self,
        id: Uuid,
    ) -> Result<Option<GithubRepositoryLinkRow>, StoreError> {
        let row = sqlx::query(
            "SELECT id, owner, repo, project_id, default_branch FROM github_repository_links WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(github_repository_row_from_pg))
    }

    /// Links a meaning delta to a GitHub Issue number.
    pub async fn link_meaning_delta_to_github_issue(
        &self,
        repository_link_id: Uuid,
        meaning_delta_id: Uuid,
        issue_number: i32,
        issue_url: Option<&str>,
    ) -> Result<GithubIssueLinkRow, StoreError> {
        if issue_number <= 0 {
            return Err(StoreError::RdeValidation(
                "issue_number must be positive".into(),
            ));
        }
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO github_issue_links (
                repository_link_id, meaning_delta_id, issue_number, issue_url
            ) VALUES ($1, $2, $3, $4)
            ON CONFLICT (repository_link_id, meaning_delta_id, issue_number) DO UPDATE SET
                issue_url = COALESCE(EXCLUDED.issue_url, github_issue_links.issue_url)
            RETURNING id"#,
        )
        .bind(repository_link_id)
        .bind(meaning_delta_id)
        .bind(issue_number)
        .bind(issue_url)
        .fetch_one(self.pool())
        .await?;
        self.get_github_issue_link(id)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }

    pub async fn get_github_issue_link(
        &self,
        id: Uuid,
    ) -> Result<Option<GithubIssueLinkRow>, StoreError> {
        let row = sqlx::query(
            r#"SELECT id, repository_link_id, meaning_delta_id, issue_number, issue_url
               FROM github_issue_links WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(github_issue_link_row_from_pg))
    }

    /// Lists issue links for a meaning delta (newest first).
    pub async fn list_github_issue_links_for_meaning_delta(
        &self,
        meaning_delta_id: Uuid,
    ) -> Result<Vec<GithubIssueLinkRow>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT id, repository_link_id, meaning_delta_id, issue_number, issue_url
               FROM github_issue_links
               WHERE meaning_delta_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(meaning_delta_id)
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(github_issue_link_row_from_pg)
            .collect())
    }

    /// Links a meaning delta to a pull request.
    pub async fn link_meaning_delta_to_github_pr(
        &self,
        repository_link_id: Uuid,
        meaning_delta_id: Uuid,
        pr_number: i32,
        pr_url: Option<&str>,
        head_sha: Option<&str>,
    ) -> Result<GithubPullRequestLinkRow, StoreError> {
        if pr_number <= 0 {
            return Err(StoreError::RdeValidation(
                "pr_number must be positive".into(),
            ));
        }
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO github_pull_request_links (
                repository_link_id, meaning_delta_id, pr_number, pr_url, head_sha
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (repository_link_id, pr_number, meaning_delta_id)
                WHERE meaning_delta_id IS NOT NULL
            DO UPDATE SET
                pr_url = COALESCE(EXCLUDED.pr_url, github_pull_request_links.pr_url),
                head_sha = COALESCE(EXCLUDED.head_sha, github_pull_request_links.head_sha)
            RETURNING id"#,
        )
        .bind(repository_link_id)
        .bind(meaning_delta_id)
        .bind(pr_number)
        .bind(pr_url)
        .bind(head_sha)
        .fetch_one(self.pool())
        .await?;
        self.get_github_pull_request_link(id)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }

    /// Links an RDE assessment to a pull request (no meaning delta on row).
    pub async fn link_rde_assessment_to_github_pr(
        &self,
        repository_link_id: Uuid,
        rde_assessment_id: Uuid,
        pr_number: i32,
        pr_url: Option<&str>,
        head_sha: Option<&str>,
    ) -> Result<GithubPullRequestLinkRow, StoreError> {
        if pr_number <= 0 {
            return Err(StoreError::RdeValidation(
                "pr_number must be positive".into(),
            ));
        }
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO github_pull_request_links (
                repository_link_id, rde_assessment_id, pr_number, pr_url, head_sha
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (repository_link_id, pr_number, rde_assessment_id)
                WHERE rde_assessment_id IS NOT NULL
            DO UPDATE SET
                pr_url = COALESCE(EXCLUDED.pr_url, github_pull_request_links.pr_url),
                head_sha = COALESCE(EXCLUDED.head_sha, github_pull_request_links.head_sha)
            RETURNING id"#,
        )
        .bind(repository_link_id)
        .bind(rde_assessment_id)
        .bind(pr_number)
        .bind(pr_url)
        .bind(head_sha)
        .fetch_one(self.pool())
        .await?;
        self.get_github_pull_request_link(id)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }

    pub async fn get_github_pull_request_link(
        &self,
        id: Uuid,
    ) -> Result<Option<GithubPullRequestLinkRow>, StoreError> {
        let row = sqlx::query(
            r#"SELECT id, repository_link_id, meaning_delta_id, rde_assessment_id,
                      pr_number, pr_url, head_sha
               FROM github_pull_request_links WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(github_pr_link_row_from_pg))
    }

    /// Lists PR links for a repository and PR number.
    pub async fn list_github_pr_links(
        &self,
        repository_link_id: Uuid,
        pr_number: i32,
    ) -> Result<Vec<GithubPullRequestLinkRow>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT id, repository_link_id, meaning_delta_id, rde_assessment_id,
                      pr_number, pr_url, head_sha
               FROM github_pull_request_links
               WHERE repository_link_id = $1 AND pr_number = $2
               ORDER BY created_at DESC"#,
        )
        .bind(repository_link_id)
        .bind(pr_number)
        .fetch_all(self.pool())
        .await?;
        Ok(rows.into_iter().map(github_pr_link_row_from_pg).collect())
    }

    /// Lists meaning deltas correlated to a PR via `head_sha` = `git_commit` or direct PR link.
    pub async fn list_meaning_deltas_for_github_pr(
        &self,
        repository_link_id: Uuid,
        pr_number: i32,
        head_sha: Option<&str>,
    ) -> Result<Vec<super::postgres::MeaningDeltaRow>, StoreError> {
        use super::postgres::meaning_delta_row_from_pg;

        let rows = sqlx::query(
            r#"SELECT DISTINCT md.id, md.git_commit, md.file_path, md.line_range_start,
                      md.line_range_end, md.diff_ref, md.observation, md.source_context
               FROM meaning_deltas md
               LEFT JOIN github_pull_request_links prl
                 ON prl.meaning_delta_id = md.id
                 AND prl.repository_link_id = $1
                 AND prl.pr_number = $2
               WHERE prl.id IS NOT NULL
                  OR ($3::text IS NOT NULL AND md.git_commit = $3)
               ORDER BY md.git_commit DESC"#,
        )
        .bind(repository_link_id)
        .bind(pr_number)
        .bind(head_sha)
        .fetch_all(self.pool())
        .await?;
        Ok(rows.into_iter().map(meaning_delta_row_from_pg).collect())
    }

    /// Records a GitHub review comment id after a ReviewDecision is posted.
    pub async fn link_review_decision_to_github_comment(
        &self,
        repository_link_id: Uuid,
        review_decision_id: Uuid,
        github_comment_id: i64,
        comment_url: Option<&str>,
    ) -> Result<GithubReviewCommentRefRow, StoreError> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO github_review_comment_refs (
                repository_link_id, review_decision_id, github_comment_id, comment_url
            ) VALUES ($1, $2, $3, $4)
            ON CONFLICT (repository_link_id, review_decision_id, github_comment_id) DO UPDATE SET
                comment_url = COALESCE(EXCLUDED.comment_url, github_review_comment_refs.comment_url)
            RETURNING id"#,
        )
        .bind(repository_link_id)
        .bind(review_decision_id)
        .bind(github_comment_id)
        .bind(comment_url)
        .fetch_one(self.pool())
        .await?;
        self.get_github_review_comment_ref(id)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }

    pub async fn get_github_review_comment_ref(
        &self,
        id: Uuid,
    ) -> Result<Option<GithubReviewCommentRefRow>, StoreError> {
        let row = sqlx::query(
            r#"SELECT id, repository_link_id, review_decision_id, github_comment_id, comment_url
               FROM github_review_comment_refs WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(github_review_comment_ref_row_from_pg))
    }
}

fn github_repository_row_from_pg(row: sqlx::postgres::PgRow) -> GithubRepositoryLinkRow {
    GithubRepositoryLinkRow {
        id: row.get("id"),
        owner: row.get("owner"),
        repo: row.get("repo"),
        project_id: row.get("project_id"),
        default_branch: row.get("default_branch"),
    }
}

fn github_issue_link_row_from_pg(row: sqlx::postgres::PgRow) -> GithubIssueLinkRow {
    GithubIssueLinkRow {
        id: row.get("id"),
        repository_link_id: row.get("repository_link_id"),
        meaning_delta_id: row.get("meaning_delta_id"),
        issue_number: row.get("issue_number"),
        issue_url: row.get("issue_url"),
    }
}

fn github_pr_link_row_from_pg(row: sqlx::postgres::PgRow) -> GithubPullRequestLinkRow {
    GithubPullRequestLinkRow {
        id: row.get("id"),
        repository_link_id: row.get("repository_link_id"),
        meaning_delta_id: row.get("meaning_delta_id"),
        rde_assessment_id: row.get("rde_assessment_id"),
        pr_number: row.get("pr_number"),
        pr_url: row.get("pr_url"),
        head_sha: row.get("head_sha"),
    }
}

fn github_review_comment_ref_row_from_pg(row: sqlx::postgres::PgRow) -> GithubReviewCommentRefRow {
    GithubReviewCommentRefRow {
        id: row.get("id"),
        repository_link_id: row.get("repository_link_id"),
        review_decision_id: row.get("review_decision_id"),
        github_comment_id: row.get("github_comment_id"),
        comment_url: row.get("comment_url"),
    }
}

#[cfg(all(test, feature = "postgres"))]
mod github_links_integration_tests {
    use super::*;
    use crate::semantic_lineage::{GitAnchor, MeaningDeltaInput, ReviewDecisionKind};
    use crate::store::postgres::PgStore;
    use serde_json::json;

    fn database_url_for_integration_test() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            panic!("postgres integration tests need DATABASE_URL");
        })
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL"]
    async fn m4_github_links_roundtrip() {
        let store = PgStore::connect(&database_url_for_integration_test())
            .await
            .expect("connect");
        store.migrate().await.expect("migrate");
        assert!(store.m4_schema_present().await.expect("m4 check"));

        let repo = store
            .upsert_github_repository(&GithubRepoRef {
                owner: "zyx-corporation".into(),
                repo: format!("m4-itest-{}", Uuid::new_v4()),
                project_id: None,
                default_branch: Some("main".into()),
            })
            .await
            .expect("upsert repo");

        let delta_id = store
            .create_meaning_delta(&MeaningDeltaInput {
                document_object_id: None,
                prior_meaning_state_id: None,
                new_meaning_state_id: None,
                agent_run_id: None,
                git_anchor: GitAnchor {
                    git_commit: "abc123def456".into(),
                    file_path: "docs/example.md".into(),
                    line_range_start: Some(1),
                    line_range_end: Some(10),
                    diff_ref: None,
                },
                observation: json!({}),
                source_context: json!({}),
                project_id: None,
                acting_principal_id: None,
            })
            .await
            .expect("create delta");

        let issue = store
            .link_meaning_delta_to_github_issue(
                repo.id,
                delta_id,
                42,
                Some("https://github.com/example/issues/42"),
            )
            .await
            .expect("link issue");
        assert_eq!(issue.issue_number, 42);

        let pr = store
            .link_meaning_delta_to_github_pr(
                repo.id,
                delta_id,
                7,
                Some("https://github.com/example/pull/7"),
                Some("abc123def456"),
            )
            .await
            .expect("link pr");
        assert_eq!(pr.pr_number, 7);

        let listed = store
            .list_meaning_deltas_for_github_pr(repo.id, 7, Some("abc123def456"))
            .await
            .expect("list deltas for pr");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, delta_id);

        let review_id = store
            .record_review_decision(&crate::semantic_lineage::RecordReviewDecisionInput {
                meaning_delta_id: delta_id,
                rde_assessment_id: None,
                decision: ReviewDecisionKind::Approve,
                decided_by: "reviewer@m4-itest".into(),
                rationale: json!({}),
                principal_id: None,
            })
            .await
            .expect("review");

        let comment = store
            .link_review_decision_to_github_comment(
                repo.id,
                review_id,
                999_001,
                Some("https://github.com/example/pull/7#discussion_r999001"),
            )
            .await
            .expect("comment ref");
        assert_eq!(comment.github_comment_id, 999_001);
    }
}
