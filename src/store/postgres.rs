//! PostgreSQL pool, migrations, and inserts validated via [`crate::rde`], [`crate::lineage`], [`crate::interchange`].

use std::path::Path;

use sqlx::postgres::{PgPool, PgPoolOptions, Postgres};
use sqlx::types::Json;
use sqlx::Executor;
use uuid::Uuid;

use crate::interchange::{self, InterchangeDocument};
use crate::lineage::{LineageUnit, LineageValidationError};
use crate::rde;
use crate::rde_attach::{
    build_validation_report, payload_schema_version_from_payload, validate_rde_payload_for_attach,
    RdeSourceKind,
};
use crate::semantic_lineage::{MeaningDeltaInput, RecordReviewDecisionInput, SemanticLineageError};

use super::principals::{OperationContext, ProjectMemberRole};

/// Pool wrapper around the v0 schema (`migrations/`).
#[derive(Debug, Clone)]
pub struct PgStore {
    pool: PgPool,
}

/// Errors from validation before insert or database I/O.
#[derive(Debug)]
pub enum StoreError {
    RdeValidation(String),
    InterchangeValidation(String),
    Lineage(LineageValidationError),
    SemanticLineage(SemanticLineageError),
    Sql(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
    MissingField(&'static str),
    Json(serde_json::Error),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::RdeValidation(s) => f.write_str(s),
            StoreError::InterchangeValidation(s) => f.write_str(s),
            StoreError::Lineage(LineageValidationError::EmptyId) => {
                f.write_str("lineage unit id must not be empty")
            }
            StoreError::SemanticLineage(e) => write!(f, "{e}"),
            StoreError::Sql(e) => write!(f, "{e}"),
            StoreError::Migrate(e) => write!(f, "{e}"),
            StoreError::MissingField(name) => write!(f, "missing JSON field: {name}"),
            StoreError::Json(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<sqlx::Error> for StoreError {
    fn from(e: sqlx::Error) -> Self {
        StoreError::Sql(e)
    }
}

impl From<sqlx::migrate::MigrateError> for StoreError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        StoreError::Migrate(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Json(e)
    }
}

/// Optional M2 metadata for [`PgStore::attach_rde_assessment`].
#[derive(Debug, Clone, Default)]
pub struct AttachRdeAssessmentMeta {
    pub source_kind: Option<RdeSourceKind>,
    pub payload_schema_version: Option<String>,
    pub validation_report: Option<serde_json::Value>,
}

/// Result of [`PgStore::validate_and_attach_rde`].
#[derive(Debug, Clone)]
pub struct ValidateAndAttachRdeResult {
    pub assessment_id: Uuid,
    pub validation_report: serde_json::Value,
    pub payload_schema_version: Option<String>,
    pub source_kind: RdeSourceKind,
}

fn validate_rde_payload(
    payload: &serde_json::Value,
    strict_rde: bool,
) -> Result<Vec<String>, StoreError> {
    validate_rde_payload_for_attach(payload, strict_rde).map_err(StoreError::RdeValidation)
}

async fn insert_lineage_unit_ex<'e, E>(e: E, unit: &LineageUnit) -> Result<(), StoreError>
where
    E: Executor<'e, Database = Postgres>,
{
    unit.validate().map_err(StoreError::Lineage)?;
    sqlx::query("INSERT INTO lineage_units (id, prior_unit_id) VALUES ($1, $2)")
        .bind(&unit.id)
        .bind(&unit.prior_unit_id)
        .execute(e)
        .await?;
    Ok(())
}

async fn insert_rde_document_value_ex<'e, E>(
    e: E,
    document: serde_json::Value,
    strict_rde: bool,
) -> Result<Uuid, StoreError>
where
    E: Executor<'e, Database = Postgres>,
{
    let json = serde_json::to_string(&document)?;
    rde::validate_json(&json, strict_rde).map_err(StoreError::RdeValidation)?;
    let (subject_ref, spec_version) = {
        let inner = document
            .get("rde_review_output")
            .ok_or(StoreError::MissingField("rde_review_output"))?;
        let subject = inner
            .get("subject_ref")
            .and_then(|x| x.as_str())
            .ok_or(StoreError::MissingField("subject_ref"))?
            .to_string();
        let spec = inner
            .get("spec_version")
            .and_then(|x| x.as_str())
            .ok_or(StoreError::MissingField("spec_version"))?
            .to_string();
        (subject, spec)
    };
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO rde_documents (subject_ref, spec_version, payload)
           VALUES ($1, $2, $3::jsonb)
           RETURNING id"#,
    )
    .bind(subject_ref)
    .bind(spec_version)
    .bind(Json(document))
    .fetch_one(e)
    .await?;
    Ok(id)
}

impl PgStore {
    /// Opens a pool against `DATABASE_URL` (or any Postgres URL accepted by SQLx).
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Runs SQL files from the crate `migrations/` directory (see SQLx `Migrator`).
    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let migrator = sqlx::migrate::Migrator::new(dir).await?;
        migrator.run(&self.pool).await
    }

