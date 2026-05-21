//! M5 AgentRun persistence ([`PgStore`] extensions).
//!
//! Issue: <https://github.com/zyx-corporation/kotonoha-core/issues/33>

use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

use super::postgres::{PgStore, StoreError};
use super::principals::{OperationContext, ProjectMemberRole};

/// Allowed `agent_runs.status` values (matches migration check constraint).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunStatus {
    Started,
    Completed,
    Failed,
    Denied,
}

impl AgentRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Denied => "denied",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "started" => Some(Self::Started),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "denied" => Some(Self::Denied),
            _ => None,
        }
    }
}

/// `agent_runs` row (M1 columns + M5 extension).
#[derive(Debug, Clone)]
pub struct AgentRunRow {
    pub id: Uuid,
    pub agent_kind: String,
    pub external_ref: Option<String>,
    pub payload: Value,
    pub capability_profile: Option<String>,
    pub parent_run_id: Option<Uuid>,
    pub status: String,
    pub output_artifact_refs: Value,
    pub denied_actions: Value,
    pub principal_id: Option<uuid::Uuid>,
}

/// Input to start an AgentRun.
#[derive(Debug, Clone)]
pub struct StartAgentRunInput {
    pub agent_kind: String,
    pub external_ref: Option<String>,
    pub capability_profile: Option<String>,
    pub parent_run_id: Option<Uuid>,
    pub payload: Value,
    /// M6: executing principal (defaults to legacy default when unset).
    pub principal_id: Option<uuid::Uuid>,
}

/// One denied-operation audit entry stored in `denied_actions` JSONB array.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeniedActionRecord {
    pub action: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

