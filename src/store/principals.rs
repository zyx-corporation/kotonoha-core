//! M6 principals / projects persistence ([`PgStore`] extensions).
//!
//! Issue: <https://github.com/zyx-corporation/kotonoha-core/issues/35>

use sqlx::Row;
use uuid::Uuid;

use super::postgres::{PgStore, StoreError};

/// Well-known IDs inserted by [`20260522120000_m6_principals_projects.sql`](../../migrations/20260522120000_m6_principals_projects.sql).
pub struct LegacyDefaults;

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
}
