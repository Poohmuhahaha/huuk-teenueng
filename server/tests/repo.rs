//! Round-trip tests for the normalized PostgreSQL projection.
//!
//! The test needs a PostgreSQL database. In CI `DATABASE_URL` points at the
//! service container; locally the test is skipped when the variable is absent.

use content_planner_server::{db, repo};
use serde_json::Value;

/// The tests share one database, so serialize them instead of racing on the
/// truncate at the start of each test.
static DB_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn pool() -> Option<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = db::init_pool(&url).await.expect("connect to DATABASE_URL");
    db::run_migrations(&pool)
        .await
        .expect("run migrations for repo test");
    sqlx::query(
        r#"
        TRUNCATE app_settings, users, workspaces, workspace_members, sessions,
                 posts, post_platforms, ideas, hashtag_groups, metrics,
                 finance_entries, content_items, content_revisions, campaigns,
                 campaign_content, platform_connections, platform_tokens,
                 media_assets, outbox_events, store_meta, app_state_snapshots
        CASCADE
        "#,
    )
    .execute(&pool)
    .await
    .expect("truncate normalized tables");
    Some(pool)
}

fn sort_key(value: &Value) -> String {
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .or_else(|| value.get("email").and_then(Value::as_str))
        .or_else(|| value.get("postId").and_then(Value::as_str))
        .unwrap_or_default();
    let platform = value
        .get("platform")
        .and_then(Value::as_str)
        .unwrap_or_default();
    format!("{id}:{platform}")
}

fn order(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|rows| rows.iter().map(sort_key).collect())
        .unwrap_or_default()
}

fn normalize(bytes: &[u8]) -> Value {
    let mut snapshot: Value = serde_json::from_slice(bytes).expect("snapshot JSON");
    for key in [
        "posts",
        "ideas",
        "hashtags",
        "metrics",
        "txns",
        "workspaces",
        "content",
        "campaigns",
        "accounts",
    ] {
        if let Some(rows) = snapshot.get_mut(key).and_then(Value::as_array_mut) {
            rows.sort_by_key(sort_key);
        }
    }
    if let Some(rows) = snapshot.get_mut("sessions").and_then(Value::as_array_mut) {
        rows.sort_by_key(|row| {
            row.as_array()
                .and_then(|pair| pair.first())
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        });
    }
    snapshot
}

#[tokio::test]
async fn snapshot_round_trips_through_normalized_tables() {
    let _guard = DB_LOCK.lock().await;
    let Some(pool) = pool().await else {
        eprintln!("skipping repo test: DATABASE_URL is not set");
        return;
    };

    let store = content_planner_server::store::Store::seed();
    let original = store.snapshot_bytes().expect("serialize seed snapshot");

    repo::sync_snapshot(&pool, &original)
        .await
        .expect("sync seed snapshot");
    let loaded = repo::load_store(&pool)
        .await
        .expect("load normalized store")
        .expect("normalized marker present");
    let round_trip = loaded.snapshot_bytes().expect("serialize loaded store");
    let original_value: Value = serde_json::from_slice(&original).expect("original snapshot JSON");
    let round_trip_value: Value =
        serde_json::from_slice(&round_trip).expect("round-trip snapshot JSON");

    for key in [
        "workspaces",
        "posts",
        "ideas",
        "hashtags",
        "metrics",
        "txns",
        "content",
        "campaigns",
    ] {
        assert_eq!(
            order(&original_value, key),
            order(&round_trip_value, key),
            "order for {key} changed"
        );
    }
    assert_eq!(normalize(&original), normalize(&round_trip));

    // Promoted columns (not just the JSON payload) must be populated.
    let email = loaded
        .accounts
        .values()
        .next()
        .expect("seed account")
        .email
        .clone();
    let (pw_hash,): (String,) = sqlx::query_as("SELECT pw_hash FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .expect("read projected pw_hash");
    assert!(!pw_hash.is_empty(), "projected password hash is empty");

    // Re-syncing the same snapshot must be idempotent.
    repo::sync_snapshot(&pool, &round_trip)
        .await
        .expect("sync loaded snapshot");
    let again = repo::load_store(&pool)
        .await
        .expect("load normalized store again")
        .expect("normalized marker present");
    assert_eq!(
        normalize(&round_trip),
        normalize(&again.snapshot_bytes().expect("serialize second load"))
    );
}

#[tokio::test]
async fn deleting_an_entity_prunes_its_normalized_row() {
    let _guard = DB_LOCK.lock().await;
    let Some(pool) = pool().await else {
        eprintln!("skipping repo test: DATABASE_URL is not set");
        return;
    };

    let mut store = content_planner_server::store::Store::seed();
    assert!(!store.posts.is_empty(), "seed must contain posts");
    let original = store.snapshot_bytes().expect("serialize seed snapshot");
    repo::sync_snapshot(&pool, &original)
        .await
        .expect("sync seed snapshot");

    let removed = store.posts.pop().expect("a post to remove");
    store.post_workspace.remove(&removed.id);
    let reduced = store.snapshot_bytes().expect("serialize reduced store");
    repo::sync_snapshot(&pool, &reduced)
        .await
        .expect("sync reduced snapshot");

    let loaded = repo::load_store(&pool)
        .await
        .expect("load normalized store")
        .expect("normalized marker present");
    assert_eq!(loaded.posts.len(), store.posts.len());
    assert!(!loaded.posts.iter().any(|post| post.id == removed.id));
}