    /// Inserts a lineage row after [`LineageUnit::validate`]. Caller must order inserts so
    /// `prior_unit_id` references already-present rows when set.
    pub async fn insert_lineage_unit(&self, unit: &LineageUnit) -> Result<(), StoreError> {
        insert_lineage_unit_ex(&self.pool, unit).await
    }

    /// Validates with [`rde::validate_json`] (`strict = true`), then stores the full document JSON in `payload`.
    pub async fn insert_rde_document_json(&self, json: &str) -> Result<Uuid, StoreError> {
        let v: serde_json::Value = serde_json::from_str(json)?;
        insert_rde_document_value_ex(&self.pool, v, true).await
    }

    /// Validates with [`interchange::validate_interchange_json`], inserts the full envelope into
    /// **`interchange_documents`**, then **in the same transaction** materializes optional
    /// `lineage_unit` and `rde_document` into **`lineage_units`** and **`rde_documents`** when
    /// present. Nested RDE uses the same `strict_rde` flag as interchange validation.
    pub async fn insert_interchange_document_json(
        &self,
        json: &str,
        strict_rde: bool,
    ) -> Result<Uuid, StoreError> {
        interchange::validate_interchange_json(json, strict_rde)
            .map_err(StoreError::InterchangeValidation)?;
        let doc: InterchangeDocument = serde_json::from_str(json)?;
        let envelope: serde_json::Value = serde_json::from_str(json)?;

        let mut tx = self.pool.begin().await?;

        let interchange_id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO interchange_documents (payload) VALUES ($1::jsonb) RETURNING id"#,
        )
        .bind(Json(envelope))
        .fetch_one(&mut *tx)
        .await?;

        if let Some(ref u) = doc.lineage_unit {
            insert_lineage_unit_ex(&mut *tx, u).await?;
        }

        if let Some(v) = doc.rde_document {
            insert_rde_document_value_ex(&mut *tx, v, strict_rde).await?;
        }

