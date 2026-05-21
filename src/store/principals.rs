//! M6 principals / projects persistence ([`PgStore`] extensions).
//!
//! Issue: <https://github.com/zyx-corporation/kotonoha-core/issues/35>

use sqlx::Row;
use uuid::Uuid;

use crate::semantic_lineage::SemanticLineageError;

use super::postgres::{PgStore, StoreError};

/// Well-known IDs inserted by [`20260522120000_m6_principals_projects.sql`](../../migrations/20260522120000_m6_principals_projects.sql).
pub struct LegacyDefaults;

/// Resolved M6 actor + project scope for one store operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationContext {
    pub principal_id: Uuid,
    pub project_id: Uuid,
}

impl OperationContext {
    pub fn legacy_default() -> Self {
        Self {
            principal_id: LegacyDefaults::PRINCIPAL_ID,
            project_id: LegacyDefaults::PROJECT_ID,
        }
    }

    pub fn resolve(principal_id: Option<Uuid>, project_id: Option<Uuid>) -> Self {
        Self {
            principal_id: principal_id.unwrap_or(LegacyDefaults::PRINCIPAL_ID),
            project_id: project_id.unwrap_or(LegacyDefaults::PROJECT_ID),
        }
    }
}

impl LegacyDefaults {
    pub const PRINCIPAL_ID: Uuid = Uuid::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]);
    pub const PROJECT_ID: Uuid = Uuid::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02,
    ]);
    pub const PRINCIPAL_EXTERNAL_REF: &'static str = "kotonoha.m6.legacy-default";
    pub const PROJECT_SLUG: &'static str = "default";
}

/// `principals.kind` values (matches migration check).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalKind {
    Human,
    Service,
    AgentChannel,
}

impl PrincipalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Service => "service",
            Self::AgentChannel => "agent_channel",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "human" => Some(Self::Human),
            "service" => Some(Self::Service),
            "agent_channel" => Some(Self::AgentChannel),
            _ => None,
        }
    }
}

/// `project_members.role` values (matches migration check).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectMemberRole {
    Owner,
    Reviewer,
    Viewer,
    AgentRunner,
}

impl ProjectMemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Reviewer => "reviewer",
            Self::Viewer => "viewer",
            Self::AgentRunner => "agent_runner",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(Self::Owner),
            "reviewer" => Some(Self::Reviewer),
            "viewer" => Some(Self::Viewer),
            "agent_runner" => Some(Self::AgentRunner),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrincipalRow {
    pub id: Uuid,
    pub kind: String,
    pub display_name: String,
    pub external_ref: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
}

