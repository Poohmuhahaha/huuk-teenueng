//! PostgreSQL connection pool, migrations, health probes, and state persistence.

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool, Row};

use crate::store::Store;

/// Initializes the PostgreSQL connection pool with strict connection limits
/// and statement/lock timeouts per Master Engineering SOP §3.4.
pub async fn init_pool(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET statement_timeout = '5000ms'; SET lock_timeout = '3000ms';")
                    .await?;
                Ok(())
            })
        })
        .connect(db_url)
        .await
}

/// Runs embedded SQLx database migrations from `./migrations`.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Executes a lightweight `SELECT 1` liveness/readiness query for `/healthz`.
pub async fn check_health(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}

/// Loads the persisted store snapshot from PostgreSQL when present.
pub async fn load_store_from_db(pool: &PgPool) -> Result<Option<Store>, String> {
    let row = sqlx::query("SELECT payload FROM app_state_snapshots WHERE id = 1")
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("failed to load store snapshot from postgres: {e}"))?;

    let Some(row) = row else {
        return Ok(None);
    };

    let payload: serde_json::Value = row
        .try_get("payload")
        .map_err(|e| format!("failed to decode snapshot payload column: {e}"))?;
    let bytes =
        serde_json::to_vec(&payload).map_err(|e| format!("failed to serialize payload: {e}"))?;
    let mut store = Store::from_snapshot_bytes(&bytes)?;
    store.db = Some(pool.clone());
    Ok(Some(store))
}

/// Persists serialized store snapshot JSON bytes into PostgreSQL (`app_state_snapshots`).
pub async fn save_snapshot_bytes_to_db(pool: &PgPool, bytes: &[u8]) -> Result<(), String> {
    let payload: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("invalid snapshot JSON: {e}"))?;

    sqlx::query(
        r#"
        INSERT INTO app_state_snapshots (id, version, payload, updated_at)
        VALUES (1, 2, $1, NOW())
        ON CONFLICT (id) DO UPDATE
        SET version = EXCLUDED.version,
            payload = EXCLUDED.payload,
            updated_at = NOW()
        "#,
    )
    .bind(payload)
    .execute(pool)
    .await
    .map_err(|e| format!("failed to persist store snapshot to postgres: {e}"))?;

    Ok(())
}
