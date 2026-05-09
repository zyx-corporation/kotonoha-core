//! PostgreSQL pool, migrations, and inserts validated via [`crate::rde`], [`crate::lineage`], [`crate::interchange`].

use std::path::Path;

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::types::Json;
use uuid::Uuid;

use crate::interchange;
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
        unit.validate().map_err(StoreError::Lineage)?;
        sqlx::query("INSERT INTO lineage_units (id, prior_unit_id) VALUES ($1, $2)")
            .bind(&unit.id)
            .bind(&unit.prior_unit_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Validates with [`rde::validate_json`] (`strict = true`), then stores the full document JSON in `payload`.
    pub async fn insert_rde_document_json(&self, json: &str) -> Result<Uuid, StoreError> {
        rde::validate_json(json, true).map_err(StoreError::RdeValidation)?;
        let v: serde_json::Value = serde_json::from_str(json)?;
        let (subject_ref, spec_version) = {
            let inner = v
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
        .bind(Json(v))
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    /// Validates with [`interchange::validate_interchange_json`], then stores the full envelope JSON in `payload`.
    pub async fn insert_interchange_document_json(
        &self,
        json: &str,
        strict_rde: bool,
    ) -> Result<Uuid, StoreError> {
        interchange::validate_interchange_json(json, strict_rde)
            .map_err(StoreError::InterchangeValidation)?;
        let v: serde_json::Value = serde_json::from_str(json)?;
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO interchange_documents (payload) VALUES ($1::jsonb) RETURNING id"#,
        )
        .bind(Json(v))
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
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
