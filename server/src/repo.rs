//! Normalized PostgreSQL persistence (Phase 1).
//!
//! The JSON snapshot stays the compatibility format and the fallback, but every
//! changed snapshot is projected into normalized tables. On boot the normalized
//! tables are preferred when `store_meta.normalized_version` exists; otherwise
//! the snapshot is loaded and re-projected on the next persist tick.
//!
//! Each entity table keeps the full entity document in `payload` JSONB and
//! promotes the columns the app filters/sorts on to real columns. That gives
//! per-entity rows and indexes without giving up the flexible model.

use std::collections::HashSet;

use serde_json::{json, Map, Value};
use sqlx::{PgPool, Postgres, Transaction};

use crate::store::Store;

/// Bump when the projection changes in a way that requires a re-sync.
const NORMALIZED_VERSION: i64 = 1;

fn s(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn opt_s(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

fn i(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn f(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn b(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn arr(v: &Value, key: &str) -> Value {
    v.get(key).cloned().unwrap_or_else(|| json!([]))
}

fn array<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn object<'a>(v: &'a Value, key: &str) -> Option<&'a Map<String, Value>> {
    v.get(key).and_then(Value::as_object)
}

fn map_workspace(map: Option<&Map<String, Value>>, id: &str, default: &str) -> String {
    map.and_then(|m| m.get(id))
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

fn is_missing_table(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db) if db.code().as_deref() == Some("42P01"))
}

/// Projects a serialized snapshot into the normalized tables in one transaction.
pub async fn sync_snapshot(pool: &PgPool, bytes: &[u8]) -> Result<(), String> {
    let snapshot: Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("invalid snapshot JSON for normalized sync: {error}"))?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| format!("cannot start normalized sync: {error}"))?;

    // The pool's default statement timeout is 5s (good for request queries).
    // A full projection can legitimately take longer, so widen it for this
    // transaction only.
    sqlx::query("SET LOCAL statement_timeout = '30s'")
        .execute(&mut *tx)
        .await
        .map_err(|error| format!("cannot configure normalized sync: {error}"))?;

    sync_in_tx(&mut tx, &snapshot)
        .await
        .map_err(|error| format!("normalized sync failed: {error}"))?;

    sqlx::query(
        r#"
        INSERT INTO store_meta (key, value, updated_at)
        VALUES ('normalized_version', $1, NOW())
        ON CONFLICT (key) DO UPDATE
        SET value = EXCLUDED.value, updated_at = NOW()
        "#,
    )
    .bind(json!(NORMALIZED_VERSION))
    .execute(&mut *tx)
    .await
    .map_err(|error| format!("cannot record normalized version: {error}"))?;

    tx.commit()
        .await
        .map_err(|error| format!("cannot commit normalized sync: {error}"))
}

