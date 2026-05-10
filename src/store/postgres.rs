//! PostgreSQL pool, migrations, and inserts validated via [`crate::rde`], [`crate::lineage`], [`crate::interchange`].

use std::path::Path;

use sqlx::postgres::{PgPool, PgPoolOptions, Postgres};
use sqlx::types::Json;
use sqlx::Executor;
use uuid::Uuid;

use crate::interchange::{self, InterchangeDocument};
use crate::lineage::{LineageUnit, LineageValidationError};
use crate::rde;

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
}
