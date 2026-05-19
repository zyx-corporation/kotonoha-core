//! PostgreSQL pool, migrations, and inserts validated via [`crate::rde`], [`crate::lineage`], [`crate::interchange`].

use std::path::Path;

use sqlx::postgres::{PgPool, PgPoolOptions, Postgres};
use sqlx::types::Json;
use sqlx::Executor;
use uuid::Uuid;

use crate::interchange::{self, InterchangeDocument};
use crate::lineage::{LineageUnit, LineageValidationError};
use crate::rde;
use crate::semantic_lineage::{MeaningDeltaInput, RecordReviewDecisionInput, SemanticLineageError};

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

    /// Stores an RDE evaluation JSONB on `rde_assessments`.
    ///
    /// When `payload` contains `rde_review_output`, validates via [`rde::validate_json`].
    /// If `materialize_rde_document` is true and validation succeeds, also inserts `rde_documents`
    /// and sets `rde_assessments.rde_document_id`.
    pub async fn attach_rde_assessment(
        &self,
        meaning_delta_id: Uuid,
        payload: serde_json::Value,
        strict_rde: bool,
        audit_correlation_id: Option<&str>,
        materialize_rde_document: bool,
    ) -> Result<Uuid, StoreError> {
        if payload.get("rde_review_output").is_some() {
            let json = serde_json::to_string(&payload)?;
            rde::validate_json(&json, strict_rde).map_err(StoreError::RdeValidation)?;
        } else if !payload.is_object() {
            return Err(StoreError::RdeValidation(
                "rde assessment payload must be a JSON object".into(),
            ));
        }

        let mut tx = self.pool.begin().await?;
        let assessment_id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO rde_assessments (meaning_delta_id, payload, audit_correlation_id)
               VALUES ($1, $2::jsonb, $3)
               RETURNING id"#,
        )
        .bind(meaning_delta_id)
        .bind(Json(payload.clone()))
        .bind(audit_correlation_id)
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
}

fn meaning_delta_row_from_pg(row: sqlx::postgres::PgRow) -> MeaningDeltaRow {
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
            .attach_rde_assessment(delta_id, rde_payload, false, Some(&subject), true)
            .await
            .expect("attach_rde_assessment");

        store
            .record_review_decision(&RecordReviewDecisionInput {
                meaning_delta_id: delta_id,
                rde_assessment_id: Some(assessment_id),
                decision: ReviewDecisionKind::Approve,
                decided_by: "integration-test".into(),
                rationale: serde_json::json!({ "note": "ok" }),
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

        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM review_decisions WHERE meaning_delta_id = $1",
        )
        .bind(delta_id)
        .fetch_one(store.pool())
        .await
        .unwrap();
        assert_eq!(n, 1);
    }
}