impl PgStore {
    /// Whether M6 `principals` table exists.
    pub async fn m6_schema_present(&self) -> Result<bool, StoreError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = 'principals'
            )",
        )
        .fetch_one(self.pool())
        .await?;
        Ok(exists)
    }

    pub async fn get_principal(&self, id: Uuid) -> Result<Option<PrincipalRow>, StoreError> {
        let row = sqlx::query(
            "SELECT id, kind, display_name, external_ref FROM principals WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(|r| PrincipalRow {
            id: r.get("id"),
            kind: r.get("kind"),
            display_name: r.get("display_name"),
            external_ref: r.get("external_ref"),
        }))
    }

    pub async fn get_project_by_slug(&self, slug: &str) -> Result<Option<ProjectRow>, StoreError> {
        let row = sqlx::query("SELECT id, slug, name FROM projects WHERE slug = $1")
            .bind(slug)
            .fetch_optional(self.pool())
            .await?;
        Ok(row.map(|r| ProjectRow {
            id: r.get("id"),
            slug: r.get("slug"),
            name: r.get("name"),
        }))
    }

    /// Returns true if `principal_id` has `role` on `project_id`.
    pub async fn principal_has_role(
        &self,
        project_id: Uuid,
        principal_id: Uuid,
        role: ProjectMemberRole,
    ) -> Result<bool, StoreError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM project_members
                WHERE project_id = $1 AND principal_id = $2 AND role = $3
            )",
        )
        .bind(project_id)
        .bind(principal_id)
        .bind(role.as_str())
        .fetch_one(self.pool())
        .await?;
        Ok(exists)
    }

    /// Project id for a meaning delta (M6).
    pub async fn meaning_delta_project_id(
        &self,
        meaning_delta_id: Uuid,
    ) -> Result<Uuid, StoreError> {
        let id: Option<Uuid> =
            sqlx::query_scalar("SELECT project_id FROM meaning_deltas WHERE id = $1")
                .bind(meaning_delta_id)
                .fetch_optional(self.pool())
                .await?
                .flatten();
        id.ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }

    /// Requires `principal_id` to hold `role` on `project_id` when M6 schema is present.
    pub async fn require_project_role(
        &self,
        ctx: &OperationContext,
        role: ProjectMemberRole,
    ) -> Result<(), StoreError> {
        if !self.m6_schema_present().await? {
            return Ok(());
        }
        if self
            .principal_has_role(ctx.project_id, ctx.principal_id, role)
            .await?
        {
            return Ok(());
        }
        Err(StoreError::SemanticLineage(
            SemanticLineageError::AccessDenied(format!(
                "principal {} lacks role '{}' on project {}",
                ctx.principal_id,
                role.as_str(),
                ctx.project_id
            )),
        ))
    }

    /// Requires any one of `roles` (M6 only).
    pub async fn require_project_role_any(
        &self,
        ctx: &OperationContext,
        roles: &[ProjectMemberRole],
    ) -> Result<(), StoreError> {
        if !self.m6_schema_present().await? {
            return Ok(());
        }
        for role in roles {
            if self
                .principal_has_role(ctx.project_id, ctx.principal_id, *role)
                .await?
            {
                return Ok(());
            }
        }
        let names: Vec<_> = roles.iter().map(|r| r.as_str()).collect();
        Err(StoreError::SemanticLineage(
            SemanticLineageError::AccessDenied(format!(
                "principal {} lacks any of [{}] on project {}",
                ctx.principal_id,
                names.join(", "),
                ctx.project_id
            )),
        ))
    }

    /// AgentRun must belong to the acting principal (M6).
    pub async fn require_agent_run_principal(
        &self,
        agent_run_id: Uuid,
        acting_principal_id: Uuid,
    ) -> Result<(), StoreError> {
        if !self.m6_schema_present().await? {
            return Ok(());
        }
        let run_principal: Option<Uuid> =
            sqlx::query_scalar("SELECT principal_id FROM agent_runs WHERE id = $1")
                .bind(agent_run_id)
                .fetch_optional(self.pool())
                .await?
                .flatten();
        let Some(run_principal) = run_principal else {
            return Err(StoreError::Sql(sqlx::Error::RowNotFound));
        };
        if run_principal == acting_principal_id {
            return Ok(());
        }
        Err(StoreError::SemanticLineage(SemanticLineageError::AccessDenied(
            format!(
                "agent_run {agent_run_id} belongs to principal {run_principal}, not {acting_principal_id}"
            ),
        )))
    }

    /// Loads legacy default principal and project (post-migration).
    pub async fn get_legacy_defaults(&self) -> Result<(PrincipalRow, ProjectRow), StoreError> {
        let principal = self
            .get_principal(LegacyDefaults::PRINCIPAL_ID)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))?;
        let project = self
            .get_project_by_slug(LegacyDefaults::PROJECT_SLUG)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))?;
        Ok((principal, project))
    }
}

#[cfg(test)]
mod legacy_defaults_tests {
    use super::LegacyDefaults;

    #[test]
    fn legacy_uuid_constants_match_migration() {
        assert_eq!(
            LegacyDefaults::PRINCIPAL_ID.to_string(),
            "00000000-0000-4000-8000-000000000001"
        );
        assert_eq!(
            LegacyDefaults::PROJECT_ID.to_string(),
            "00000000-0000-4000-8000-000000000002"
        );
    }
}

#[cfg(all(test, feature = "postgres"))]
mod principals_integration_tests {
    use super::*;
    use crate::store::postgres::PgStore;