impl PgStore {
    /// Whether M5 `agent_runs.capability_profile` column exists.
    pub async fn m5_schema_present(&self) -> Result<bool, StoreError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'agent_runs'
                  AND column_name = 'capability_profile'
            )",
        )
        .fetch_one(self.pool())
        .await?;
        Ok(exists)
    }

    /// Inserts a new AgentRun with `status = started`.
    pub async fn start_agent_run(
        &self,
        input: &StartAgentRunInput,
    ) -> Result<AgentRunRow, StoreError> {
        let payload = if input.payload.is_null() {
            Value::Object(serde_json::Map::new())
        } else {
            input.payload.clone()
        };
        let m6 = self.m6_schema_present().await?;
        if m6 {
            let ctx = OperationContext::resolve(input.principal_id, None);
            self.require_project_role(&ctx, ProjectMemberRole::AgentRunner)
                .await?;
            let id = sqlx::query_scalar::<_, Uuid>(
                r#"INSERT INTO agent_runs (
                       agent_kind, external_ref, payload,
                       capability_profile, parent_run_id, status,
                       output_artifact_refs, denied_actions, principal_id
                   )
                   VALUES ($1, $2, $3, $4, $5, 'started', '[]'::jsonb, '[]'::jsonb, $6)
                   RETURNING id"#,
            )
            .bind(&input.agent_kind)
            .bind(&input.external_ref)
            .bind(&payload)
            .bind(&input.capability_profile)
            .bind(input.parent_run_id)
            .bind(ctx.principal_id)
            .fetch_one(self.pool())
            .await?;
            return self
                .get_agent_run(id)
                .await?
                .ok_or(StoreError::Sql(sqlx::Error::RowNotFound));
        }
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO agent_runs (
                   agent_kind, external_ref, payload,
                   capability_profile, parent_run_id, status,
                   output_artifact_refs, denied_actions
               )
               VALUES ($1, $2, $3, $4, $5, 'started', '[]'::jsonb, '[]'::jsonb)
               RETURNING id"#,
        )
        .bind(&input.agent_kind)
        .bind(&input.external_ref)
        .bind(&payload)
        .bind(&input.capability_profile)
        .bind(input.parent_run_id)
        .fetch_one(self.pool())
        .await?;
        self.get_agent_run(id)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }

    /// Fetches an AgentRun by id.
    pub async fn get_agent_run(&self, id: Uuid) -> Result<Option<AgentRunRow>, StoreError> {
        let m6 = self.m6_schema_present().await?;
        let row = if m6 {
            sqlx::query(
                r#"SELECT id, agent_kind, external_ref, payload,
                          capability_profile, parent_run_id, status,
                          output_artifact_refs, denied_actions, principal_id
                   FROM agent_runs WHERE id = $1"#,
            )
            .bind(id)
            .fetch_optional(self.pool())
            .await?
        } else {
            sqlx::query(
                r#"SELECT id, agent_kind, external_ref, payload,
                          capability_profile, parent_run_id, status,
                          output_artifact_refs, denied_actions,
                          NULL::uuid AS principal_id
                   FROM agent_runs WHERE id = $1"#,
            )
            .bind(id)
            .fetch_optional(self.pool())
            .await?
        };
        Ok(row.map(agent_run_row_from_pg))
    }

    /// Updates `status` (e.g. `completed`, `failed`, `denied`).
    pub async fn update_agent_run_status(
        &self,
        id: Uuid,
        status: AgentRunStatus,
    ) -> Result<AgentRunRow, StoreError> {
        sqlx::query("UPDATE agent_runs SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(status.as_str())
            .execute(self.pool())
            .await?;
        self.get_agent_run(id)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }

    /// Appends a denied-action record to `denied_actions` and sets `status = denied`.
    pub async fn append_agent_run_denied_action(
        &self,
        id: Uuid,
        record: &DeniedActionRecord,
    ) -> Result<AgentRunRow, StoreError> {
        let entry = serde_json::to_value(record).map_err(StoreError::Json)?;
        sqlx::query(
            r#"UPDATE agent_runs
               SET denied_actions = denied_actions || $2::jsonb,
                   status = 'denied'
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(entry)
        .execute(self.pool())
        .await?;
        self.get_agent_run(id)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }

    /// Replaces `output_artifact_refs` JSON array.
    pub async fn set_agent_run_output_artifacts(
        &self,
        id: Uuid,
        refs: &Value,
    ) -> Result<AgentRunRow, StoreError> {
        sqlx::query("UPDATE agent_runs SET output_artifact_refs = $2 WHERE id = $1")
            .bind(id)
            .bind(refs)
            .execute(self.pool())
            .await?;
        self.get_agent_run(id)
            .await?
            .ok_or(StoreError::Sql(sqlx::Error::RowNotFound))
    }
}

fn agent_run_row_from_pg(row: sqlx::postgres::PgRow) -> AgentRunRow {
    AgentRunRow {
        id: row.get("id"),
        agent_kind: row.get("agent_kind"),
        external_ref: row.get("external_ref"),
        payload: row.get("payload"),
        capability_profile: row.get("capability_profile"),
        parent_run_id: row.get("parent_run_id"),
        status: row.get("status"),
        output_artifact_refs: row.get("output_artifact_refs"),
        denied_actions: row.get("denied_actions"),
        principal_id: row.get("principal_id"),
    }
}

#[cfg(test)]
mod agent_run_status_tests {
    use super::AgentRunStatus;

    #[test]
    fn status_roundtrip() {
        assert_eq!(AgentRunStatus::Started.as_str(), "started");
        assert_eq!(
            AgentRunStatus::parse("denied"),
            Some(AgentRunStatus::Denied)
        );
        assert_eq!(AgentRunStatus::parse("invalid"), None);
    }
}

#[cfg(all(test, feature = "postgres"))]
mod agent_runs_integration_tests {
    use super::*;
    use crate::store::postgres::PgStore;
    use serde_json::json;

    fn database_url_for_integration_test() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            panic!("postgres integration tests need DATABASE_URL");
        })
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL"]
    async fn m5_agent_run_lifecycle_and_denied_action() {
        let store = PgStore::connect(&database_url_for_integration_test())
            .await
            .expect("connect");
        store.migrate().await.expect("migrate");
        assert!(store.m5_schema_present().await.expect("m5 check"));

        let run = store
            .start_agent_run(&StartAgentRunInput {
                agent_kind: "chatgpt-mcp".into(),
                external_ref: Some("thread-test".into()),
                capability_profile: Some("kotonoha-agent".into()),
                parent_run_id: None,
                payload: json!({"channel": "m5-itest"}),
                principal_id: None,
            })
            .await
            .expect("start");
        assert_eq!(run.status, "started");

        let denied = store
            .append_agent_run_denied_action(
                run.id,
                &DeniedActionRecord {
                    action: "review approve".into(),
                    reason: "capability_profile".into(),
                    profile: Some("kotonoha-agent".into()),
                },
            )
            .await
            .expect("deny");
        assert_eq!(denied.status, "denied");
        assert!(denied.denied_actions.as_array().unwrap().len() >= 1);

        let completed = store
            .update_agent_run_status(run.id, AgentRunStatus::Completed)
            .await
            .expect("complete");
        assert_eq!(completed.status, "completed");
    }
}
