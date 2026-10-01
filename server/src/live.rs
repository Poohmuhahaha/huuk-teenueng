//! Live mirror of the connected platform's content.
//!
//! OAuth only proves who the user is; this module pulls the actual Page and
//! Instagram content (posts, captions, media, engagement) into the workspace so
//! the app can show real data next to the planned content.
//!
//! Rules:
//! - The store lock is never held across network I/O (same rule as OAuth).
//! - A failed refresh never wipes the previous mirror: it records
//!   [`LiveData::error`] and leaves the posts in place.
//! - Everything is best-effort: a metric the API does not report stays `0`, and
//!   per-post insight failures are ignored instead of failing the refresh.
//! - Instagram media is stored under `platform: "instagram"` even though it
//!   arrives through the same Meta login, so the UI can tag it correctly.
use std::time::Duration;

use serde_json::Value;

use crate::model::{LiveAccount, LiveData, LivePost};
use crate::oauth::OauthSettings;
use crate::AppState;

/// Posts kept in the mirror (older ones are dropped to bound the snapshot).
const KEEP_LIMIT: usize = 60;
/// Per-post insight calls are capped: they are one request each and feed the
/// reactions/clicks/watch-time columns. Newest posts win.
const INSIGHT_LIMIT: usize = 20;

fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

/// `GET {GRAPH}{path}` with the access token appended; Graph errors are mapped
/// to their message so the UI can show something actionable.
async fn graph_get(
    base: &str,
    oauth: &OauthSettings,
    path: &str,
    query: &[(&str, &str)],
    token: &str,
) -> Result<Value, String> {
    let res = oauth
        .http()
        .get(format!("{base}{path}"))
        .query(query)
        .query(&[("access_token", token)])
        .send()
        .await
        .map_err(|e| format!("graph request failed: {e}"))?;
    let status = res.status();
    let body: Value = res
        .json()
        .await
        .map_err(|e| format!("graph response was not JSON ({status}): {e}"))?;
    if let Some(err) = body.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown Meta error");
        return Err(format!("graph error: {msg}"));
    }
    Ok(body)
}

fn as_u64(value: Option<&Value>) -> u64 {
    value
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
        })
        .unwrap_or(0)
}

fn str_of<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

/// Reaction/clicks maps come back as `{ "like": 3, "love": 1 }`.
pub fn parse_count_map(value: Option<&Value>) -> std::collections::BTreeMap<String, u64> {
    let mut map = std::collections::BTreeMap::new();
    if let Some(obj) = value.and_then(Value::as_object) {
        for (key, raw) in obj {
            let count = as_u64(Some(raw));
            if count > 0 {
                map.insert(key.clone(), count);
            }
        }
    }
    map
}

/// Total and link-only clicks from `post_clicks_by_type`.
pub fn parse_clicks(value: Option<&Value>) -> (u64, u64) {
    let map = parse_count_map(value);
    let total = map.values().sum();
    let link = map
        .iter()
        .filter(|(key, _)| key.contains("link"))
        .map(|(_, v)| *v)
        .sum();
    (total, link)
}

/// Reads one metric out of a multi-metric insights response.
fn insight_of(body: &Value, metric: &str) -> Option<Value> {
    body.get("data")
        .and_then(Value::as_array)?
        .iter()
        .find(|row| row.get("name").and_then(Value::as_str) == Some(metric))
        .and_then(|row| row.get("values"))
        .and_then(Value::as_array)
        .and_then(|values| values.last())
        .and_then(|v| v.get("value"))
        .cloned()
}

/// Account stats from a Page object (also carries the linked IG account).
pub fn parse_page(page: &Value) -> LiveAccount {
    LiveAccount {
        platform: "meta".into(),
        id: str_of(page, "id").to_string(),
        username: {
            let name = str_of(page, "name");
            if name.is_empty() {
                str_of(page, "link").to_string()
            } else {
                name.to_string()
            }
        },
        followers: {
            let followers = as_u64(page.get("followers_count"));
            if followers > 0 {
                followers
            } else {
                as_u64(page.get("fan_count"))
            }
        },
        posts: 0,
        engagements: 0,
        net_follows: 0,
        page_views: 0,
        video_views: 0,
    }
}