    fn database_url_for_integration_test() -> String {
        std::env::var("DATABASE_URL").expect("postgres integration tests need DATABASE_URL")
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL"]
    async fn m6_migration_backfill_and_membership() {
        let store = PgStore::connect(&database_url_for_integration_test())
            .await
            .expect("connect");
        store.migrate().await.expect("migrate");
        assert!(store.m6_schema_present().await.expect("m6 check"));

        let (principal, project) = store.get_legacy_defaults().await.expect("defaults");
        assert_eq!(principal.id, LegacyDefaults::PRINCIPAL_ID);
        assert_eq!(project.id, LegacyDefaults::PROJECT_ID);
        assert!(store
            .principal_has_role(project.id, principal.id, ProjectMemberRole::Owner,)
            .await
            .expect("role"));

        let null_agent_runs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM agent_runs WHERE principal_id IS NULL",
        )
        .fetch_one(store.pool())
        .await
        .expect("count");
        assert_eq!(null_agent_runs, 0);

        let null_deltas: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM meaning_deltas WHERE project_id IS NULL",
        )
        .fetch_one(store.pool())
        .await
        .expect("count");
        assert_eq!(null_deltas, 0);
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL"]
    async fn m6_rbac_denies_review_without_reviewer_role() {
        use crate::semantic_lineage::{
            GitAnchor, MeaningDeltaInput, RecordReviewDecisionInput, ReviewDecisionKind,
        };
        use crate::store::postgres::StoreError;

        let store = PgStore::connect(&database_url_for_integration_test())
            .await
            .expect("connect");
        store.migrate().await.expect("migrate");

        let viewer_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO principals (kind, display_name, external_ref)
               VALUES ('human', 'Viewer Only', 'test.m6.viewer')
               RETURNING id"#,
        )
        .fetch_one(store.pool())
        .await
        .expect("insert principal");

        sqlx::query(
            r#"INSERT INTO project_members (project_id, principal_id, role)
               VALUES ($1, $2, 'viewer')"#,
        )
        .bind(LegacyDefaults::PROJECT_ID)
        .bind(viewer_id)
        .execute(store.pool())
        .await
        .expect("membership");

        let commit = uuid::Uuid::new_v4().to_string().replace('-', "");
        let delta_id = store
            .create_meaning_delta(&MeaningDeltaInput {
                document_object_id: None,
                prior_meaning_state_id: None,
                new_meaning_state_id: None,
                agent_run_id: None,
                git_anchor: GitAnchor {
                    git_commit: commit,
                    file_path: "docs/m6-rbac.md".into(),
                    line_range_start: Some(1),
                    line_range_end: Some(2),
                    diff_ref: None,
                },
                observation: serde_json::json!({}),
                source_context: serde_json::json!({}),
                project_id: None,
                acting_principal_id: None,
            })
            .await
            .expect("create delta");

        let denied = store
            .record_review_decision(&RecordReviewDecisionInput {
                meaning_delta_id: delta_id,
                rde_assessment_id: None,
                decision: ReviewDecisionKind::Approve,
                decided_by: "viewer@test".into(),
                rationale: serde_json::json!({}),
                principal_id: Some(viewer_id),
            })
            .await
            .expect_err("review should be denied");

        match denied {
            StoreError::SemanticLineage(
                crate::semantic_lineage::SemanticLineageError::AccessDenied(_),
            ) => {}
            other => panic!("expected AccessDenied, got {other:?}"),
        }

        sqlx::query(
            r#"INSERT INTO project_members (project_id, principal_id, role)
               VALUES ($1, $2, 'reviewer')
               ON CONFLICT (project_id, principal_id) DO UPDATE SET role = 'reviewer'"#,
        )
        .bind(LegacyDefaults::PROJECT_ID)
        .bind(viewer_id)
        .execute(store.pool())
        .await
        .expect("promote to reviewer");

        store
            .record_review_decision(&RecordReviewDecisionInput {
                meaning_delta_id: delta_id,
                rde_assessment_id: None,
                decision: ReviewDecisionKind::Approve,
                decided_by: "viewer@test".into(),
                rationale: serde_json::json!({}),
                principal_id: Some(viewer_id),
            })
            .await
            .expect("review after role grant");
    }
}