        tx.commit().await?;
        Ok(interchange_id)
    }

    /// Append-only audit row (`audit_events`).
    pub async fn insert_audit_event(
        &self,
        event_type: &str,
        correlation_ref: Option<&str>,
        payload: serde_json::Value,
    ) -> Result<i64, StoreError> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"INSERT INTO audit_events (event_type, correlation_ref, payload)
               VALUES ($1, $2, $3::jsonb)
               RETURNING id"#,
        )
        .bind(event_type)
        .bind(correlation_ref)
        .bind(Json(payload))
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    // --- M1 semantic lineage (#22) ---

    /// Inserts a row into `meaning_deltas` after [`MeaningDeltaInput::validate`].
    pub async fn create_meaning_delta(
        &self,
        input: &MeaningDeltaInput,
    ) -> Result<Uuid, StoreError> {
        input.validate().map_err(StoreError::SemanticLineage)?;
        let a = &input.git_anchor;
        let m6 = self.m6_schema_present().await?;
        if m6 {
            let ctx = OperationContext::resolve(input.acting_principal_id, input.project_id);
            if let Some(run_id) = input.agent_run_id {
                self.require_project_role(&ctx, ProjectMemberRole::AgentRunner)
                    .await?;
                self.require_agent_run_principal(run_id, ctx.principal_id)
                    .await?;
            } else {
                self.require_project_role_any(
                    &ctx,
                    &[
                        ProjectMemberRole::Owner,
                        ProjectMemberRole::Reviewer,
                        ProjectMemberRole::AgentRunner,
                    ],
                )
                .await?;
            }
            let id = sqlx::query_scalar::<_, Uuid>(
                r#"INSERT INTO meaning_deltas (
                    document_object_id, prior_meaning_state_id, new_meaning_state_id,
                    agent_run_id, git_commit, file_path, line_range_start, line_range_end,
                    diff_ref, observation, source_context, project_id
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::jsonb, $11::jsonb, $12)
                RETURNING id"#,
            )
            .bind(input.document_object_id)
            .bind(input.prior_meaning_state_id)
            .bind(input.new_meaning_state_id)
            .bind(input.agent_run_id)
            .bind(&a.git_commit)
            .bind(&a.file_path)
            .bind(a.line_range_start)
            .bind(a.line_range_end)
            .bind(&a.diff_ref)
            .bind(Json(input.observation.clone()))
            .bind(Json(input.source_context.clone()))
            .bind(ctx.project_id)
            .fetch_one(&self.pool)
            .await?;
            return Ok(id);
        }
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO meaning_deltas (
                document_object_id, prior_meaning_state_id, new_meaning_state_id,
                agent_run_id, git_commit, file_path, line_range_start, line_range_end,
                diff_ref, observation, source_context
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::jsonb, $11::jsonb)
            RETURNING id"#,
        )
        .bind(input.document_object_id)
        .bind(input.prior_meaning_state_id)
        .bind(input.new_meaning_state_id)
        .bind(input.agent_run_id)
        .bind(&a.git_commit)
        .bind(&a.file_path)
        .bind(a.line_range_start)
        .bind(a.line_range_end)
        .bind(&a.diff_ref)
        .bind(Json(input.observation.clone()))
        .bind(Json(input.source_context.clone()))
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    /// Validates RDE JSON, builds a [`validation_report`](AttachRdeAssessmentMeta::validation_report), and inserts in one transaction.
    ///
    /// When `strict_rde` is true, interchange `SHOULD` warnings cause attach to fail (no row inserted).
    pub async fn validate_and_attach_rde(
        &self,
        meaning_delta_id: Uuid,
        payload: serde_json::Value,
        strict_rde: bool,
        source_kind: RdeSourceKind,
        audit_correlation_id: Option<&str>,
        materialize_rde_document: bool,
        acting_principal_id: Option<Uuid>,
    ) -> Result<ValidateAndAttachRdeResult, StoreError> {
        let warnings = validate_rde_payload(&payload, strict_rde)?;
        let validation_report = build_validation_report(strict_rde, &warnings);
        let payload_schema_version = payload_schema_version_from_payload(&payload);
        let meta = AttachRdeAssessmentMeta {
            source_kind: Some(source_kind),
            payload_schema_version: payload_schema_version.clone(),
            validation_report: Some(validation_report.clone()),
        };
        let assessment_id = self
            .attach_rde_assessment(
                meaning_delta_id,
                payload,
                strict_rde,
                audit_correlation_id,
                materialize_rde_document,
                Some(meta),
                acting_principal_id,
            )
            .await?;
        Ok(ValidateAndAttachRdeResult {
            assessment_id,
            validation_report,
            payload_schema_version,
            source_kind,
        })
    }

    /// Stores an RDE evaluation JSONB on `rde_assessments`.
    ///
    /// When `payload` contains `rde_review_output`, validates via [`rde::validate_json`].
    /// If `materialize_rde_document` is true and validation succeeds, also inserts `rde_documents`
    /// and sets `rde_assessments.rde_document_id`.
    ///
    /// Pass [`AttachRdeAssessmentMeta`] when M2 columns are populated (see [`Self::validate_and_attach_rde`]).
    pub async fn attach_rde_assessment(
        &self,
        meaning_delta_id: Uuid,
        payload: serde_json::Value,
        strict_rde: bool,
        audit_correlation_id: Option<&str>,
        materialize_rde_document: bool,
        meta: Option<AttachRdeAssessmentMeta>,
        acting_principal_id: Option<Uuid>,
    ) -> Result<Uuid, StoreError> {
        if self.m6_schema_present().await? {
            let project_id = self.meaning_delta_project_id(meaning_delta_id).await?;
            let ctx = OperationContext::resolve(acting_principal_id, Some(project_id));
            self.require_project_role(&ctx, ProjectMemberRole::AgentRunner)
                .await?;
        }
        if payload.get("rde_review_output").is_some() {
            validate_rde_payload(&payload, strict_rde)?;
        } else if !payload.is_object() {
            return Err(StoreError::RdeValidation(
                "rde assessment payload must be a JSON object".into(),
            ));
        }

        let source_kind = meta
            .as_ref()
            .and_then(|m| m.source_kind)
            .map(|k| k.as_db_str().to_string());
        let payload_schema_version = meta
            .as_ref()
            .and_then(|m| m.payload_schema_version.clone())
            .or_else(|| payload_schema_version_from_payload(&payload));
        let validation_report = meta.as_ref().and_then(|m| m.validation_report.clone());

        let mut tx = self.pool.begin().await?;
        let assessment_id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO rde_assessments (
                meaning_delta_id, payload, audit_correlation_id,
                payload_schema_version, source_kind, validation_report
            ) VALUES ($1, $2::jsonb, $3, $4, $5, $6::jsonb)
            RETURNING id"#,
        )
        .bind(meaning_delta_id)
        .bind(Json(payload.clone()))
        .bind(audit_correlation_id)
        .bind(payload_schema_version)
        .bind(source_kind)
        .bind(validation_report.map(Json))
        .fetch_one(&mut *tx)
        .await?;

        if materialize_rde_document && payload.get("rde_review_output").is_some() {
            let doc_id = insert_rde_document_value_ex(&mut *tx, payload, strict_rde).await?;
            sqlx::query("UPDATE rde_assessments SET rde_document_id = $1 WHERE id = $2")
                .bind(doc_id)
                .bind(assessment_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(assessment_id)
    }

    /// Records a human or institutional review decision.
    pub async fn record_review_decision(
        &self,
        input: &RecordReviewDecisionInput,
    ) -> Result<Uuid, StoreError> {
        input.validate().map_err(StoreError::SemanticLineage)?;
        let m6 = self.m6_schema_present().await?;
        if m6 {
            let project_id = self
                .meaning_delta_project_id(input.meaning_delta_id)
                .await?;
            let ctx = OperationContext::resolve(input.principal_id, Some(project_id));
            self.require_project_role(&ctx, ProjectMemberRole::Reviewer)
                .await?;
            let id = sqlx::query_scalar::<_, Uuid>(
                r#"INSERT INTO review_decisions (
                    meaning_delta_id, rde_assessment_id, decision, decided_by, rationale,
                    principal_id
                ) VALUES ($1, $2, $3, $4, $5::jsonb, $6)
                RETURNING id"#,
            )
            .bind(input.meaning_delta_id)
            .bind(input.rde_assessment_id)
            .bind(input.decision.as_db_str())
            .bind(&input.decided_by)
            .bind(Json(input.rationale.clone()))
            .bind(ctx.principal_id)
            .fetch_one(&self.pool)
            .await?;
            return Ok(id);
        }
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO review_decisions (
                meaning_delta_id, rde_assessment_id, decision, decided_by, rationale
            ) VALUES ($1, $2, $3, $4, $5::jsonb)
            RETURNING id"#,
        )
        .bind(input.meaning_delta_id)
        .bind(input.rde_assessment_id)
        .bind(input.decision.as_db_str())
        .bind(&input.decided_by)
        .bind(Json(input.rationale.clone()))
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    /// Fetches one meaning delta by id.
    pub async fn get_meaning_delta(&self, id: Uuid) -> Result<Option<MeaningDeltaRow>, StoreError> {
        let row = sqlx::query(
            r#"SELECT id, git_commit, file_path, line_range_start, line_range_end, diff_ref,
                      observation, source_context
               FROM meaning_deltas WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(meaning_delta_row_from_pg))
    }

    /// Whether M2 `rde_assessments.source_kind` column exists.
    pub async fn m2_schema_present(&self) -> Result<bool, StoreError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'rde_assessments'
                  AND column_name = 'source_kind'
            )",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    /// Whether M1 `meaning_deltas` table exists (migration applied).
    pub async fn m1_schema_present(&self) -> Result<bool, StoreError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = 'meaning_deltas'
            )",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    /// Count rows in `meaning_deltas` (0 if table missing — call `m1_schema_present` first).
    pub async fn count_meaning_deltas(&self) -> Result<i64, StoreError> {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM meaning_deltas")
            .fetch_one(&self.pool)
            .await?;
        Ok(n)
    }

    /// Lists meaning deltas anchored to a Git commit (newest first).
    pub async fn list_meaning_deltas_by_git_commit(
        &self,
        git_commit: &str,
    ) -> Result<Vec<MeaningDeltaRow>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT id, git_commit, file_path, line_range_start, line_range_end, diff_ref,
                      observation, source_context
               FROM meaning_deltas
               WHERE git_commit = $1
               ORDER BY created_at DESC"#,
        )
        .bind(git_commit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(meaning_delta_row_from_pg).collect())
    }

    /// Lists RDE assessments for a meaning delta (newest first).
    pub async fn list_rde_assessments_for_meaning_delta(
        &self,
        meaning_delta_id: Uuid,
    ) -> Result<Vec<RdeAssessmentRow>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT id, meaning_delta_id, payload, audit_correlation_id, rde_document_id,
                      payload_schema_version, source_kind, validation_report
               FROM rde_assessments
               WHERE meaning_delta_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(meaning_delta_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(rde_assessment_row_from_pg).collect())
    }

    /// Lists review decisions for a meaning delta (newest first).
    pub async fn list_review_decisions_for_meaning_delta(
        &self,
        meaning_delta_id: Uuid,
    ) -> Result<Vec<ReviewDecisionRow>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT id, meaning_delta_id, rde_assessment_id, decision, decided_by, rationale
               FROM review_decisions
               WHERE meaning_delta_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(meaning_delta_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(review_decision_row_from_pg).collect())
    }
}