/// Linked Instagram account stats, when the Page has one.
pub fn parse_ig_account(page: &Value) -> Option<LiveAccount> {
    let ig = page.get("instagram_business_account")?;
    let id = str_of(ig, "id");
    if id.is_empty() {
        return None;
    }
    Some(LiveAccount {
        platform: "instagram".into(),
        id: id.to_string(),
        username: str_of(ig, "username").to_string(),
        followers: as_u64(ig.get("followers_count")),
        posts: as_u64(ig.get("media_count")),
        engagements: 0,
        net_follows: 0,
        page_views: 0,
        video_views: 0,
    })
}

/// One Facebook Page post from `/posts` (engagement from summary fields).
pub fn parse_fb_post(post: &Value, fetched_at: i64) -> Option<LivePost> {
    let id = str_of(post, "id");
    if id.is_empty() {
        return None;
    }
    let media_type = post
        .get("attachments")
        .and_then(|a| a.get("data"))
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|a| a.get("media_type"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let kind = match media_type {
        "video" | "video_inline" => "video",
        "photo" => "photo",
        _ => "post",
    };
    let full_picture = str_of(post, "full_picture");
    let attachment = post
        .get("attachments")
        .and_then(|a| a.get("data"))
        .and_then(Value::as_array)
        .and_then(|a| a.first());
    let attachment_title = attachment
        .map(|a| str_of(a, "title"))
        .unwrap_or("")
        .to_string();
    let link_url = attachment
        .map(|a| {
            let url = str_of(a, "unshimmed_url");
            if url.is_empty() {
                str_of(a, "url")
            } else {
                url
            }
        })
        .unwrap_or("")
        .to_string();
    Some(LivePost {
        id: id.to_string(),
        platform: "meta".into(),
        kind: kind.into(),
        caption: str_of(post, "message").to_string(),
        media_url: full_picture.to_string(),
        thumbnail_url: full_picture.to_string(),
        permalink: str_of(post, "permalink_url").to_string(),
        created_at: str_of(post, "created_time").to_string(),
        likes: as_u64(
            post.get("likes")
                .and_then(|l| l.get("summary"))
                .and_then(|s| s.get("total_count")),
        ),
        comments: as_u64(
            post.get("comments")
                .and_then(|c| c.get("summary"))
                .and_then(|s| s.get("total_count")),
        ),
        shares: as_u64(post.get("shares").and_then(|s| s.get("count"))),
        views: 0,
        reactions: Default::default(),
        reach: 0,
        saves: 0,
        clicks: 0,
        link_clicks: 0,
        video_length: 0,
        video_avg_watch_time: 0.0,
        attachment_title,
        link_url,
        fetched_at,
    })
}

/// One Instagram media item from `/{ig-id}/media`.
pub fn parse_ig_media(media: &Value, fetched_at: i64) -> Option<LivePost> {
    let id = str_of(media, "id");
    if id.is_empty() {
        return None;
    }
    let kind = match str_of(media, "media_type") {
        "VIDEO" => "reel",
        "CAROUSEL_ALBUM" => "carousel",
        _ => "photo",
    };
    let media_url = str_of(media, "media_url");
    let thumbnail = str_of(media, "thumbnail_url");
    Some(LivePost {
        id: id.to_string(),
        platform: "instagram".into(),
        kind: kind.into(),
        caption: str_of(media, "caption").to_string(),
        media_url: media_url.to_string(),
        thumbnail_url: {
            if thumbnail.is_empty() {
                media_url.to_string()
            } else {
                thumbnail.to_string()
            }
        },
        permalink: str_of(media, "permalink").to_string(),
        created_at: str_of(media, "timestamp").to_string(),
        likes: as_u64(media.get("like_count")),
        comments: as_u64(media.get("comments_count")),
        shares: 0,
        views: 0,
        reactions: Default::default(),
        reach: 0,
        saves: 0,
        clicks: 0,
        link_clicks: 0,
        video_length: 0,
        video_avg_watch_time: 0.0,
        attachment_title: String::new(),
        link_url: String::new(),
        fetched_at,
    })
}

/// Reads `{ "data": [ { name, values: [{ value }] } ] }` into a total.
pub fn parse_insight_total(body: &Value, metric: &str) -> u64 {
    body.get("data")
        .and_then(Value::as_array)
        .and_then(|rows| {
            rows.iter()
                .find(|r| r.get("name").and_then(Value::as_str) == Some(metric))
        })
        .and_then(|row| row.get("values"))
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .map(|v| as_u64(v.get("value")))
        .unwrap_or(0)
}

/// Fetches the Page's posts, the linked Instagram media, and account stats.
/// The Graph base comes from `oauth.graph_base` (default graph.facebook.com).
pub async fn fetch_meta(
    oauth: &OauthSettings,
    token: &str,
    page_id: &str,
) -> Result<LiveData, String> {
    let base = oauth.graph_base.as_str();
    let fetched_at = now_secs();
    let mut accounts = Vec::new();
    let mut posts: Vec<LivePost> = Vec::new();

    // Page + linked Instagram account in one call.
    let page = graph_get(
        base,
        oauth,
        &format!("/{page_id}"),
        &[(
            "fields",
            "id,name,fan_count,followers_count,link,instagram_business_account{id,username,followers_count,media_count}",
        )],
        token,
    )
    .await?;
    let page_account = parse_page(&page);
    if !page_account.id.is_empty() {
        accounts.push(page_account);
    }
    let ig_account = parse_ig_account(&page);
    if let Some(account) = &ig_account {
        accounts.push(account.clone());
    }

    // Facebook Page posts.
    let feed = graph_get(
        base,
        oauth,
        &format!("/{page_id}/posts"),
        &[
            (
                "fields",
                "id,message,created_time,permalink_url,full_picture,attachments{media_type,title,unshimmed_url,url},shares,likes.summary(true),comments.summary(true)",
            ),
            ("limit", "25"),
        ],
        token,
    )
    .await?;
    if let Some(rows) = feed.get("data").and_then(Value::as_array) {
        for row in rows {
            if let Some(post) = parse_fb_post(row, fetched_at) {
                posts.push(post);
            }
        }
    }

    // Instagram media, when a business account is linked.
    if let Some(account) = &ig_account {
        match graph_get(
            base,
            oauth,
            &format!("/{}/media", account.id),
            &[
                (
                    "fields",
                    "id,caption,media_type,media_url,thumbnail_url,permalink,timestamp,like_count,comments_count",
                ),
                ("limit", "25"),
            ],
            token,
        )
        .await
        {
            Ok(media) => {
                if let Some(rows) = media.get("data").and_then(Value::as_array) {
                    for row in rows {
                        if let Some(post) = parse_ig_media(row, fetched_at) {
                            posts.push(post);
                        }
                    }
                }
            }
            Err(error) => {
                // The Page mirror is still useful without Instagram.
                tracing::warn!(%error, "instagram media fetch failed");
            }
        }
    }

    posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    posts.truncate(KEEP_LIMIT);

    // Page-level insights for the account card (best-effort: the metrics Meta
    // still exposes for the new Pages experience).
    if let Some(account) = accounts.iter_mut().find(|a| a.platform == "meta") {
        if let Ok(body) = graph_get(
            base,
            oauth,
            &format!("/{page_id}/insights"),
            &[
                (
                    "metric",
                    "page_post_engagements,page_follows,page_views_total,page_video_views",
                ),
                ("period", "day"),
            ],
            token,
        )
        .await
        {
            account.engagements = insight_of(&body, "page_post_engagements")
                .map(|v| as_u64(Some(&v)))
                .unwrap_or(0);
            account.page_views = insight_of(&body, "page_views_total")
                .map(|v| as_u64(Some(&v)))
                .unwrap_or(0);
            account.video_views = insight_of(&body, "page_video_views")
                .map(|v| as_u64(Some(&v)))
                .unwrap_or(0);
            if let Some(follows) = insight_of(&body, "page_follows") {
                account.followers = account.followers.max(as_u64(Some(&follows)));
            }
        }
        if let Ok(body) = graph_get(
            base,
            oauth,
            &format!("/{page_id}/insights"),
            &[
                (
                    "metric",
                    "page_daily_follows_unique,page_daily_unfollows_unique",
                ),
                ("period", "day"),
            ],
            token,
        )
        .await
        {
            let gained = insight_of(&body, "page_daily_follows_unique")
                .map(|v| as_u64(Some(&v)))
                .unwrap_or(0) as i64;
            let lost = insight_of(&body, "page_daily_unfollows_unique")
                .map(|v| as_u64(Some(&v)))
                .unwrap_or(0) as i64;
            account.net_follows = gained - lost;
        }
    }

    // Best-effort per-post insights for the newest posts: reactions, clicks
    // and (for videos) watch time. Meta only still exposes a small metric set
    // for the new Pages experience, so an invalid metric fails the whole call
    // and we retry with the always-valid reactions metric.
    for post in posts.iter_mut().take(INSIGHT_LIMIT) {
        if post.platform == "instagram" {
            let metrics = if post.kind == "reel" || post.kind == "video" {
                "views,reach,saved"
            } else {
                "reach,saved"
            };
            let mut body = graph_get(
                base,
                oauth,
                &format!("/{}/insights", post.id),
                &[("metric", metrics)],
                token,
            )
            .await;
            if body.is_err() {
                body = graph_get(
                    base,
                    oauth,
                    &format!("/{}/insights", post.id),
                    &[("metric", "reach")],
                    token,
                )
                .await;
            }
            let Ok(body) = body else { continue };
            if let Some(views) = insight_of(&body, "views") {
                post.views = as_u64(Some(&views));
            }
            if let Some(reach) = insight_of(&body, "reach") {
                post.reach = as_u64(Some(&reach));
            }
            if let Some(saved) = insight_of(&body, "saved") {
                post.saves = as_u64(Some(&saved));
            }
            continue;
        }
        let metrics = if post.kind == "video" || post.kind == "reel" {
            "post_reactions_by_type_total,post_clicks_by_type,post_video_views,post_video_avg_time_watched,post_video_length"
        } else {
            "post_reactions_by_type_total,post_clicks_by_type"
        };
        let mut body = graph_get(
            base,
            oauth,
            &format!("/{}/insights", post.id),
            &[("metric", metrics)],
            token,
        )
        .await;
        if body.is_err() {
            body = graph_get(
                base,
                oauth,
                &format!("/{}/insights", post.id),
                &[("metric", "post_reactions_by_type_total")],
                token,
            )
            .await;
        }
        let Ok(body) = body else { continue };
        post.reactions =
            parse_count_map(insight_of(&body, "post_reactions_by_type_total").as_ref());
        let (clicks, link_clicks) = parse_clicks(insight_of(&body, "post_clicks_by_type").as_ref());
        post.clicks = clicks;
        post.link_clicks = link_clicks;
        if let Some(views) = insight_of(&body, "post_video_views") {
            post.views = as_u64(Some(&views));
        }
        if let Some(avg) = insight_of(&body, "post_video_avg_time_watched") {
            post.video_avg_watch_time = avg.as_f64().unwrap_or(0.0);
        }
        if let Some(length) = insight_of(&body, "post_video_length") {
            post.video_length = as_u64(Some(&length));
        }
    }
    Ok(LiveData {
        fetched_at: Some(fetched_at),
        accounts,
        posts,
        error: String::new(),
    })
}

/// Meta's answer when a *user* token is used for Page content (the new Pages
/// experience requires a Page access token). Connections made before the
/// Page/user token split stored only the user token, so this is also the
/// signal to upgrade them in place instead of asking for a reconnect.
fn needs_page_token(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("page access token is required")
        || lower.contains("invalid oauth 2.0 access token")
}

/// Upgrades a legacy single-token connection in place: trades the stored user
/// token for a long-lived one, fetches the Page token, and stores both in the
/// new layout (`meta` = Page token, `meta#user` = user token). Returns the
/// Page token to retry with, or the reason it could not be repaired.
async fn upgrade_legacy_token(
    state: &AppState,
    workspace_id: &str,
    oauth: &OauthSettings,
    legacy_token: &str,
    page_id: &str,
) -> Result<String, String> {
    let Some(provider) = crate::oauth::provider("meta") else {
        return Err("meta provider missing".into());
    };
    let (user_token, expires) = crate::oauth::exchange_long_lived(provider, oauth, legacy_token)
        .await
        .map_err(|e| format!("token upgrade failed: {e}"))?;
    let page_token = crate::oauth::fetch_page_token(oauth, &user_token, page_id).await?;
    let mut store = state.write().await;
    store.set_platform_token(workspace_id, "meta", &page_token);
    store.set_platform_token(workspace_id, "meta#user", &user_token);
    if let Some(connection) = store
        .workspace_mut(workspace_id)
        .and_then(|w| w.connections.iter_mut().find(|c| c.id == "meta"))
    {
        if let Some(secs) = expires {
            connection.expires_at = Some(
                (chrono::Utc::now() + chrono::Duration::seconds(secs))
                    .format("%Y-%m-%d")
                    .to_string(),
            );
        }
    }
    tracing::info!(workspace = %workspace_id, page = %page_id, "legacy meta token upgraded to page + long-lived user token");
    Ok(page_token)
}

/// Refreshes one workspace's Meta mirror. Never holds the store lock across
/// network I/O: reads what it needs, fetches, then writes the result back.
pub async fn refresh_workspace(state: &AppState, workspace_id: &str) -> Result<LiveData, String> {
    let (oauth, token, page_id) = {
        let mut store = state.write().await;
        let Some(workspace) = store.workspace(workspace_id) else {
            return Err("workspace not found".into());
        };
        let Some(connection) = workspace
            .connections
            .iter()
            .find(|c| c.id == "meta" && c.status == "connected")
        else {
            return Err("connect Meta first".into());
        };
        let page_id = connection.external_id.clone();
        let token = store.platform_token(workspace_id, "meta");
        (store.oauth.clone(), token, page_id)
    };
    let Some(token) = token else {
        return Err("reconnect Meta to grant access — the stored token is missing".into());
    };
    if page_id.is_empty() {
        return Err("reconnect Meta and pick a Page".into());
    }

    let mut token = token;
    let first = fetch_meta(&oauth, &token, &page_id).await;
    // A legacy user token cannot read Page content: repair the connection and
    // retry once before reporting failure.
    let fetched = match first {
        Err(error) if needs_page_token(&error) => {
            let has_split = {
                let mut store = state.write().await;
                store.platform_token(workspace_id, "meta#user").is_some()
            };
            if has_split {
                Err(error)
            } else {
                match upgrade_legacy_token(state, workspace_id, &oauth, &token, &page_id).await {
                    Ok(page_token) => {
                        token = page_token;
                        fetch_meta(&oauth, &token, &page_id).await
                    }
                    Err(upgrade_error) => {
                        tracing::warn!(workspace = %workspace_id, %upgrade_error, "legacy token upgrade failed");
                        Err(format!(
                            "reconnect Meta to grant access — the stored token cannot read this Page ({error})"
                        ))
                    }
                }
            }
        }
        other => other,
    };

    match fetched {
        Ok(live) => {
            let mut store = state.write().await;
            if let Some(workspace) = store.workspace_mut(workspace_id) {
                let count = live.posts.len() as u32;
                workspace.live = live.clone();
                if let Some(connection) = workspace.connections.iter_mut().find(|c| c.id == "meta")
                {
                    connection.last_sync = Some(crate::store::stamp());
                    connection.media_count = count;
                }
            }
            tracing::info!(
                workspace = %workspace_id,
                posts = live.posts.len(),
                accounts = live.accounts.len(),
                "live mirror refreshed"
            );
            Ok(live)
        }
        Err(error) => {
            tracing::warn!(workspace = %workspace_id, %error, "live refresh failed");
            let mut store = state.write().await;
            if let Some(workspace) = store.workspace_mut(workspace_id) {
                workspace.live.error = error.clone();
            }
            Err(error)
        }
    }
}

/// Background refresh: every `LIVE_REFRESH_MIN` minutes (default 30, 0
/// disables), refresh each workspace that has a connected Meta login with a
/// stored token. Failures are logged and retried on the next tick.
pub fn spawn_refresh_loop(state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let minutes = std::env::var("LIVE_REFRESH_MIN")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(30);
        if minutes == 0 {
            tracing::info!("live refresh loop disabled (LIVE_REFRESH_MIN=0)");
            return;
        }
        // Let the server finish booting before the first sweep.
        tokio::time::sleep(Duration::from_secs(20)).await;
        let mut ticker = tokio::time::interval(Duration::from_secs(minutes * 60));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            let workspace_ids: Vec<String> = {
                let mut store = state.write().await;
                let ids: Vec<String> = store
                    .workspaces
                    .iter()
                    .filter(|w| {
                        w.connections
                            .iter()
                            .any(|c| c.id == "meta" && c.status == "connected")
                    })
                    .map(|w| w.id.clone())
                    .collect();
                // Only workspaces with a usable token: a connected slot without
                // one needs a reconnect, and warning about it every tick would
                // just be noise.
                ids.into_iter()
                    .filter(|id| store.platform_token(id, "meta").is_some())
                    .collect()
            };
            for id in workspace_ids {
                let _ = refresh_workspace(&state, &id).await;
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fb_post_parses_engagement_summaries() {
        let raw = json!({
            "id": "1207146735819378_999",
            "message": "Hello world",
            "created_time": "2026-09-01T10:00:00+0000",
            "permalink_url": "https://www.facebook.com/permalink.php?story_fbid=1&id=2",
            "full_picture": "https://scontent.example/pic.jpg",
            "attachments": { "data": [{ "media_type": "photo" }] },
            "shares": { "count": 4 },
            "likes": { "summary": { "total_count": 12 } },
            "comments": { "summary": { "total_count": 3 } }
        });
        let post = parse_fb_post(&raw, 1_700_000_000).unwrap();
        assert_eq!(post.platform, "meta");
        assert_eq!(post.kind, "photo");
        assert_eq!(post.caption, "Hello world");
        assert_eq!(post.media_url, "https://scontent.example/pic.jpg");
        assert_eq!(post.likes, 12);
        assert_eq!(post.comments, 3);
        assert_eq!(post.shares, 4);
        assert_eq!(post.views, 0);
    }

    #[test]
    fn fb_video_post_detects_kind() {
        let raw = json!({
            "id": "1_2",
            "attachments": { "data": [{ "media_type": "video_inline" }] },
            "likes": { "summary": { "total_count": "7" } }
        });
        let post = parse_fb_post(&raw, 1).unwrap();
        assert_eq!(post.kind, "video");
        assert_eq!(post.likes, 7, "string counts parse too");
    }

    #[test]
    fn posts_without_an_id_are_skipped() {
        assert!(parse_fb_post(&json!({ "message": "no id" }), 1).is_none());
        assert!(parse_ig_media(&json!({ "caption": "no id" }), 1).is_none());
    }

    #[test]
    fn ig_media_parses_and_falls_back_to_media_url() {
        let raw = json!({
            "id": "17841400000000001",
            "caption": "Reel time",
            "media_type": "VIDEO",
            "media_url": "https://scontent.example/vid.mp4",
            "permalink": "https://www.instagram.com/reel/abc/",
            "timestamp": "2026-09-02T10:00:00+0000",
            "like_count": 21,
            "comments_count": 2
        });
        let post = parse_ig_media(&raw, 1).unwrap();
        assert_eq!(post.platform, "instagram");
        assert_eq!(post.kind, "reel");
        assert_eq!(post.thumbnail_url, post.media_url);
        assert_eq!(post.likes, 21);
    }

    #[test]
    fn page_and_ig_account_stats_parse() {
        let raw = json!({
            "id": "1207146735819378",
            "name": "Meni",
            "fan_count": 100,
            "followers_count": 150,
            "instagram_business_account": {
                "id": "17841400000000001",
                "username": "meni.shop",
                "followers_count": 900,
                "media_count": 42
            }
        });
        let page = parse_page(&raw);
        assert_eq!(page.followers, 150, "followers_count wins over fan_count");
        assert_eq!(page.username, "Meni");
        let ig = parse_ig_account(&raw).unwrap();
        assert_eq!(ig.platform, "instagram");
        assert_eq!(ig.username, "meni.shop");
        assert_eq!(ig.posts, 42);
    }

    #[test]
    fn count_maps_and_clicks_parse() {
        let reactions = json!({ "like": 12, "love": "3", "haha": 0 });
        let map = parse_count_map(Some(&reactions));
        assert_eq!(map.get("like"), Some(&12));
        assert_eq!(map.get("love"), Some(&3), "string counts parse");
        assert_eq!(map.get("haha"), None, "zero counts are dropped");
        assert!(parse_count_map(None).is_empty());

        let clicks = json!({ "link clicks": 4, "other clicks": 2, "photo view": 30 });
        assert_eq!(parse_clicks(Some(&clicks)), (36, 4));
        assert_eq!(parse_clicks(None), (0, 0));
    }

    #[test]
    fn insight_totals_read_the_metric_row() {
        let body = json!({
            "data": [
                { "name": "post_impressions", "values": [{ "value": 999 }] },
                { "name": "post_video_views", "values": [{ "value": 321 }] }
            ]
        });
        assert_eq!(parse_insight_total(&body, "post_video_views"), 321);
        assert_eq!(parse_insight_total(&body, "missing"), 0);
    }

    /// A stand-in for graph.facebook.com: canned Page, posts, IG media, and
    /// insight responses, so the whole fetch pipeline runs without network.
    async fn fake_graph() -> String {
        use axum::{routing::get, Json, Router};
        let app = Router::new()
            .route(
                "/1207146735819378",
                get(|| async {
                    Json(json!({
                        "id": "1207146735819378",
                        "name": "Meni",
                        "followers_count": 150,
                        "instagram_business_account": {
                            "id": "1784",
                            "username": "meni.shop",
                            "followers_count": 900,
                            "media_count": 42
                        }
                    }))
                }),
            )
            .route(
                "/1207146735819378/posts",
                get(|| async {
                    Json(json!({ "data": [
                        {
                            "id": "p1", "message": "hello",
                            "created_time": "2026-09-02T10:00:00+0000",
                            "permalink_url": "https://fb.example/p1",
                            "full_picture": "https://img.example/p1.jpg",
                            "attachments": { "data": [{ "media_type": "photo" }] },
                            "shares": { "count": 2 },
                            "likes": { "summary": { "total_count": 10 } },
                            "comments": { "summary": { "total_count": 1 } }
                        },
                        {
                            "id": "p2", "message": "video",
                            "created_time": "2026-09-01T10:00:00+0000",
                            "attachments": { "data": [{ "media_type": "video" }] },
                            "likes": { "summary": { "total_count": 3 } },
                            "comments": { "summary": { "total_count": 0 } }
                        }
                    ]}))
                }),
            )
            .route(
                "/1784/media",
                get(|| async {
                    Json(json!({ "data": [{
                        "id": "m1", "caption": "reel", "media_type": "VIDEO",
                        "media_url": "https://img.example/m1.mp4",
                        "permalink": "https://ig.example/m1",
                        "timestamp": "2026-09-03T10:00:00+0000",
                        "like_count": 5, "comments_count": 1
                    }]}))
                }),
            )
            .route(
                "/p2/insights",
                get(|| async {
                    Json(json!({ "data": [
                        { "name": "post_reactions_by_type_total", "values": [{ "value": { "like": 3, "love": 1 } }] },
                        { "name": "post_clicks_by_type", "values": [{ "value": { "link clicks": 4, "other clicks": 2 } }] },
                        { "name": "post_video_views", "values": [{ "value": 777 }] },
                        { "name": "post_video_avg_time_watched", "values": [{ "value": 12.5 }] },
                        { "name": "post_video_length", "values": [{ "value": 60 }] }
                    ]}))
                }),
            )
            .route(
                "/m1/insights",
                get(|| async {
                    Json(json!({ "data": [
                        { "name": "views", "values": [{ "value": 1234 }] },
                        { "name": "reach", "values": [{ "value": 900 }] },
                        { "name": "saved", "values": [{ "value": 12 }] }
                    ]}))
                }),
            )
            .route(
                "/p1/insights",
                get(|| async {
                    Json(json!({ "data": [
                        { "name": "post_reactions_by_type_total", "values": [{ "value": { "like": 10 } }] },
                        { "name": "post_clicks_by_type", "values": [{ "value": { "photo view": 30, "link clicks": 5 } }] }
                    ]}))
                }),
            )
            .route(
                "/1207146735819378/insights",
                get(|| async {
                    Json(json!({ "data": [
                        { "name": "page_post_engagements", "values": [{ "value": 42 }] },
                        { "name": "page_follows", "values": [{ "value": 151 }] },
                        { "name": "page_views_total", "values": [{ "value": 900 }] },
                        { "name": "page_video_views", "values": [{ "value": 5400 }] },
                        { "name": "page_daily_follows_unique", "values": [{ "value": 5 }] },
                        { "name": "page_daily_unfollows_unique", "values": [{ "value": 2 }] }
                    ]}))
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn fetch_meta_pulls_posts_media_and_views() {
        let base = fake_graph().await;
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&base);
        let live = fetch_meta(&oauth, "token", "1207146735819378")
            .await
            .unwrap();

        assert_eq!(live.accounts.len(), 2, "Page + linked Instagram");
        assert_eq!(live.posts.len(), 3);
        // Newest first: IG reel (09-03), Page photo (09-02), Page video (09-01).
        assert_eq!(live.posts[0].id, "m1");
        assert_eq!(live.posts[0].platform, "instagram");
        assert_eq!(live.posts[0].views, 1234, "IG views from insights");
        assert_eq!(live.posts[0].reach, 900, "IG reach");
        assert_eq!(live.posts[0].saves, 12, "IG saves");
        assert_eq!(live.posts[1].id, "p1");
        assert_eq!(live.posts[1].kind, "photo");
        assert_eq!(live.posts[1].likes, 10);
        assert_eq!(live.posts[1].shares, 2);
        assert_eq!(live.posts[1].reactions.get("like"), Some(&10));
        assert_eq!(live.posts[1].clicks, 35, "photo views + link clicks");
        assert_eq!(live.posts[1].link_clicks, 5);
        assert_eq!(live.posts[2].id, "p2");
        assert_eq!(live.posts[2].kind, "video");
        assert_eq!(live.posts[2].views, 777, "FB video views from insights");
        assert_eq!(live.posts[2].video_length, 60);
        assert_eq!(live.posts[2].video_avg_watch_time, 12.5);
        assert_eq!(live.posts[2].reactions.get("love"), Some(&1));

        // Page-level insights land on the account card.
        let page = &live.accounts[0];
        assert_eq!(page.engagements, 42);
        assert_eq!(page.page_views, 900);
        assert_eq!(page.video_views, 5400);
        assert_eq!(page.net_follows, 3, "5 new follows - 2 unfollows");
        assert_eq!(page.followers, 151, "page_follows wins when higher");
        assert!(live.error.is_empty());
        assert!(live.fetched_at.is_some());
    }

    /// Fake Graph that rejects the legacy user token on `/posts` (the new
    /// Pages experience), serves the long-lived exchange, and hands out the
    /// Page token — i.e. exactly what a real repair needs.
    async fn fake_upgrade_graph(allow_exchange: bool) -> String {
        use axum::{extract::Query, routing::get, Json, Router};
        use std::collections::HashMap;
        let app = Router::new()
            .route(
                "/oauth/access_token",
                get(move |Query(q): Query<HashMap<String, String>>| async move {
                    if !allow_exchange {
                        return Json(json!({ "error": { "message": "exchange refused" } }));
                    }
                    if q.get("grant_type").map(String::as_str) == Some("fb_exchange_token") {
                        Json(json!({ "access_token": "long-user-token", "expires_in": 5183944 }))
                    } else {
                        Json(json!({ "error": { "message": "unknown grant" } }))
                    }
                }),
            )
            .route(
                "/102400000000001",
                get(|Query(q): Query<HashMap<String, String>>| async move {
                    if q.get("fields").map(String::as_str) == Some("access_token") {
                        Json(json!({ "access_token": "page-token" }))
                    } else {
                        Json(json!({ "id": "102400000000001", "name": "Meni", "followers_count": 150 }))
                    }
                }),
            )
            .route(
                "/102400000000001/posts",
                get(|Query(q): Query<HashMap<String, String>>| async move {
                    if q.get("access_token").map(String::as_str) != Some("page-token") {
                        return Json(json!({ "error": {
                            "message": "Invalid OAuth 2.0 Access Token",
                            "error_user_msg": "A Page access token is required for this call for the new Pages experience.",
                            "code": 190
                        }}));
                    }
                    Json(json!({ "data": [{
                        "id": "p1", "message": "hello",
                        "created_time": "2026-09-02T10:00:00+0000",
                        "likes": { "summary": { "total_count": 5 } }
                    }]}))
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }

    async fn state_with_legacy_token(base: &str) -> crate::AppState {
        let state = crate::new_state();
        {
            let mut store = state.write().await;
            store.oauth.set_graph_base(base);
            store
                .oauth
                .set_credentials("meta", "app-id-123", "app-secret-xyz");
            store.enable_token_vault("unit-test-admin-secret-0123456789");
            store.set_platform_token(
                crate::store::DEFAULT_WORKSPACE_ID,
                "meta",
                "legacy-user-token",
            );
        }
        state
    }

    #[tokio::test]
    async fn legacy_user_token_is_upgraded_to_page_and_long_lived_tokens() {
        let base = fake_upgrade_graph(true).await;
        let state = state_with_legacy_token(&base).await;
        let live = refresh_workspace(&state, crate::store::DEFAULT_WORKSPACE_ID)
            .await
            .unwrap();
        assert_eq!(live.posts.len(), 1, "retry with the Page token succeeds");
        assert_eq!(live.posts[0].likes, 5);

        let mut store = state.write().await;
        assert_eq!(
            store
                .platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta")
                .as_deref(),
            Some("page-token")
        );
        assert_eq!(
            store
                .platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta#user")
                .as_deref(),
            Some("long-user-token"),
            "the ads mirror keeps the long-lived user token"
        );
    }

    #[tokio::test]
    async fn failed_upgrade_asks_for_a_reconnect() {
        let base = fake_upgrade_graph(false).await;
        let state = state_with_legacy_token(&base).await;
        let error = refresh_workspace(&state, crate::store::DEFAULT_WORKSPACE_ID)
            .await
            .unwrap_err();
        assert!(error.contains("reconnect"), "{error}");
    }

    #[tokio::test]
    async fn fetch_meta_maps_graph_errors() {
        use axum::{routing::get, Json, Router};
        let app = Router::new().route(
            "/999",
            get(|| async {
                Json(json!({ "error": { "message": "Invalid OAuth access token", "code": 190 } }))
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&format!("http://{addr}"));
        let error = fetch_meta(&oauth, "bad", "999").await.unwrap_err();
        assert!(error.contains("Invalid OAuth access token"), "{error}");
    }
}