async fn sync_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
) -> Result<(), sqlx::Error> {
    let settings = json!({
        "setup": snapshot.get("setup").cloned().unwrap_or_else(|| json!({})),
        "token_salt": snapshot.get("token_salt").cloned().unwrap_or_else(|| json!("")),
    });
    sqlx::query(
        r#"
        INSERT INTO app_settings (id, payload, updated_at)
        VALUES (1, $1, NOW())
        ON CONFLICT (id) DO UPDATE
        SET payload = EXCLUDED.payload, updated_at = NOW()
        "#,
    )
    .bind(settings)
    .execute(&mut **tx)
    .await?;

    let workspaces = array(snapshot, "workspaces");
    let default_workspace = workspaces
        .first()
        .map(|w| s(w, "id"))
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| crate::store::DEFAULT_WORKSPACE_ID.to_string());

    let mut workspace_ids = Vec::with_capacity(workspaces.len());
    let mut owner_emails = HashSet::new();
    for (position, workspace) in workspaces.iter().enumerate() {
        let id = s(workspace, "id");
        if id.is_empty() {
            continue;
        }
        workspace_ids.push(id.clone());
        let owner = s(workspace, "owner");
        if !owner.is_empty() {
            owner_emails.insert(owner.to_ascii_lowercase());
        }

        sqlx::query(
            r#"
            INSERT INTO workspaces (id, name, owner_email, created, position, payload, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                owner_email = EXCLUDED.owner_email,
                created = EXCLUDED.created,
                position = EXCLUDED.position,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            WHERE workspaces.payload IS DISTINCT FROM EXCLUDED.payload
               OR workspaces.name IS DISTINCT FROM EXCLUDED.name
               OR workspaces.owner_email IS DISTINCT FROM EXCLUDED.owner_email
               OR workspaces.position IS DISTINCT FROM EXCLUDED.position
            "#,
        )
        .bind(&id)
        .bind(s(workspace, "name"))
        .bind(owner)
        .bind(s(workspace, "created"))
        .bind(position as i32)
        .bind(workspace.clone())
        .execute(&mut **tx)
        .await?;

        sqlx::query("DELETE FROM workspace_members WHERE workspace_id = $1")
            .bind(&id)
            .execute(&mut **tx)
            .await?;
        if let Some(members) = workspace.get("members").and_then(Value::as_array) {
            for member in members.iter().filter_map(Value::as_str) {
                sqlx::query(
                    r#"
                    INSERT INTO workspace_members (workspace_id, email, role)
                    VALUES ($1, $2, 'member')
                    ON CONFLICT (workspace_id, email) DO NOTHING
                    "#,
                )
                .bind(&id)
                .bind(member.to_ascii_lowercase())
                .execute(&mut **tx)
                .await?;
            }
        }

        sqlx::query("DELETE FROM platform_connections WHERE workspace_id = $1")
            .bind(&id)
            .execute(&mut **tx)
            .await?;
        if let Some(connections) = workspace.get("connections").and_then(Value::as_array) {
            for connection in connections {
                let connection_id = s(connection, "id");
                if connection_id.is_empty() {
                    continue;
                }
                sqlx::query(
                    r#"
                    INSERT INTO platform_connections
                        (workspace_id, id, provider, status, handle, external_id, expires_at, payload, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
                    ON CONFLICT (workspace_id, id) DO UPDATE
                    SET provider = EXCLUDED.provider,
                        status = EXCLUDED.status,
                        handle = EXCLUDED.handle,
                        external_id = EXCLUDED.external_id,
                        expires_at = EXCLUDED.expires_at,
                        payload = EXCLUDED.payload,
                        updated_at = NOW()
                    "#,
                )
                .bind(&id)
                .bind(&connection_id)
                .bind(&connection_id)
                .bind(s(connection, "status"))
                .bind(s(connection, "handle"))
                .bind(s(connection, "externalId"))
                .bind(opt_s(connection, "expiresAt"))
                .bind(connection.clone())
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    if !workspace_ids.is_empty() {
        sqlx::query("DELETE FROM workspaces WHERE id <> ALL($1)")
            .bind(&workspace_ids)
            .execute(&mut **tx)
            .await?;
    }

    sync_accounts(tx, snapshot, &owner_emails).await?;
    sync_sessions(tx, snapshot).await?;
    sync_posts(tx, snapshot, &default_workspace).await?;
    sync_ideas(tx, snapshot, &default_workspace).await?;
    sync_hashtags(tx, snapshot, &default_workspace).await?;
    sync_metrics(tx, snapshot, &default_workspace).await?;
    sync_finance(tx, snapshot, &default_workspace).await?;
    sync_content(tx, snapshot, &default_workspace).await?;
    sync_campaigns(tx, snapshot, &default_workspace).await?;
    sync_tokens(tx, snapshot).await?;

    Ok(())
}

async fn sync_accounts(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
    owners: &HashSet<String>,
) -> Result<(), sqlx::Error> {
    let accounts = array(snapshot, "accounts");
    let mut keep = HashSet::new();
    for account in accounts {
        let email = s(account, "email").to_ascii_lowercase();
        if email.is_empty() {
            continue;
        }
        keep.insert(email.clone());
        let role = if owners.contains(&email) {
            "owner"
        } else {
            "member"
        };
        sqlx::query(
            r#"
            INSERT INTO users (email, name, role, plan, status, pw_hash, last_active, payload, updated_at)
            VALUES ($1, $2, $3, $4, 'active', $5, $6, $7, NOW())
            ON CONFLICT (email) DO UPDATE
            SET name = EXCLUDED.name,
                role = EXCLUDED.role,
                plan = EXCLUDED.plan,
                pw_hash = EXCLUDED.pw_hash,
                last_active = EXCLUDED.last_active,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            WHERE users.payload IS DISTINCT FROM EXCLUDED.payload
               OR users.role IS DISTINCT FROM EXCLUDED.role
               OR users.pw_hash IS DISTINCT FROM EXCLUDED.pw_hash
            "#,
        )
        .bind(&email)
        .bind(s(account, "name"))
        .bind(role)
        .bind(s(account, "plan"))
        .bind(s(account, "password_hash"))
        .bind(opt_s(account, "last_active"))
        .bind(account.clone())
        .execute(&mut **tx)
        .await?;
    }

    let existing: Vec<(String,)> = sqlx::query_as("SELECT email FROM users")
        .fetch_all(&mut **tx)
        .await?;
    for (email,) in existing {
        if !keep.contains(&email) {
            sqlx::query("DELETE FROM users WHERE email = $1")
                .bind(&email)
                .execute(&mut **tx)
                .await?;
        }
    }
    Ok(())
}

async fn sync_sessions(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions")
        .execute(&mut **tx)
        .await?;

    for item in array(snapshot, "sessions") {
        let Some(pair) = item.as_array() else {
            continue;
        };
        if pair.len() != 2 {
            continue;
        }
        let Some(key) = pair[0].as_str() else {
            continue;
        };
        let payload = &pair[1];
        let Some(expires) = payload
            .get("expires")
            .and_then(Value::as_str)
            .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
        else {
            continue;
        };
        sqlx::query(
            r#"
            INSERT INTO sessions (token_hash, email, expires_at, payload, updated_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (token_hash) DO UPDATE
            SET email = EXCLUDED.email,
                expires_at = EXCLUDED.expires_at,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(s(payload, "email").to_ascii_lowercase())
        .bind(expires.with_timezone(&chrono::Utc))
        .bind(payload.clone())
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn sync_posts(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
    default_workspace: &str,
) -> Result<(), sqlx::Error> {
    let map = object(snapshot, "post_workspace");
    let mut keep = HashSet::new();
    let mut platforms = Vec::new();

    for (position, post) in array(snapshot, "posts").iter().enumerate() {
        let id = s(post, "id");
        if id.is_empty() {
            continue;
        }
        let workspace = map_workspace(map, &id, default_workspace);
        keep.insert((workspace.clone(), id.clone()));

        sqlx::query(
            r#"
            INSERT INTO posts (workspace_id, id, month, year, date, status, platforms, position, payload, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            ON CONFLICT (workspace_id, id) DO UPDATE
            SET month = EXCLUDED.month,
                year = EXCLUDED.year,
                date = EXCLUDED.date,
                status = EXCLUDED.status,
                platforms = EXCLUDED.platforms,
                position = EXCLUDED.position,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            WHERE posts.payload IS DISTINCT FROM EXCLUDED.payload
            "#,
        )
        .bind(&workspace)
        .bind(&id)
        .bind(i(post, "month") as i32)
        .bind(i(post, "year") as i32)
        .bind(opt_s(post, "date"))
        .bind(s(post, "status"))
        .bind(arr(post, "platforms"))
        .bind(position as i32)
        .bind(post.clone())
        .execute(&mut **tx)
        .await?;

        if let Some(list) = post.get("platforms").and_then(Value::as_array) {
            for platform in list.iter().filter_map(Value::as_str) {
                platforms.push((workspace.clone(), id.clone(), platform.to_string()));
            }
        }
    }

    prune_by_id(tx, "posts", &keep).await?;

    sqlx::query("DELETE FROM post_platforms")
        .execute(&mut **tx)
        .await?;
    for (workspace, post, platform) in platforms {
        sqlx::query(
            r#"
            INSERT INTO post_platforms (workspace_id, post_id, platform)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(workspace)
        .bind(post)
        .bind(platform)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn sync_ideas(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
    default_workspace: &str,
) -> Result<(), sqlx::Error> {
    let map = object(snapshot, "idea_workspace");
    let mut keep = HashSet::new();
    for (position, idea) in array(snapshot, "ideas").iter().enumerate() {
        let id = s(idea, "id");
        if id.is_empty() {
            continue;
        }
        let workspace = map_workspace(map, &id, default_workspace);
        keep.insert((workspace.clone(), id.clone()));
        sqlx::query(
            r#"
            INSERT INTO ideas (workspace_id, id, topic, format, done, position, payload, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (workspace_id, id) DO UPDATE
            SET topic = EXCLUDED.topic,
                format = EXCLUDED.format,
                done = EXCLUDED.done,
                position = EXCLUDED.position,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            WHERE ideas.payload IS DISTINCT FROM EXCLUDED.payload
            "#,
        )
        .bind(&workspace)
        .bind(&id)
        .bind(s(idea, "topic"))
        .bind(s(idea, "format"))
        .bind(b(idea, "done"))
        .bind(position as i32)
        .bind(idea.clone())
        .execute(&mut **tx)
        .await?;
    }
    prune_by_id(tx, "ideas", &keep).await
}

async fn sync_hashtags(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
    default_workspace: &str,
) -> Result<(), sqlx::Error> {
    let map = object(snapshot, "hashtag_workspace");
    let mut keep = HashSet::new();
    for (position, group) in array(snapshot, "hashtags").iter().enumerate() {
        let id = s(group, "id");
        if id.is_empty() {
            continue;
        }
        let workspace = map_workspace(map, &id, default_workspace);
        keep.insert((workspace.clone(), id.clone()));
        sqlx::query(
            r#"
            INSERT INTO hashtag_groups (workspace_id, id, title, tags, position, payload, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (workspace_id, id) DO UPDATE
            SET title = EXCLUDED.title,
                tags = EXCLUDED.tags,
                position = EXCLUDED.position,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            WHERE hashtag_groups.payload IS DISTINCT FROM EXCLUDED.payload
            "#,
        )
        .bind(&workspace)
        .bind(&id)
        .bind(s(group, "title"))
        .bind(arr(group, "tags"))
        .bind(position as i32)
        .bind(group.clone())
        .execute(&mut **tx)
        .await?;
    }
    prune_by_id(tx, "hashtag_groups", &keep).await
}

async fn sync_metrics(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
    default_workspace: &str,
) -> Result<(), sqlx::Error> {
    let map = object(snapshot, "metric_workspace");
    let mut keep = HashSet::new();
    for (position, metric) in array(snapshot, "metrics").iter().enumerate() {
        let post_id = s(metric, "postId");
        let platform = s(metric, "platform");
        if post_id.is_empty() || platform.is_empty() {
            continue;
        }
        let key = format!("{post_id}:{platform}");
        let workspace = map_workspace(map, &key, default_workspace);
        keep.insert((workspace.clone(), post_id.clone(), platform.clone()));
        sqlx::query(
            r#"
            INSERT INTO metrics (workspace_id, post_id, platform, likes, views, position, payload, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (workspace_id, post_id, platform) DO UPDATE
            SET likes = EXCLUDED.likes,
                views = EXCLUDED.views,
                position = EXCLUDED.position,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            WHERE metrics.payload IS DISTINCT FROM EXCLUDED.payload
            "#,
        )
        .bind(&workspace)
        .bind(&post_id)
        .bind(&platform)
        .bind(i(metric, "likes"))
        .bind(i(metric, "views"))
        .bind(position as i32)
        .bind(metric.clone())
        .execute(&mut **tx)
        .await?;
    }

    let existing: Vec<(String, String, String)> =
        sqlx::query_as("SELECT workspace_id, post_id, platform FROM metrics")
            .fetch_all(&mut **tx)
            .await?;
    for (workspace, post_id, platform) in existing {
        if !keep.contains(&(workspace.clone(), post_id.clone(), platform.clone())) {
            sqlx::query(
                "DELETE FROM metrics WHERE workspace_id = $1 AND post_id = $2 AND platform = $3",
            )
            .bind(workspace)
            .bind(post_id)
            .bind(platform)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

async fn sync_finance(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
    default_workspace: &str,
) -> Result<(), sqlx::Error> {
    let map = object(snapshot, "txn_workspace");
    let mut keep = HashSet::new();
    for (position, txn) in array(snapshot, "txns").iter().enumerate() {
        let id = s(txn, "id");
        if id.is_empty() {
            continue;
        }
        let workspace = map_workspace(map, &id, default_workspace);
        keep.insert((workspace.clone(), id.clone()));
        sqlx::query(
            r#"
            INSERT INTO finance_entries
                (workspace_id, id, date, amount, kind, category, sub, position, payload, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            ON CONFLICT (workspace_id, id) DO UPDATE
            SET date = EXCLUDED.date,
                amount = EXCLUDED.amount,
                kind = EXCLUDED.kind,
                category = EXCLUDED.category,
                sub = EXCLUDED.sub,
                position = EXCLUDED.position,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            WHERE finance_entries.payload IS DISTINCT FROM EXCLUDED.payload
            "#,
        )
        .bind(&workspace)
        .bind(&id)
        .bind(s(txn, "date"))
        .bind(f(txn, "amount"))
        .bind(s(txn, "kind"))
        .bind(s(txn, "category"))
        .bind(s(txn, "sub"))
        .bind(position as i32)
        .bind(txn.clone())
        .execute(&mut **tx)
        .await?;
    }
    prune_by_id(tx, "finance_entries", &keep).await
}

async fn sync_content(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
    default_workspace: &str,
) -> Result<(), sqlx::Error> {
    let map = object(snapshot, "content_workspace");
    let mut keep = HashSet::new();
    let mut revisions = Vec::new();

    for (position, content) in array(snapshot, "content").iter().enumerate() {
        let id = s(content, "id");
        if id.is_empty() {
            continue;
        }
        let workspace = map_workspace(map, &id, default_workspace);
        keep.insert((workspace.clone(), id.clone()));
        sqlx::query(
            r#"
            INSERT INTO content_items
                (workspace_id, id, title, slug, kind, status, author, updated_at_text,
                 published_at, scheduled_for, position, payload, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
            ON CONFLICT (workspace_id, id) DO UPDATE
            SET title = EXCLUDED.title,
                slug = EXCLUDED.slug,
                kind = EXCLUDED.kind,
                status = EXCLUDED.status,
                author = EXCLUDED.author,
                updated_at_text = EXCLUDED.updated_at_text,
                published_at = EXCLUDED.published_at,
                scheduled_for = EXCLUDED.scheduled_for,
                position = EXCLUDED.position,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            WHERE content_items.payload IS DISTINCT FROM EXCLUDED.payload
            "#,
        )
        .bind(&workspace)
        .bind(&id)
        .bind(s(content, "title"))
        .bind(s(content, "slug"))
        .bind(s(content, "kind"))
        .bind(s(content, "status"))
        .bind(s(content, "author"))
        .bind(s(content, "updatedAt"))
        .bind(opt_s(content, "publishedAt"))
        .bind(opt_s(content, "scheduledFor"))
        .bind(position as i32)
        .bind(content.clone())
        .execute(&mut **tx)
        .await?;

        if let Some(list) = content.get("revisions").and_then(Value::as_array) {
            for revision in list {
                revisions.push((workspace.clone(), id.clone(), revision.clone()));
            }
        }
    }

    prune_by_id(tx, "content_items", &keep).await?;

    sqlx::query("DELETE FROM content_revisions")
        .execute(&mut **tx)
        .await?;
    for (workspace, content_id, revision) in revisions {
        sqlx::query(
            r#"
            INSERT INTO content_revisions
                (workspace_id, content_id, revision, saved_at, author, payload)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (workspace_id, content_id, revision) DO UPDATE
            SET saved_at = EXCLUDED.saved_at,
                author = EXCLUDED.author,
                payload = EXCLUDED.payload
            "#,
        )
        .bind(workspace)
        .bind(content_id)
        .bind(i(&revision, "revision") as i32)
        .bind(s(&revision, "savedAt"))
        .bind(s(&revision, "author"))
        .bind(revision)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn sync_campaigns(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
    default_workspace: &str,
) -> Result<(), sqlx::Error> {
    let map = object(snapshot, "campaign_workspace");
    let mut keep = HashSet::new();
    let mut links = Vec::new();

    for (position, campaign) in array(snapshot, "campaigns").iter().enumerate() {
        let id = s(campaign, "id");
        if id.is_empty() {
            continue;
        }
        let workspace = map_workspace(map, &id, default_workspace);
        keep.insert((workspace.clone(), id.clone()));
        sqlx::query(
            r#"
            INSERT INTO campaigns
                (workspace_id, id, name, objective, status, start_date, end_date,
                 budget, position, payload, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
            ON CONFLICT (workspace_id, id) DO UPDATE
            SET name = EXCLUDED.name,
                objective = EXCLUDED.objective,
                status = EXCLUDED.status,
                start_date = EXCLUDED.start_date,
                end_date = EXCLUDED.end_date,
                budget = EXCLUDED.budget,
                position = EXCLUDED.position,
                payload = EXCLUDED.payload,
                updated_at = NOW()
            WHERE campaigns.payload IS DISTINCT FROM EXCLUDED.payload
            "#,
        )
        .bind(&workspace)
        .bind(&id)
        .bind(s(campaign, "name"))
        .bind(s(campaign, "objective"))
        .bind(s(campaign, "status"))
        .bind(opt_s(campaign, "startDate"))
        .bind(opt_s(campaign, "endDate"))
        .bind(f(campaign, "budget"))
        .bind(position as i32)
        .bind(campaign.clone())
        .execute(&mut **tx)
        .await?;

        if let Some(list) = campaign.get("contentIds").and_then(Value::as_array) {
            for content_id in list.iter().filter_map(Value::as_str) {
                links.push((workspace.clone(), id.clone(), content_id.to_string()));
            }
        }
    }

    prune_by_id(tx, "campaigns", &keep).await?;

    sqlx::query("DELETE FROM campaign_content")
        .execute(&mut **tx)
        .await?;
    for (workspace, campaign_id, content_id) in links {
        sqlx::query(
            r#"
            INSERT INTO campaign_content (workspace_id, campaign_id, content_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(workspace)
        .bind(campaign_id)
        .bind(content_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn sync_tokens(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM platform_tokens")
        .execute(&mut **tx)
        .await?;

    let Some(tokens) = snapshot
        .get("platform_tokens_enc")
        .and_then(Value::as_object)
    else {
        return Ok(());
    };
    for (key, sealed) in tokens {
        let Some(sealed) = sealed.as_str() else {
            continue;
        };
        let (workspace, platform) = key
            .split_once(':')
            .map(|(workspace, platform)| (workspace.to_string(), platform.to_string()))
            .unwrap_or_else(|| {
                (
                    crate::store::DEFAULT_WORKSPACE_ID.to_string(),
                    key.to_string(),
                )
            });
        sqlx::query(
            r#"
            INSERT INTO platform_tokens (workspace_id, platform, sealed, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (workspace_id, platform) DO UPDATE
            SET sealed = EXCLUDED.sealed, updated_at = NOW()
            "#,
        )
        .bind(workspace)
        .bind(platform)
        .bind(sealed)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn prune_by_id(
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
    keep: &HashSet<(String, String)>,
) -> Result<(), sqlx::Error> {
    let existing: Vec<(String, String)> =
        sqlx::query_as(&format!("SELECT workspace_id, id FROM {table}"))
            .fetch_all(&mut **tx)
            .await?;
    for (workspace, id) in existing {
        if !keep.contains(&(workspace.clone(), id.clone())) {
            sqlx::query(&format!(
                "DELETE FROM {table} WHERE workspace_id = $1 AND id = $2"
            ))
            .bind(workspace)
            .bind(id)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

/// Records metadata for an object uploaded to R2. This is a projection for
/// querying/auditing; the object itself lives in the bucket.
pub async fn record_media_asset(
    pool: &PgPool,
    workspace_id: &str,
    id: &str,
    kind: &str,
    r2_key: &str,
    mime: &str,
    size_bytes: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO media_assets
            (workspace_id, id, kind, r2_key, mime, size_bytes, payload, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, '{}'::jsonb, NOW())
        ON CONFLICT (r2_key) DO UPDATE
        SET workspace_id = EXCLUDED.workspace_id,
            id = EXCLUDED.id,
            kind = EXCLUDED.kind,
            mime = EXCLUDED.mime,
            size_bytes = EXCLUDED.size_bytes,
            created_at = NOW()
        "#,
    )
    .bind(workspace_id)
    .bind(id)
    .bind(kind)
    .bind(r2_key)
    .bind(mime)
    .bind(size_bytes)
    .execute(pool)
    .await
    .map_err(|error| format!("cannot record media asset: {error}"))?;
    Ok(())
}

/// Removes the metadata row for a deleted R2 object.
pub async fn delete_media_asset(pool: &PgPool, r2_key: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM media_assets WHERE r2_key = $1")
        .bind(r2_key)
        .execute(pool)
        .await
        .map_err(|error| format!("cannot delete media asset: {error}"))?;
    Ok(())
}

/// Loads the store from normalized tables. Returns `Ok(None)` when the
/// projection has not been populated yet (or the migration is missing), so the
/// caller can fall back to the snapshot.
pub async fn load_store(pool: &PgPool) -> Result<Option<Store>, String> {
    let marker = sqlx::query_as::<_, (Value,)>(
        "SELECT value FROM store_meta WHERE key = 'normalized_version'",
    )
    .fetch_optional(pool)
    .await;

    let marker = match marker {
        Ok(Some(row)) => row.0,
        Ok(None) => return Ok(None),
        Err(error) if is_missing_table(&error) => return Ok(None),
        Err(error) => return Err(format!("cannot read normalized marker: {error}")),
    };
    if marker.as_i64().unwrap_or(0) != NORMALIZED_VERSION {
        return Ok(None);
    }

    let settings: Option<(Value,)> =
        sqlx::query_as("SELECT payload FROM app_settings WHERE id = 1")
            .fetch_optional(pool)
            .await
            .map_err(|error| format!("cannot load app settings: {error}"))?;
    let Some((settings,)) = settings else {
        return Ok(None);
    };
    let setup = settings.get("setup").cloned().unwrap_or(Value::Null);
    if setup.is_null() {
        return Ok(None);
    }
    let token_salt = settings
        .get("token_salt")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let workspaces: Vec<(Value,)> =
        sqlx::query_as("SELECT payload FROM workspaces ORDER BY position, created_at")
            .fetch_all(pool)
            .await
            .map_err(|error| format!("cannot load workspaces: {error}"))?;
    if workspaces.is_empty() {
        return Ok(None);
    }

    let accounts: Vec<(Value,)> = sqlx::query_as("SELECT payload FROM users ORDER BY email")
        .fetch_all(pool)
        .await
        .map_err(|error| format!("cannot load accounts: {error}"))?;
    let sessions: Vec<(String, Value)> =
        sqlx::query_as("SELECT token_hash, payload FROM sessions WHERE expires_at > NOW()")
            .fetch_all(pool)
            .await
            .map_err(|error| format!("cannot load sessions: {error}"))?;

    let posts = load_entities(
        pool,
        "SELECT workspace_id, payload FROM posts ORDER BY position",
        "post_workspace",
    )
    .await?;
    let ideas = load_entities(
        pool,
        "SELECT workspace_id, payload FROM ideas ORDER BY position",
        "idea_workspace",
    )
    .await?;
    let hashtags = load_entities(
        pool,
        "SELECT workspace_id, payload FROM hashtag_groups ORDER BY position",
        "hashtag_workspace",
    )
    .await?;
    let txns = load_entities(
        pool,
        "SELECT workspace_id, payload FROM finance_entries ORDER BY position",
        "txn_workspace",
    )
    .await?;
    let content = load_entities(
        pool,
        "SELECT workspace_id, payload FROM content_items ORDER BY position",
        "content_workspace",
    )
    .await?;
    let campaigns = load_entities(
        pool,
        "SELECT workspace_id, payload FROM campaigns ORDER BY position",
        "campaign_workspace",
    )
    .await?;

    let metric_rows: Vec<(String, String, String, Value)> = sqlx::query_as(
        "SELECT workspace_id, post_id, platform, payload FROM metrics ORDER BY position",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| format!("cannot load metrics: {error}"))?;
    let mut metrics = Vec::with_capacity(metric_rows.len());
    let mut metric_workspace = Map::new();
    for (workspace, post_id, platform, payload) in metric_rows {
        metric_workspace.insert(format!("{post_id}:{platform}"), json!(workspace));
        metrics.push(payload);
    }

    let tokens: Vec<(String, String, String)> =
        sqlx::query_as("SELECT workspace_id, platform, sealed FROM platform_tokens")
            .fetch_all(pool)
            .await
            .map_err(|error| format!("cannot load platform tokens: {error}"))?;
    let mut platform_tokens_enc = Map::new();
    for (workspace, platform, sealed) in tokens {
        platform_tokens_enc.insert(format!("{workspace}:{platform}"), json!(sealed));
    }

    let session_pairs: Vec<Value> = sessions
        .into_iter()
        .map(|(key, payload)| json!([key, payload]))
        .collect();

    let snapshot = json!({
        "version": 2,
        "setup": setup,
        "posts": posts.0,
        "ideas": ideas.0,
        "hashtags": hashtags.0,
        "metrics": metrics,
        "txns": txns.0,
        "workspaces": workspaces.into_iter().map(|(value,)| value).collect::<Vec<_>>(),
        "content": content.0,
        "campaigns": campaigns.0,
        "post_workspace": posts.1,
        "idea_workspace": ideas.1,
        "hashtag_workspace": hashtags.1,
        "metric_workspace": metric_workspace,
        "txn_workspace": txns.1,
        "content_workspace": content.1,
        "campaign_workspace": campaigns.1,
        "platform_tokens_enc": platform_tokens_enc,
        "token_salt": token_salt,
        "accounts": accounts.into_iter().map(|(value,)| value).collect::<Vec<_>>(),
        "sessions": session_pairs,
    });

    let bytes = serde_json::to_vec(&snapshot)
        .map_err(|error| format!("cannot encode normalized store: {error}"))?;
    let mut store = Store::from_snapshot_bytes(&bytes)?;
    store.db = Some(pool.clone());
    Ok(Some(store))
}

async fn load_entities(
    pool: &PgPool,
    query: &str,
    map_name: &str,
) -> Result<(Vec<Value>, Map<String, Value>), String> {
    let rows: Vec<(String, Value)> = sqlx::query_as(query)
        .fetch_all(pool)
        .await
        .map_err(|error| format!("cannot load {map_name}: {error}"))?;
    let mut values = Vec::with_capacity(rows.len());
    let mut map = Map::new();
    for (workspace, payload) in rows {
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if !id.is_empty() {
            map.insert(id, json!(workspace));
        }
        values.push(payload);
    }
    Ok((values, map))
}