pub(crate) fn meaning_delta_row_from_pg(row: sqlx::postgres::PgRow) -> MeaningDeltaRow {
    use sqlx::Row;
    MeaningDeltaRow {
        id: row.get("id"),
        git_commit: row.get("git_commit"),
        file_path: row.get("file_path"),
        line_range_start: row.get("line_range_start"),
        line_range_end: row.get("line_range_end"),
        diff_ref: row.get("diff_ref"),
        observation: row.get::<Json<serde_json::Value>, _>("observation").0,
        source_context: row.get::<Json<serde_json::Value>, _>("source_context").0,
    }
}

/// Row returned by [`PgStore::get_meaning_delta`] and list helpers.
#[derive(Debug, Clone)]
pub struct MeaningDeltaRow {
    pub id: Uuid,
    pub git_commit: String,
    pub file_path: String,
    pub line_range_start: Option<i32>,
    pub line_range_end: Option<i32>,
    pub diff_ref: Option<String>,
    pub observation: serde_json::Value,
    pub source_context: serde_json::Value,
}

/// Row returned by [`PgStore::list_rde_assessments_for_meaning_delta`].
#[derive(Debug, Clone)]
pub struct RdeAssessmentRow {
    pub id: Uuid,
    pub meaning_delta_id: Uuid,
    pub payload: serde_json::Value,
    pub audit_correlation_id: Option<String>,
    pub rde_document_id: Option<Uuid>,
    pub payload_schema_version: Option<String>,
    pub source_kind: Option<String>,
    pub validation_report: Option<serde_json::Value>,
}

/// Row returned by [`PgStore::list_review_decisions_for_meaning_delta`].
#[derive(Debug, Clone)]
pub struct ReviewDecisionRow {
    pub id: Uuid,
    pub meaning_delta_id: Uuid,
    pub rde_assessment_id: Option<Uuid>,
    pub decision: String,
    pub decided_by: String,
    pub rationale: serde_json::Value,
}

fn rde_assessment_row_from_pg(row: sqlx::postgres::PgRow) -> RdeAssessmentRow {
    use sqlx::Row;
    let validation_report = row
        .try_get::<Option<Json<serde_json::Value>>, _>("validation_report")
        .ok()
        .flatten()
        .map(|j| j.0);
    RdeAssessmentRow {
        id: row.get("id"),
        meaning_delta_id: row.get("meaning_delta_id"),
        payload: row.get::<Json<serde_json::Value>, _>("payload").0,
        audit_correlation_id: row.get("audit_correlation_id"),
        rde_document_id: row.get("rde_document_id"),
        payload_schema_version: row.try_get("payload_schema_version").unwrap_or(None),
        source_kind: row.try_get("source_kind").unwrap_or(None),
        validation_report,
    }
}

fn review_decision_row_from_pg(row: sqlx::postgres::PgRow) -> ReviewDecisionRow {
    use sqlx::Row;
    ReviewDecisionRow {
        id: row.get("id"),
        meaning_delta_id: row.get("meaning_delta_id"),
        rde_assessment_id: row.get("rde_assessment_id"),
        decision: row.get("decision"),
        decided_by: row.get("decided_by"),
        rationale: row.get::<Json<serde_json::Value>, _>("rationale").0,
    }
}

#[cfg(all(test, feature = "postgres"))]
mod postgres_integration_tests {
    use super::*;
    use crate::interchange::INTERCHANGE_FORMAT_V1;
    use crate::TARGET_SPEC_BUNDLE;

    fn database_url_for_integration_test() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            panic!(
                "postgres integration tests need DATABASE_URL when running with --include-ignored; \
                 see CONTRIBUTING.md (e.g. docker compose then export DATABASE_URL=...)"
            );
        })
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL (run with: cargo test --features postgres -- --include-ignored)"]
    async fn migrate_inserts_interchange_and_lineage_row() {
        let url = database_url_for_integration_test();
        let store = PgStore::connect(&url).await.expect("connect");
        store.migrate().await.expect("migrate");

        let lid = format!("https://example.invalid/pg-itest/{}", uuid::Uuid::new_v4());
        let json = serde_json::json!({
            "format": INTERCHANGE_FORMAT_V1,
            "spec_bundle": TARGET_SPEC_BUNDLE,
            "lineage_unit": { "id": lid, "prior_unit_id": null },
        })
        .to_string();

        let interchange_id = store
            .insert_interchange_document_json(&json, false)
            .await
            .expect("insert_interchange_document_json");

        let n: i64 =
            sqlx::query_scalar("SELECT COUNT(*)::bigint FROM interchange_documents WHERE id = $1")
                .bind(interchange_id)
                .fetch_one(store.pool())
                .await
                .expect("count interchange_documents");
        assert_eq!(n, 1, "interchange_documents row");

        let n: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM lineage_units WHERE id = $1")
            .bind(&lid)
            .fetch_one(store.pool())
            .await
            .expect("count lineage_units");
        assert_eq!(n, 1, "lineage_units materialized");
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL (run with: cargo test --features postgres -- --include-ignored)"]
    async fn migrate_inserts_interchange_lineage_and_rde_rows() {
        let url = database_url_for_integration_test();
        let store = PgStore::connect(&url).await.expect("connect");
        store.migrate().await.expect("migrate");

        let lid = format!("https://example.invalid/pg-itest/{}", uuid::Uuid::new_v4());
        let subject = format!(
            "https://example.invalid/subject-itest/{}",
            uuid::Uuid::new_v4()
        );
        let rde = serde_json::json!({
            "rde_review_output": {
                "spec_version": TARGET_SPEC_BUNDLE,
                "subject_ref": subject,
                "categories": {
                    "preserved": [],
                    "transformed": [],
                    "complemented": [],
                    "intentionally_unresolved": [],
                    "lost": [],
                    "deviation_risk": [],
                    "next_update_policy": []
                }
            }
        });
        let json = serde_json::json!({
            "format": INTERCHANGE_FORMAT_V1,
            "spec_bundle": TARGET_SPEC_BUNDLE,
            "lineage_unit": { "id": lid, "prior_unit_id": null },
            "rde_document": rde,
        })
        .to_string();

        store
            .insert_interchange_document_json(&json, false)
            .await
            .expect("insert");

        let n: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM lineage_units WHERE id = $1")
            .bind(&lid)
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(n, 1);

        let n: i64 =
            sqlx::query_scalar("SELECT COUNT(*)::bigint FROM rde_documents WHERE subject_ref = $1")
                .bind(&subject)
                .fetch_one(store.pool())
                .await
                .unwrap();
        assert_eq!(n, 1);
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL (run with: cargo test --features postgres -- --include-ignored)"]
    async fn migrate_applies_m6_principals_tables() {
        use crate::store::LegacyDefaults;

        let url = database_url_for_integration_test();
        let store = PgStore::connect(&url).await.expect("connect");
        store.migrate().await.expect("migrate");
        assert!(store.m6_schema_present().await.expect("m6 check"));

        for table in ["principals", "projects", "project_members"] {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS (
                    SELECT 1 FROM information_schema.tables
                    WHERE table_schema = 'public' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .unwrap_or_else(|e| panic!("check table {table}: {e}"));
            assert!(exists, "expected public.{table} after M6 migration");
        }

        let (principal, project) = store.get_legacy_defaults().await.expect("defaults");
        assert_eq!(principal.id, LegacyDefaults::PRINCIPAL_ID);
        assert_eq!(project.slug, LegacyDefaults::PROJECT_SLUG);
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL (run with: cargo test --features postgres -- --include-ignored)"]
    async fn migrate_applies_m1_semantic_lineage_tables() {
        let url = database_url_for_integration_test();
        let store = PgStore::connect(&url).await.expect("connect");
        store.migrate().await.expect("migrate");

        for table in [
            "document_objects",
            "meaning_states",
            "meaning_deltas",
            "rde_assessments",
            "review_decisions",
            "agent_runs",
        ] {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS (
                    SELECT 1 FROM information_schema.tables
                    WHERE table_schema = 'public' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .unwrap_or_else(|e| panic!("check table {table}: {e}"));
            assert!(exists, "expected public.{table} after M1 migration");
        }
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL (run with: cargo test --features postgres -- --include-ignored)"]
    async fn m1_semantic_lineage_round_trip() {
        use crate::semantic_lineage::ReviewDecisionKind;
        use crate::semantic_lineage::{GitAnchor, MeaningDeltaInput, RecordReviewDecisionInput};

        let url = database_url_for_integration_test();
        let store = PgStore::connect(&url).await.expect("connect");
        store.migrate().await.expect("migrate");

        let commit = uuid::Uuid::new_v4().to_string().replace('-', "");
        let delta_id = store
            .create_meaning_delta(&MeaningDeltaInput {
                document_object_id: None,
                prior_meaning_state_id: None,
                new_meaning_state_id: None,
                agent_run_id: None,
                git_anchor: GitAnchor {
                    git_commit: commit.clone(),
                    file_path: "docs/example.md".into(),
                    line_range_start: Some(1),
                    line_range_end: Some(4),
                    diff_ref: None,
                },
                observation: serde_json::json!({
                    "preserved": ["intent"],
                    "lost": []
                }),
                source_context: serde_json::json!({}),
                project_id: None,
                acting_principal_id: None,
            })
            .await
            .expect("create_meaning_delta");

        let subject = format!("https://example.invalid/m1-itest/{}", uuid::Uuid::new_v4());
        let rde_payload = serde_json::json!({
            "rde_review_output": {
                "spec_version": TARGET_SPEC_BUNDLE,
                "subject_ref": subject,
                "categories": {
                    "preserved": [],
                    "transformed": [],
                    "complemented": [],
                    "intentionally_unresolved": [],
                    "lost": [],
                    "deviation_risk": [],
                    "next_update_policy": []
                }
            }
        });
        let assessment_id = store
            .attach_rde_assessment(
                delta_id,
                rde_payload,
                false,
                Some(&subject),
                true,
                None,
                None,
            )
            .await
            .expect("attach_rde_assessment");

        store
            .record_review_decision(&RecordReviewDecisionInput {
                meaning_delta_id: delta_id,
                rde_assessment_id: Some(assessment_id),
                decision: ReviewDecisionKind::Approve,
                decided_by: "integration-test".into(),
                rationale: serde_json::json!({ "note": "ok" }),
                principal_id: None,
            })
            .await
            .expect("record_review_decision");

        let row = store
            .get_meaning_delta(delta_id)
            .await
            .expect("get")
            .expect("row");
        assert_eq!(row.git_commit, commit);
        assert_eq!(row.file_path, "docs/example.md");

        let listed = store
            .list_meaning_deltas_by_git_commit(&commit)
            .await
            .expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, delta_id);

        let assessments = store
            .list_rde_assessments_for_meaning_delta(delta_id)
            .await
            .expect("list assessments");
        assert_eq!(assessments.len(), 1);
        assert_eq!(assessments[0].id, assessment_id);

        let decisions = store
            .list_review_decisions_for_meaning_delta(delta_id)
            .await
            .expect("list decisions");
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].decision, "approve");
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL (run with: cargo test --features postgres -- --include-ignored)"]
    async fn migrate_applies_m2_rde_meta_columns() {
        let url = database_url_for_integration_test();
        let store = PgStore::connect(&url).await.expect("connect");
        store.migrate().await.expect("migrate");
        assert!(store.m2_schema_present().await.expect("m2 check"));
        for col in ["payload_schema_version", "source_kind", "validation_report"] {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_schema = 'public'
                      AND table_name = 'rde_assessments'
                      AND column_name = $1
                )",
            )
            .bind(col)
            .fetch_one(store.pool())
            .await
            .unwrap_or_else(|e| panic!("column {col}: {e}"));
            assert!(exists, "expected rde_assessments.{col}");
        }
    }

    async fn m2_validate_and_attach_stores_meta() {
        use crate::rde_attach::RdeSourceKind;
        use crate::semantic_lineage::{GitAnchor, MeaningDeltaInput};

        let url = database_url_for_integration_test();
        let store = PgStore::connect(&url).await.expect("connect");
        store.migrate().await.expect("migrate");
        assert!(store.m2_schema_present().await.expect("m2 check"));

        let commit = uuid::Uuid::new_v4().to_string().replace('-', "");
        let delta_id = store
            .create_meaning_delta(&MeaningDeltaInput {
                document_object_id: None,
                prior_meaning_state_id: None,
                new_meaning_state_id: None,
                agent_run_id: None,
                git_anchor: GitAnchor {
                    git_commit: commit,
                    file_path: "docs/m2.md".into(),
                    line_range_start: Some(1),
                    line_range_end: Some(2),
                    diff_ref: None,
                },
                observation: serde_json::json!({ "preserved": ["intent"] }),
                source_context: serde_json::json!({}),
                project_id: None,
                acting_principal_id: None,
            })
            .await
            .expect("delta");

        let subject = format!("https://example.invalid/m2-itest/{}", uuid::Uuid::new_v4());
        let rde_payload = serde_json::json!({
            "rde_review_output": {
                "spec_version": TARGET_SPEC_BUNDLE,
                "subject_ref": subject,
                "categories": {
                    "preserved": [],
                    "transformed": [],
                    "complemented": [],
                    "intentionally_unresolved": [],
                    "lost": [],
                    "deviation_risk": [],
                    "next_update_policy": []
                }
            }
        });

        let result = store
            .validate_and_attach_rde(
                delta_id,
                rde_payload,
                false,
                RdeSourceKind::Cli,
                Some("m2-itest"),
                false,
                None,
            )
            .await
            .expect("validate_and_attach");

        let assessments = store
            .list_rde_assessments_for_meaning_delta(delta_id)
            .await
            .expect("list");
        assert_eq!(assessments.len(), 1);
        let row = &assessments[0];
        assert_eq!(row.id, result.assessment_id);
        assert_eq!(row.source_kind.as_deref(), Some("cli"));
        assert_eq!(
            row.payload_schema_version.as_deref(),
            Some(TARGET_SPEC_BUNDLE)
        );
        assert!(row.validation_report.is_some());
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL (run with: cargo test --features postgres -- --include-ignored)"]
    async fn m2_strict_attach_rolls_back_on_warnings() {
        use crate::rde_attach::RdeSourceKind;
        use crate::semantic_lineage::{GitAnchor, MeaningDeltaInput};

        let url = database_url_for_integration_test();
        let store = PgStore::connect(&url).await.expect("connect");
        store.migrate().await.expect("migrate");

        let delta_id = store
            .create_meaning_delta(&MeaningDeltaInput {
                document_object_id: None,
                prior_meaning_state_id: None,
                new_meaning_state_id: None,
                agent_run_id: None,
                git_anchor: GitAnchor {
                    git_commit: "abc".into(),
                    file_path: "f.md".into(),
                    line_range_start: Some(1),
                    line_range_end: Some(1),
                    diff_ref: None,
                },
                observation: serde_json::json!({}),
                source_context: serde_json::json!({}),
                project_id: None,
                acting_principal_id: None,
            })
            .await
            .expect("delta");

        let rde_payload = serde_json::json!({
            "rde_review_output": {
                "spec_version": TARGET_SPEC_BUNDLE,
                "subject_ref": "https://example.invalid/strict",
                "categories": {
                    "preserved": [{}],
                    "transformed": [],
                    "complemented": [],
                    "intentionally_unresolved": [],
                    "lost": [],
                    "deviation_risk": [],
                    "next_update_policy": []
                }
            }
        });

        let err = store
            .validate_and_attach_rde(
                delta_id,
                rde_payload,
                true,
                RdeSourceKind::Cli,
                None,
                false,
                None,
            )
            .await
            .expect_err("strict should fail");
        assert!(matches!(err, StoreError::RdeValidation(_)));

        let assessments = store
            .list_rde_assessments_for_meaning_delta(delta_id)
            .await
            .expect("list");
        assert!(assessments.is_empty());
    }
}
