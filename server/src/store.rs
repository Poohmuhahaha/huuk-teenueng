//! In-memory state + ids/timestamps, with optional durable JSON snapshots.
//!
//! The store is the single source of truth for every handler (`AppState` wraps
//! it in an `RwLock`). `Store::save_to` writes an atomic snapshot and
//! `Store::load_or_seed` restores one, so a deployment can survive restarts
//! without a database; swapping in a repository later keeps the same surface.
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::model::*;

/// How long a session token stays valid.
pub const SESSION_TTL_DAYS: i64 = 7;

/// Maximum live sessions kept per account (oldest are evicted first).
pub const MAX_SESSIONS_PER_ACCOUNT: usize = 20;

/// Failed logins allowed per account inside [`LOGIN_WINDOW_MIN`] before lockout.
pub const LOGIN_MAX_FAILURES: u32 = 10;

/// Sliding window for the failed-login lockout.
pub const LOGIN_WINDOW_MIN: i64 = 15;

/// Password length limits (Argon2 hashing cost grows with input size).
pub const PASSWORD_MIN_CHARS: usize = 12;
pub const PASSWORD_MAX_CHARS: usize = 256;

/// Snapshot format version — bump when the persisted shape changes.
/// v2 moved brand + connections into per-workspace containers.
const SNAPSHOT_VERSION: u32 = 2;

/// Id of the workspace created for snapshots that predate workspaces.
pub const DEFAULT_WORKSPACE_ID: &str = "ws-default";

/// Internal sign-in credentials. Never serialized into an API response — the
/// wire-facing user is always `model::User { name, role }`. The snapshot file
/// does contain password hashes; keep it owner-readable only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub email: String,
    pub password_hash: String,
    /// SaaS package for this account ("free", "pro", "business").
    #[serde(default = "default_plan")]
    pub plan: String,
    /// Last successful sign-in ("YYYY-MM-DD HH:MM"). Used to hand pre-ownership
    /// workspaces to the account that actually used the deployment.
    #[serde(default)]
    pub last_active: Option<String>,
}

fn default_plan() -> String {
    "free".to_string()
}

/// An active login session. The account is identified by email — display names
/// are only labels, so two accounts can never share one identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub email: String,
    pub expires: DateTime<Utc>,
}

/// Failed-login bookkeeping for one account.
#[derive(Debug, Clone)]
pub struct LoginAttempt {
    pub failures: u32,
    pub window_start: DateTime<Utc>,
}

#[derive(Debug)]
pub struct Store {
    pub setup: SetupConfig,
    pub posts: Vec<Post>,
    pub ideas: Vec<Idea>,
    pub hashtags: Vec<HashtagGroup>,
    pub metrics: Vec<Metric>,
    pub txns: Vec<Txn>,
    /// Workspaces own the brand identity and the social connections. At least
    /// one always exists; the first one is the fallback when a request does
    /// not send `X-Workspace-Id`.
    pub workspaces: Vec<Workspace>,
    /// CMS content items (articles/pages/notes) with revision history.
    pub content: Vec<Content>,
    /// Marketing campaigns (brief, schedule window, targets, linked content).
    pub campaigns: Vec<Campaign>,
    /// Lowercased email → account (internal only, never serialized).
    pub accounts: HashMap<String, Account>,
    /// Session token → session (random 64-char hex tokens, 7-day TTL).
    pub sessions: HashMap<String, Session>,
    /// OAuth client credentials + URLs (from env; empty ⇒ mock mode).
    pub oauth: crate::oauth::OauthSettings,
    /// Pending OAuth `state` values (CSRF protection).
    pub oauth_states: HashMap<String, crate::oauth::OAuthState>,
    /// Pending OAuth page picks (memory-only, never persisted): connect flows
    /// that returned several Pages and wait for the user to choose one.
    pub oauth_picks: HashMap<String, crate::oauth::PendingPick>,
    /// Provider access tokens for connected platforms (internal only; tokens
    /// are never sent to the browser). This map is the in-memory working copy;
    /// `platform_tokens_enc` holds the sealed form that is persisted.
    pub platform_tokens: HashMap<String, String>,
    /// Sealed provider tokens (`{workspace}:{platform}` → ChaCha20-Poly1305
    /// blob), persisted in the snapshot. See `crate::secrets`.
    pub platform_tokens_enc: HashMap<String, String>,
    /// Salt for the token-sealing key (hex, persisted). Regenerated only when
    /// a snapshot does not carry one.
    pub token_salt: String,
    /// Derived token-sealing key. `None` when `ADMIN_TOKEN` is unset/short
    /// (demo mode) — tokens then stay in memory, as before.
    pub token_sealing_key: Option<[u8; 32]>,
    /// Failed-login counters (internal, in-memory only).
    pub login_attempts: HashMap<String, LoginAttempt>,
    /// True when the server runs with the wireframe demo accounts/seed.
    pub demo_mode: bool,
    /// Whether `POST /api/auth/register` is open.
    pub allow_registration: bool,
    /// Optional operator recovery token (`X-Admin-Token` header) for lockout recovery.
    pub admin_token: Option<String>,
    /// Directory for uploaded brand fonts (set at boot; `None` disables them).
    pub fonts_dir: Option<PathBuf>,
    /// Uploaded brand images (logos, moodboard), beside the snapshot.
    pub images_dir: Option<PathBuf>,
}

/// Hashes a password with Argon2id (default params); returns a PHC string.
pub fn hash_password(pw: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .expect("argon2 hashing cannot fail with valid params")
        .to_string()
}

pub fn verify_password(pw: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(pw.as_bytes(), &parsed)
        .is_ok()
}

/// A constant Argon2 hash used to equalize login timing for unknown emails, so
/// the endpoint does not reveal whether an account exists.
pub fn dummy_password_hash() -> &'static str {
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY.get_or_init(|| hash_password("timing-equalization-placeholder"))
}

/// `bytes` random bytes rendered as lowercase hex.
pub fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

/// A 32-byte random token rendered as 64 lowercase hex characters.
pub fn session_token() -> String {
    random_hex(32)
}

/// Storage key for a session: the token is hashed so the snapshot can keep
/// sessions across restarts without ever holding a usable bearer token.
pub fn session_key(token: &str) -> String {
    use blake2::digest::{Update, VariableOutput};
    let mut hasher = blake2::Blake2bVar::new(32).expect("32-byte blake2b");
    hasher.update(token.as_bytes());
    let mut out = [0u8; 32];
    hasher.finalize_variable(&mut out).expect("32-byte output");
    out.iter().map(|b| format!("{b:02x}")).collect()
}

/// Collision-free identifiers: prefix + 8 random hex chars.
pub fn uid(prefix: &str) -> String {
    format!("{prefix}-{}", random_hex(4))
}

pub fn stamp() -> String {
    Utc::now().format("%Y-%m-%d %H:%M").to_string()
}

pub fn plus_days(days: i64) -> String {
    (Utc::now() + Duration::days(days))
        .format("%Y-%m-%d")
        .to_string()
}

#[allow(clippy::too_many_arguments)]
fn post(
    id: &str,
    month: u32,
    topic: &str,
    pillar: &str,
    format: &str,
    date: &str,
    time: &str,
    status: &str,
    hook: &str,
    caption: &str,
    cta: &str,
    group: &str,
    tags: &[&str],
    done: bool,
    platforms: &[&str],
) -> Post {
    Post {
        id: id.into(),
        month,
        topic: topic.into(),
        pillar: pillar.into(),
        format: format.into(),
        goal: "10K subscribe".into(),
        date: Some(date.into()),
        time: time.into(),
        status: status.into(),
        hook: hook.into(),
        caption: caption.into(),
        cta: cta.into(),
        hashtag_group: group.into(),
        hashtags: tags.iter().map(|s| s.to_string()).collect(),
        image_url: String::new(),
        note: String::new(),
        done,
        platforms: platforms.iter().map(|s| s.to_string()).collect(),
        locked_by: None,
    }
}

/// Demo live mirror: the Meta seed connection with a few mirrored posts so the
/// wireframe shows the "Live" sections without a real refresh.
fn seed_live() -> crate::model::LiveData {
    let fetched = Utc::now().timestamp();
    let post = |id: &str,
                platform: &str,
                kind: &str,
                caption: &str,
                date: &str,
                likes: u64,
                comments: u64,
                shares: u64,
                views: u64| {
        crate::model::LivePost {
            id: id.into(),
            platform: platform.into(),
            kind: kind.into(),
            caption: caption.into(),
            media_url: String::new(),
            thumbnail_url: String::new(),
            permalink: format!("https://www.facebook.com/{id}"),
            created_at: date.into(),
            likes,
            comments,
            shares,
            views,
            reactions: Default::default(),
            reach: 0,
            saves: 0,
            clicks: 0,
            link_clicks: 0,
            video_length: 0,
            video_avg_watch_time: 0.0,
            attachment_title: String::new(),
            link_url: String::new(),
            fetched_at: fetched,
        }
    };
    crate::model::LiveData {
        fetched_at: Some(fetched),
        accounts: vec![
            crate::model::LiveAccount {
                platform: "meta".into(),
                id: "102400000000001".into(),
                username: "Studio Channel".into(),
                followers: 4_820,
                posts: 214,
                engagements: 312,
                net_follows: 46,
                page_views: 1_240,
                video_views: 5_400,
            },
            crate::model::LiveAccount {
                platform: "instagram".into(),
                id: "17841400000000001".into(),
                username: "studio.channel".into(),
                followers: 3_140,
                posts: 128,
                engagements: 0,
                net_follows: 0,
                page_views: 0,
                video_views: 0,
            },
        ],
        posts: vec![
            post(
                "102400000000001_9001",
                "meta",
                "photo",
                "Behind the scenes of the spring shoot.",
                "2026-03-01T06:00:00+0000",
                214,
                18,
                6,
                0,
            ),
            post(
                "102400000000001_9002",
                "meta",
                "video",
                "60-second studio tour.",
                "2026-02-26T09:30:00+0000",
                132,
                9,
                4,
                5_400,
            ),
            post(
                "17841400000000001_7001",
                "instagram",
                "reel",
                "Slow living, honest reviews.",
                "2026-02-24T12:00:00+0000",
                96,
                7,
                0,
                3_100,
            ),
        ],
        error: String::new(),
    }
}

/// Demo ads mirror: one account, two campaigns with settings + insights, so
/// the Campaigns page has real shapes to render without a Graph connection.
fn seed_ads() -> crate::model::AdsData {
    use crate::model::{Ad, AdAccount, AdCampaign, AdInsight, AdSet, AdsData};
    let fetched = Utc::now().timestamp();
    AdsData {
        fetched_at: Some(fetched),
        accounts: vec![AdAccount {
            id: "act_102400000000001".into(),
            name: "Studio Channel Ads".into(),
            status: "active".into(),
            currency: "THB".into(),
            timezone: "Asia/Bangkok".into(),
            business: "Studio Channel".into(),
        }],
        campaigns: vec![
            AdCampaign {
                id: "cmp-ads-1".into(),
                account_id: "act_102400000000001".into(),
                name: "Spring Collection — Traffic".into(),
                objective: "OUTCOME_TRAFFIC".into(),
                status: "ACTIVE".into(),
                effective_status: "ACTIVE".into(),
                buying_type: "AUCTION".into(),
                daily_budget: 50_000,
                budget_remaining: 32_000,
                bid_strategy: "LOWEST_COST_WITHOUT_CAP".into(),
                special_ad_categories: vec!["NONE".into()],
                start_time: "2026-02-20T02:00:00+0000".into(),
                created_time: "2026-02-18T04:00:00+0000".into(),
                ..Default::default()
            },
            AdCampaign {
                id: "cmp-ads-2".into(),
                account_id: "act_102400000000001".into(),
                name: "Reels Engagement — Paused".into(),
                objective: "OUTCOME_ENGAGEMENT".into(),
                status: "PAUSED".into(),
                effective_status: "PAUSED".into(),
                buying_type: "AUCTION".into(),
                lifetime_budget: 300_000,
                budget_remaining: 120_000,
                bid_strategy: "LOWEST_COST_WITH_BID_CAP".into(),
                special_ad_categories: vec!["NONE".into()],
                start_time: "2026-02-10T02:00:00+0000".into(),
                created_time: "2026-02-08T04:00:00+0000".into(),
                ..Default::default()
            },
        ],
        adsets: vec![
            AdSet {
                id: "adset-1".into(),
                campaign_id: "cmp-ads-1".into(),
                name: "Bangkok 25-45".into(),
                status: "ACTIVE".into(),
                effective_status: "ACTIVE".into(),
                optimization_goal: "LINK_CLICKS".into(),
                billing_event: "IMPRESSIONS".into(),
                start_time: "2026-02-20T02:00:00+0000".into(),
                targeting: "25-45 · TH · 3 interests".into(),
                promoted_object: "page 102400000000001".into(),
                ..Default::default()
            },
            AdSet {
                id: "adset-2".into(),
                campaign_id: "cmp-ads-2".into(),
                name: "Reels viewers".into(),
                status: "PAUSED".into(),
                effective_status: "PAUSED".into(),
                optimization_goal: "POST_ENGAGEMENT".into(),
                billing_event: "IMPRESSIONS".into(),
                targeting: "18-34 · TH".into(),
                ..Default::default()
            },
        ],
        ads: vec![
            Ad {
                id: "ad-1".into(),
                adset_id: "adset-1".into(),
                name: "Spring photo — link".into(),
                status: "ACTIVE".into(),
                effective_status: "ACTIVE".into(),
                creative_title: "Spring is here".into(),
                creative_body: "Shop the new collection.".into(),
                story_id: "102400000000001_9001".into(),
                ..Default::default()
            },
            Ad {
                id: "ad-2".into(),
                adset_id: "adset-1".into(),
                name: "Studio tour — video".into(),
                status: "ACTIVE".into(),
                effective_status: "ACTIVE".into(),
                creative_title: "60-second studio tour".into(),
                story_id: "102400000000001_9002".into(),
                ..Default::default()
            },
        ],
        insights: vec![
            AdInsight {
                campaign_id: "cmp-ads-1".into(),
                spend: 1_860.0,
                impressions: 42_000,
                reach: 31_500,
                frequency: 1.33,
                clicks: 1_240,
                ctr: 2.95,
                cpc: 1.5,
                cpm: 44.29,
                results: 1_240,
                result_label: "Link clicks".into(),
                roas: 0.0,
            },
            AdInsight {
                campaign_id: "cmp-ads-2".into(),
                spend: 640.0,
                impressions: 18_200,
                reach: 15_100,
                frequency: 1.21,
                clicks: 410,
                ctr: 2.25,
                cpc: 1.56,
                cpm: 35.16,
                results: 3_400,
                result_label: "Engagements".into(),
                roas: 0.0,
            },
        ],
        audit: vec![],
        error: String::new(),
    }
}

/// Demo CMS content: one published article (with a revision), one draft, and
/// one scheduled piece — enough to exercise the whole studio workflow.
fn seed_campaigns() -> Vec<Campaign> {
    vec![
        Campaign {
            id: "cmp-spring".into(),
            name: "Spring Launch".into(),
            objective: "Launch the spring collection with a 4-week content sprint.".into(),
            status: "active".into(),
            start_date: Some("2026-03-01".into()),
            end_date: Some("2026-03-31".into()),
            platforms: vec!["Instagram".into(), "TikTok".into()],
            pillars: vec!["Pillar I".into(), "Pillar II".into()],
            hashtags: vec!["spring".into(), "launch".into()],
            budget: 15_000.0,
            goal_metric: "views".into(),
            goal_target: 250_000.0,
            owner: "Studio Owner".into(),
            notes: "Two posts per week plus one story series.".into(),
            content_ids: vec!["c-march".into()],
            created_at: "2026-02-20 09:00".into(),
            updated_at: "2026-02-20 09:00".into(),
        },
        Campaign {
            id: "cmp-evergreen".into(),
            name: "Evergreen Growth".into(),
            objective: "Keep the back catalogue working with monthly refreshes.".into(),
            status: "draft".into(),
            start_date: Some("2026-04-01".into()),
            end_date: Some("2026-06-30".into()),
            platforms: vec!["Youtube".into()],
            pillars: vec!["Pillar III".into()],
            hashtags: vec!["evergreen".into()],
            budget: 0.0,
            goal_metric: "likes".into(),
            goal_target: 5_000.0,
            owner: "Studio Owner".into(),
            notes: String::new(),
            content_ids: vec![],
            created_at: "2026-03-01 10:00".into(),
            updated_at: "2026-03-01 10:00".into(),
        },
    ]
}

fn seed_content() -> Vec<Content> {
    let now = Utc::now();
    let at = |days: i64, hours: i64| {
        (now + Duration::days(days) + Duration::hours(hours))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    };
    let welcome = Content {
        id: "c-welcome".into(),
        title: "Why slow mornings changed our year".into(),
        slug: "why-slow-mornings-changed-our-year".into(),
        kind: "article".into(),
        status: "published".into(),
        body: "# Why slow mornings changed our year\n\nWe started filming before sunrise and \
learned more about our audience than any analytics dashboard told us.\n\n## What changed\n\n\
- We plan one meaningful post per day, not five rushed ones\n- The first hour is offline: no \
notifications, just notes\n- Every caption is written the evening before\n\n> Consistency is a \
byproduct of a calm routine, not the other way around.\n\nRead more on the [planner](#/planner/2)."
            .into(),
        excerpt: "Our editorial experiment: one calm hour before the city wakes, and the content it produced."
            .into(),
        hero_image_url: String::new(),
        tags: vec!["routine".into(), "behind-the-scenes".into()],
        seo_title: "Slow mornings, better content".into(),
        seo_description: "A calm morning routine changed how we plan and publish content.".into(),
        author: "Studio Owner".into(),
        created_at: at(-12, 0),
        updated_at: at(-1, 0),
        published_at: Some(at(-1, 0)),
        scheduled_for: None,
        version: 3,
        revisions: vec![ContentRevision {
            revision: 1,
            saved_at: at(-2, 0),
            author: "Studio Owner".into(),
            note: "First draft".into(),
            title: "Slow mornings".into(),
            body: "Draft notes about our slow morning routine.".into(),
            excerpt: "Draft notes.".into(),
            seo_title: String::new(),
            seo_description: String::new(),
            tags: vec!["routine".into()],
            status: "draft".into(),
        }],
    };
    let draft = Content {
        id: "c-bts".into(),
        title: "Behind the scenes: the February shoot".into(),
        slug: "behind-the-scenes-february-shoot".into(),
        kind: "article".into(),
        status: "draft".into(),
        body: "## Shot list\n\n1. Kitchen counter reset\n2. Desk before/after\n3. Golden hour close-ups\n\n_Add captions before publishing._".into(),
        excerpt: "Set photos, mistakes and the gear that survived the day.".into(),
        hero_image_url: String::new(),
        tags: vec!["behind-the-scenes".into()],
        seo_title: String::new(),
        seo_description: String::new(),
        author: "Editor Earn".into(),
        created_at: at(-1, 0),
        updated_at: at(0, -6),
        published_at: None,
        scheduled_for: None,
        version: 1,
        revisions: vec![],
    };
    let scheduled = Content {
        id: "c-march".into(),
        title: "March launch announcement".into(),
        slug: "march-launch-announcement".into(),
        kind: "page".into(),
        status: "scheduled".into(),
        body: "**Save the date** — the new spring collection lands March 1.\n\nJoin the newsletter for first access.".into(),
        excerpt: "The spring collection lands March 1 — here is what to expect.".into(),
        hero_image_url: String::new(),
        tags: vec!["launch".into(), "event".into()],
        seo_title: "March launch".into(),
        seo_description: "Spring collection launching March 1.".into(),
        author: "Studio Owner".into(),
        created_at: at(-1, 0),
        updated_at: at(0, -2),
        published_at: None,
        scheduled_for: Some(at(7, 0)),
        version: 1,
        revisions: vec![],
    };
    vec![welcome, draft, scheduled]
}

/// Persisted shape of the store (versioned, JSON). Ephemeral data (OAuth
/// pending states, login counters) is intentionally omitted; provider tokens
/// are persisted only in their sealed form (`platform_tokens_enc`) and
/// sessions only as token hashes, so the snapshot holds no usable secrets.
#[derive(Serialize, Deserialize)]
struct Snapshot {
    version: u32,
    setup: SetupConfig,
    posts: Vec<Post>,
    ideas: Vec<Idea>,
    hashtags: Vec<HashtagGroup>,
    metrics: Vec<Metric>,
    txns: Vec<Txn>,
    /// Workspace containers (v2+). Empty in v1 snapshots — the legacy
    /// `brand`/`connections` fields below are migrated into a default
    /// workspace on load.
    #[serde(default)]
    workspaces: Vec<Workspace>,
    /// v1 legacy brand; read on load, never written back.
    #[serde(default, skip_serializing)]
    brand: Option<Brand>,
    /// v1 legacy connections; read on load, never written back.
    #[serde(default, skip_serializing)]
    connections: Option<Vec<PlatformConnection>>,
    #[serde(default)]
    content: Vec<Content>,
    #[serde(default)]
    campaigns: Vec<Campaign>,
    /// Sealed provider tokens (`Store::platform_tokens_enc`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    platform_tokens_enc: HashMap<String, String>,
    /// Salt for the token-sealing key (hex; `Store::token_salt`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    token_salt: String,
    accounts: Vec<Account>,
    /// Hashed sessions (`store::session_key`), so a restart keeps users signed
    /// in while the snapshot never holds a usable bearer token.
    #[serde(default)]
    sessions: Vec<(String, Session)>,
}

/// Atomically replaces `path` with `bytes` (write temp + rename).
///
/// The temp file is created owner-only (0600 on Unix) and synced before the
/// rename, so a crash cannot leave a truncated or group/world-readable store
/// behind. The parent directory is synced too, so the rename itself survives
/// power loss.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        }
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("store.json");
    let tmp = path.with_file_name(format!(".{name}.{}.tmp", random_hex(8)));

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&tmp)
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    file.sync_all()
        .map_err(|e| format!("cannot flush {}: {e}", tmp.display()))?;
    drop(file);

    std::fs::rename(&tmp, path).map_err(|e| format!("cannot replace {}: {e}", path.display()))?;
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            if let Ok(dir) = File::open(dir) {
                let _ = dir.sync_all();
            }
        }
    }
    Ok(())
}

/// The provider slots every workspace starts with, all disconnected.
pub fn blank_connections() -> Vec<PlatformConnection> {
    let slot = |id: &str, note: &str| PlatformConnection {
        id: id.into(),
        status: "disconnected".into(),
        handle: String::new(),
        external_id: String::new(),
        scopes: vec![],
        token_type: "—".into(),
        expires_at: None,
        last_sync: None,
        media_count: 0,
        note: note.into(),
    };
    vec![
        slot(
            "meta",
            "Facebook Page + linked Instagram · token refresh due day 45",
        ),
        slot("youtube", "Quota 10,000 units/day"),
        slot("tiktok", "Audited app required to publish"),
    ]
}

/// A workspace with a blank brand and the standard disconnected provider slots.
pub fn blank_workspace(id: &str, name: &str) -> Workspace {
    blank_workspace_for(id, name, "")
}

/// A workspace owned by `owner` (an account email; empty = shared).
pub fn blank_workspace_for(id: &str, name: &str, owner: &str) -> Workspace {
    Workspace {
        id: id.to_string(),
        name: name.trim().to_string(),
        created: stamp(),
        owner: owner.to_string(),
        members: vec![],
        brand: Brand::empty(),
        connections: blank_connections(),
        live: crate::model::LiveData::default(),
        ads: crate::model::AdsData::default(),
        ads_manage: false,
    }
}

impl Store {
    /// Resolves which workspace a request targets: the `X-Workspace-Id`
    /// header when it names a workspace visible to `account`, otherwise the
    /// first visible one. `None` (demo mode / operator token) sees everything.
    pub fn resolve_workspace_id(
        &self,
        requested: Option<&str>,
        account: Option<&str>,
    ) -> Option<String> {
        if let Some(id) = requested {
            if self
                .workspaces
                .iter()
                .any(|w| w.id == id && w.visible_to(account))
            {
                return Some(id.to_string());
            }
        }
        self.workspaces
            .iter()
            .find(|w| w.visible_to(account))
            .map(|w| w.id.clone())
    }

    /// Workspaces visible to `account` (all of them for demo/operator).
    pub fn visible_workspaces(&self, account: Option<&str>) -> Vec<&Workspace> {
        self.workspaces
            .iter()
            .filter(|w| w.visible_to(account))
            .collect()
    }

    pub fn workspace(&self, id: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    pub fn workspace_mut(&mut self, id: &str) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|w| w.id == id)
    }

    /// Internal provider-token slot: one token per workspace and platform, so
    /// connecting the same platform in two workspaces never shares a login.
    pub fn token_key(workspace_id: &str, platform: &str) -> String {
        format!("{workspace_id}:{platform}")
    }

    /// Enables encrypted-at-rest token storage with `admin_token` as the
    /// sealing secret. No-op for short/empty secrets (demo mode): tokens then
    /// live in memory only.
    pub fn enable_token_vault(&mut self, admin_token: &str) {
        let salt = crate::secrets::from_hex(&self.token_salt)
            .unwrap_or_else(|| crate::secrets::new_salt().to_vec());
        self.token_sealing_key = crate::secrets::derive_key(admin_token, &salt);
        if self.token_sealing_key.is_none() {
            tracing::warn!(
                "provider tokens stay in memory: ADMIN_TOKEN is unset or shorter than 16 chars"
            );
        }
    }

    /// Stores a provider token in memory and (when the vault is enabled) seals
    /// it for the snapshot. Never logged.
    pub fn set_platform_token(&mut self, workspace_id: &str, platform: &str, token: &str) {
        let key = Self::token_key(workspace_id, platform);
        if let Some(sealing) = &self.token_sealing_key {
            if let Some(sealed) = crate::secrets::seal(sealing, token) {
                self.platform_tokens_enc.insert(key.clone(), sealed);
            }
        }
        self.platform_tokens.insert(key, token.to_string());
    }

    /// Reads a provider token, decrypting the sealed copy on first use after a
    /// restart and caching it in memory for the process lifetime.
    pub fn platform_token(&mut self, workspace_id: &str, platform: &str) -> Option<String> {
        let key = Self::token_key(workspace_id, platform);
        if let Some(token) = self.platform_tokens.get(&key) {
            return Some(token.clone());
        }
        let sealing = self.token_sealing_key?;
        let sealed = self.platform_tokens_enc.get(&key)?.clone();
        match crate::secrets::open(&sealing, &sealed) {
            Some(token) => {
                self.platform_tokens.insert(key, token.clone());
                Some(token)
            }
            None => {
                tracing::warn!(
                    %key,
                    "stored provider token cannot be decrypted — ADMIN_TOKEN changed? reconnect the platform"
                );
                None
            }
        }
    }

    /// Presence check for the UI (does a usable token exist for this
    /// connection?), without decrypting or caching anything.
    pub fn has_platform_token(&self, workspace_id: &str, platform: &str) -> bool {
        let key = Self::token_key(workspace_id, platform);
        self.platform_tokens.contains_key(&key) || self.platform_tokens_enc.contains_key(&key)
    }

    pub fn clear_platform_token(&mut self, workspace_id: &str, platform: &str) {
        let key = Self::token_key(workspace_id, platform);
        self.platform_tokens.remove(&key);
        self.platform_tokens_enc.remove(&key);
        // Derived keys (e.g. `meta#user`: the user token kept alongside the
        // Meta Page token) belong to the same connection and go with it.
        let derived = format!("{key}#");
        self.platform_tokens.retain(|k, _| !k.starts_with(&derived));
        self.platform_tokens_enc
            .retain(|k, _| !k.starts_with(&derived));
    }

    /// Drops every stored token of a deleted workspace (both copies).
    pub fn clear_workspace_tokens(&mut self, workspace_id: &str) {
        let prefix = format!("{workspace_id}:");
        self.platform_tokens
            .retain(|key, _| !key.starts_with(&prefix));
        self.platform_tokens_enc
            .retain(|key, _| !key.starts_with(&prefix));
    }

    /// Starts a session for the account with `email` and returns the token.
    /// Evicts the oldest sessions when the per-account cap is reached.
    pub fn open_session(&mut self, email: &str) -> String {
        if let Some(account) = self.accounts.get_mut(email) {
            account.last_active = Some(stamp());
        }
        let mut mine: Vec<(String, DateTime<Utc>)> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.email == email)
            .map(|(t, s)| (t.clone(), s.expires))
            .collect();
        if mine.len() >= MAX_SESSIONS_PER_ACCOUNT {
            mine.sort_by_key(|(_, expires)| *expires);
            let excess = mine.len() - MAX_SESSIONS_PER_ACCOUNT + 1;
            for (token, _) in mine.into_iter().take(excess) {
                self.sessions.remove(&token);
            }
        }
        let token = session_token();
        self.sessions.insert(
            session_key(&token),
            Session {
                email: email.to_string(),
                expires: Utc::now() + Duration::days(SESSION_TTL_DAYS),
            },
        );
        token
    }

    /// Looks up a live session by its bearer token (hashed before lookup).
    pub fn session(&self, token: &str) -> Option<&Session> {
        self.sessions.get(&session_key(token))
    }

    /// Revokes one session by its bearer token.
    pub fn revoke_session(&mut self, token: &str) {
        self.sessions.remove(&session_key(token));
    }

    /// Live (non-expired) sessions for an account. Expired entries are pruned.
    pub fn live_session_count(&mut self, email: &str) -> usize {
        let now = Utc::now();
        self.sessions.retain(|_, s| s.expires > now);
        self.sessions.values().filter(|s| s.email == email).count()
    }

    /// Seconds the caller must wait before retrying login for `email`, if the
    /// account is currently locked out after repeated failures.
    pub fn login_retry_after(&self, email: &str) -> Option<i64> {
        let attempt = self.login_attempts.get(email)?;
        if attempt.failures < LOGIN_MAX_FAILURES {
            return None;
        }
        let elapsed = Utc::now() - attempt.window_start;
        let window = Duration::minutes(LOGIN_WINDOW_MIN);
        if elapsed >= window {
            return None;
        }
        Some((window - elapsed).num_seconds().max(1))
    }

    /// Records a failed login, resetting the window after it expires.
    pub fn record_login_failure(&mut self, email: &str) {
        let now = Utc::now();
        let entry = self
            .login_attempts
            .entry(email.to_string())
            .or_insert(LoginAttempt {
                failures: 0,
                window_start: now,
            });
        if now - entry.window_start > Duration::minutes(LOGIN_WINDOW_MIN) {
            entry.failures = 0;
            entry.window_start = now;
        }
        entry.failures += 1;
    }

    pub fn clear_login_failures(&mut self, email: &str) {
        self.login_attempts.remove(email);
    }

    /// Drops expired sessions, pending OAuth states and page picks (called
    /// periodically).
    pub fn prune_ephemeral(&mut self) {
        let now = Utc::now();
        self.sessions.retain(|_, s| s.expires > now);
        let window = Duration::minutes(crate::oauth::STATE_TTL_MIN);
        self.oauth_states.retain(|_, st| now - st.created <= window);
        let pick_window = Duration::minutes(crate::oauth::PICK_TTL_MIN);
        self.oauth_picks
            .retain(|_, p| now - p.created <= pick_window);
        self.login_attempts
            .retain(|_, a| now - a.window_start <= Duration::minutes(LOGIN_WINDOW_MIN));
    }

    /// Serializes the durable portion of the store as pretty JSON.
    pub fn snapshot_bytes(&self) -> Result<Vec<u8>, String> {
        let snapshot = Snapshot {
            version: SNAPSHOT_VERSION,
            setup: self.setup.clone(),
            posts: self.posts.clone(),
            ideas: self.ideas.clone(),
            hashtags: self.hashtags.clone(),
            metrics: self.metrics.clone(),
            txns: self.txns.clone(),
            workspaces: self.workspaces.clone(),
            brand: None,
            connections: None,
            content: self.content.clone(),
            campaigns: self.campaigns.clone(),
            platform_tokens_enc: self.platform_tokens_enc.clone(),
            token_salt: self.token_salt.clone(),
            accounts: self.accounts.values().cloned().collect(),
            // Sessions are stored as token *hashes*, so persisting them keeps
            // users signed in across restarts without storing usable tokens.
            sessions: self
                .sessions
                .iter()
                .map(|(key, session)| (key.clone(), session.clone()))
                .collect(),
        };
        serde_json::to_vec_pretty(&snapshot).map_err(|e| format!("cannot serialize store: {e}"))
    }

    /// Loads a snapshot from `path`, or seeds a fresh store when it is absent.
    /// A corrupt snapshot is an error — the caller decides whether to stop.
    pub fn load_or_seed(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::seed());
        }
        let bytes =
            std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let snapshot: Snapshot = serde_json::from_slice(&bytes)
            .map_err(|e| format!("{} is not a valid snapshot: {e}", path.display()))?;
        // v1 predates workspaces and is migrated below; v2 is current.
        if snapshot.version != 1 && snapshot.version != SNAPSHOT_VERSION {
            return Err(format!(
                "snapshot version {} is not supported (expected 1 or {SNAPSHOT_VERSION})",
                snapshot.version
            ));
        }
        let mut store = Self::seed();
        store.setup = snapshot.setup;
        store.posts = snapshot.posts;
        store.ideas = snapshot.ideas;
        store.hashtags = snapshot.hashtags;
        store.metrics = snapshot.metrics;
        store.txns = snapshot.txns;
        store.workspaces = snapshot.workspaces;
        if store.workspaces.is_empty() {
            // v1 snapshot (or a v2 one that lost its workspaces): rebuild the
            // default workspace from the legacy fields / seed defaults.
            let mut workspace = blank_workspace(DEFAULT_WORKSPACE_ID, &store.setup.workspace_name);
            if let Some(brand) = snapshot.brand {
                workspace.brand = brand;
            }
            if let Some(connections) = snapshot.connections {
                workspace.connections = connections;
            }
            store.workspaces = vec![workspace];
            tracing::info!(
                workspace = DEFAULT_WORKSPACE_ID,
                "migrated snapshot to per-workspace storage"
            );
        }
        store.content = snapshot.content;
        store.campaigns = snapshot.campaigns;
        store.platform_tokens_enc = snapshot.platform_tokens_enc;
        store.token_salt = snapshot.token_salt;
        if store.token_salt.is_empty() {
            store.token_salt = crate::secrets::to_hex(&crate::secrets::new_salt());
        }
        store.accounts = snapshot
            .accounts
            .into_iter()
            .map(|a| (a.email.clone(), a))
            .collect();
        // Restore hashed sessions (expired ones are dropped).
        let now = Utc::now();
        store.sessions = snapshot
            .sessions
            .into_iter()
            .filter(|(_, session)| session.expires > now)
            .collect();
        tracing::info!(sessions = store.sessions.len(), "restored live sessions");
        store.migrate_platforms();
        Ok(store)
    }

    /// Normalizes connection slots written before Meta replaced the separate
    /// Facebook and Instagram providers: the `facebook` slot becomes `meta`
    /// (its handle upgraded to the Page's profile URL), the `instagram` slot is
    /// dropped — one Meta login covers both — and stored tokens follow.
    pub fn migrate_platforms(&mut self) {
        for workspace in &mut self.workspaces {
            let mut ordered: Vec<PlatformConnection> = Vec::new();
            for mut slot in blank_connections() {
                if let Some(existing) = workspace.connections.iter().find(|c| c.id == slot.id) {
                    slot = existing.clone();
                } else if slot.id == "meta" {
                    if let Some(old) = workspace.connections.iter().find(|c| c.id == "facebook") {
                        slot = old.clone();
                        slot.id = "meta".into();
                    }
                }
                if slot.id == "meta"
                    && !slot.handle.starts_with("http")
                    && !slot.external_id.is_empty()
                {
                    // Older connections stored the Page name; the profile URL is
                    // what the app links to (works for any Page id).
                    slot.handle = format!(
                        "https://www.facebook.com/profile.php?id={}",
                        slot.external_id
                    );
                }
                ordered.push(slot);
            }
            workspace.connections = ordered;
        }
        // Provider tokens are keyed `{workspace}:{platform}`; carry the Meta
        // login over and drop the retired Instagram tokens (both the working
        // copy and the sealed snapshot copy).
        for tokens in [&mut self.platform_tokens, &mut self.platform_tokens_enc] {
            let renamed: Vec<(String, String)> = tokens
                .iter()
                .filter(|(k, _)| k.ends_with(":facebook"))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (key, token) in renamed {
                tokens.remove(&key);
                let meta_key = format!("{}meta", &key[..key.len() - "facebook".len()]);
                tokens.insert(meta_key, token);
            }
            tokens.retain(|k, _| !k.ends_with(":instagram"));
        }
    }

    /// Writes this store to `path` atomically.
    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        write_atomic(path, &self.snapshot_bytes()?)
    }

    /// Applies deployment environment settings on top of the seeded/loaded
    /// state: demo mode, auth default, registration open/closed, operator
    /// recovery token, and the optional bootstrap admin account.
    /// A platform entry with no credentials/handles — connectable from the UI.
    fn disconnected_stub(c: &PlatformConnection) -> PlatformConnection {
        PlatformConnection {
            id: c.id.clone(),
            status: "disconnected".into(),
            handle: String::new(),
            external_id: String::new(),
            scopes: Vec::new(),
            token_type: "—".into(),
            expires_at: None,
            last_sync: None,
            media_count: 0,
            note: String::new(),
        }
    }

    pub fn apply_env(&mut self) {
        // Snapshots written before platform stubs existed (or an early
        // production seed) have none — restore them so platforms can connect.
        for workspace in &mut self.workspaces {
            if workspace.connections.is_empty() {
                workspace.connections = blank_connections();
            }
        }

        // English is the only UI language now; clear any legacy "TH" value.
        self.setup.language = "EN".into();

        // Built-in Owner gains new permissions as the product grows; snapshots
        // written by older versions would otherwise lock the owner out.
        if let Some(owner) = self.setup.roles.iter_mut().find(|r| r.name == "Owner") {
            for perm in KNOWN_PERMISSIONS {
                if !owner.permissions.iter().any(|p| p == perm) {
                    owner.permissions.push(perm.to_string());
                }
            }
        }

        let demo = env_bool("DEMO_MODE", false);
        self.demo_mode = demo;
        self.allow_registration = env_bool("ALLOW_REGISTRATION", demo);
        self.setup.allow_registration = self.allow_registration;
        self.admin_token = std::env::var("ADMIN_TOKEN").ok().filter(|t| t.len() >= 16);
        self.setup.auth_required = env_bool("AUTH_REQUIRED", !demo);
        // Persisted provider tokens are sealed with a key derived from the
        // operator secret, so they survive restarts without lying in the clear.
        match self.admin_token.clone() {
            Some(secret) => self.enable_token_vault(&secret),
            None => self.token_sealing_key = None,
        }

        if !demo {
            // Never keep the well-known demo credentials in production.
            self.accounts
                .retain(|email, _| !email.ends_with("@studio.local"));
            self.sessions
                .retain(|_, session| !session.email.ends_with("@studio.local"));
        } else {
            // A ready-made client account so the content-studio path can be
            // tried locally: client@studio.local / demo1234 (role Client).
            let name = "Studio Client".to_string();
            let email = "client@studio.local".to_string();
            if !self.setup.users.iter().any(|u| u.name == name) {
                self.setup.users.push(User {
                    name: name.clone(),
                    role: "Client".into(),
                });
            }
            self.accounts
                .entry(email.clone())
                .or_insert_with(|| Account {
                    name,
                    email,
                    password_hash: hash_password("demo1234"),
                    plan: default_plan(),
                    last_active: None,
                });
        }

        let email = std::env::var("ADMIN_EMAIL")
            .unwrap_or_default()
            .trim()
            .to_lowercase();
        let password = std::env::var("ADMIN_PASSWORD").unwrap_or_default();
        if !email.is_empty() && password.len() >= PASSWORD_MIN_CHARS {
            let name = self
                .accounts
                .get(&email)
                .map(|a| a.name.clone())
                .unwrap_or_else(|| email.split('@').next().unwrap_or("Admin").to_string());
            let existing = self.accounts.contains_key(&email);
            self.accounts.insert(
                email.clone(),
                Account {
                    name: name.clone(),
                    email: email.clone(),
                    password_hash: hash_password(&password),
                    plan: default_plan(),
                    last_active: None,
                },
            );
            if existing {
                self.sessions.retain(|_, s| s.email != email);
            }
            if self.setup.owner.trim().is_empty() {
                self.setup.owner = name.clone();
            }
            if !self.setup.users.iter().any(|u| u.name == name) {
                self.setup.users.push(User {
                    name,
                    role: "Owner".into(),
                });
            }
        }

        // Workspaces written before per-account ownership are handed to the
        // account that signed in most recently, so the upgrade keeps existing
        // data with the person who was using it. Demo mode stays shared.
        if !demo && self.workspaces.iter().any(|w| w.owner.is_empty()) {
            let owner = self
                .accounts
                .values()
                .filter(|a| a.last_active.is_some())
                .max_by_key(|a| a.last_active.clone())
                .map(|a| a.email.clone())
                .or_else(|| {
                    self.accounts
                        .contains_key(&email)
                        .then(|| email.clone())
                        .filter(|e| !e.is_empty())
                });
            if let Some(owner) = owner {
                let count = self
                    .workspaces
                    .iter()
                    .filter(|w| w.owner.is_empty())
                    .count();
                for workspace in &mut self.workspaces {
                    if workspace.owner.is_empty() {
                        workspace.owner = owner.clone();
                    }
                }
                tracing::info!(
                    owner = %owner,
                    count,
                    "assigned pre-ownership workspaces to the most recent account"
                );
            }
        }
    }

    pub fn seed() -> Self {
        Self {
            setup: SetupConfig {
                language: "EN".into(),
                year: 2026,
                owner: "Studio Owner".into(),
                workspace_name: "workspace".into(),
                pillars: vec!["Pillar I".into(), "Pillar II".into(), "Pillar III".into()],
                formats: vec!["Long-video".into(), "Short-video".into()],
                goals: vec!["10K subscribe".into()],
                statuses: vec!["Start".into(), "Design".into(), "Dev".into(), "Done".into()],
                platforms: vec![
                    "Facebook".into(),
                    "Instagram".into(),
                    "TikTok".into(),
                    "Youtube".into(),
                ],
                show_editable_colors: true,
                users: vec![
                    User {
                        name: "Studio Owner".into(),
                        role: "Owner".into(),
                    },
                    User {
                        name: "Editor Earn".into(),
                        role: "Editor".into(),
                    },
                ],
                auth_required: false,
                allow_registration: true,
                roles: vec![
                    Role {
                        name: "Owner".into(),
                        permissions: KNOWN_PERMISSIONS.iter().map(|p| p.to_string()).collect(),
                    },
                    Role {
                        name: "Editor".into(),
                        permissions: [
                            "posts.write",
                            "posts.lock",
                            "metrics.import",
                            "ideas.write",
                            "hashtags.write",
                            "content.write",
                            "content.publish",
                            "campaigns.write",
                        ]
                        .iter()
                        .map(|p| p.to_string())
                        .collect(),
                    },
                    Role {
                        name: "Viewer".into(),
                        permissions: vec![],
                    },
                    // Client-facing role: can use the Content Studio only.
                    Role {
                        name: "Client".into(),
                        permissions: ["content.write", "content.publish"]
                            .iter()
                            .map(|p| p.to_string())
                            .collect(),
                    },
                ],
            },
            posts: {
                let mut v = vec![
                    post(
                        "p-morning",
                        2,
                        "Morning Vlog",
                        "Pillar II",
                        "Short-video",
                        "2026-02-03",
                        "08:00",
                        "Done",
                        "POV: slow mornings",
                        "A slow morning routine before the city wakes.",
                        "Save for later",
                        "Reach",
                        &["morning", "vlog", "slow"],
                        true,
                        &["Instagram", "TikTok"],
                    ),
                    post(
                        "p-night",
                        2,
                        "Self-Care Night Routine",
                        "Pillar I",
                        "Long-video",
                        "2026-02-05",
                        "20:00",
                        "Design",
                        "Unwind with me",
                        "Full night routine, no cuts.",
                        "Subscribe",
                        "Niche",
                        &["selfcare", "night"],
                        false,
                        &["Youtube"],
                    ),
                    post(
                        "p-desk",
                        2,
                        "Desk Setup Tour",
                        "Pillar III",
                        "Short-video",
                        "2026-02-06",
                        "12:00",
                        "Start",
                        "My 2026 desk",
                        "Everything on my desk and why.",
                        "Comment yours",
                        "Reach",
                        &["desksetup"],
                        false,
                        &["TikTok", "Instagram"],
                    ),
                    post(
                        "p-budget",
                        2,
                        "Budget Breakfast",
                        "Pillar I",
                        "Short-video",
                        "2026-02-10",
                        "07:30",
                        "Dev",
                        "Eat for under $2",
                        "Three cheap breakfasts that slap.",
                        "Share this",
                        "Niche",
                        &["budget", "food"],
                        false,
                        &["Facebook", "Instagram"],
                    ),
                    post(
                        "p-feed",
                        2,
                        "Feed Aesthetic Tips",
                        "Pillar II",
                        "Long-video",
                        "2026-02-12",
                        "18:00",
                        "Start",
                        "Fix your grid",
                        "Five rules for a coherent feed.",
                        "Follow",
                        "Branded",
                        &["feed", "aesthetic"],
                        false,
                        &["Instagram"],
                    ),
                    post(
                        "p-march",
                        3,
                        "March Teaser",
                        "Pillar I",
                        "Short-video",
                        "2026-03-02",
                        "09:00",
                        "Start",
                        "Coming soon",
                        "What March looks like.",
                        "Stay tuned",
                        "Event",
                        &["march"],
                        false,
                        &["TikTok"],
                    ),
                ];
                // Mirrors the mock seed: one row already locked by another user (US-014 demo).
                if let Some(p) = v.iter_mut().find(|p| p.id == "p-night") {
                    p.locked_by = Some("Editor Earn".into());
                }
                v
            },
            ideas: vec![
                Idea {
                    id: "i-1".into(),
                    topic: "Rainy Day Reads".into(),
                    format: "Long-video".into(),
                    idea: "Cozy reading vlog".into(),
                    link: String::new(),
                    done: false,
                },
                Idea {
                    id: "i-2".into(),
                    topic: "Camera Roll Dump".into(),
                    format: "Short-video".into(),
                    idea: "Weekly photo dump".into(),
                    link: String::new(),
                    done: false,
                },
                Idea {
                    id: "i-3".into(),
                    topic: "Q&A Sunday".into(),
                    format: "Long-video".into(),
                    idea: "Answer comments on camera".into(),
                    link: String::new(),
                    done: true,
                },
            ],
            hashtags: vec![
                HashtagGroup {
                    id: "g-niche".into(),
                    title: "Niche".into(),
                    tags: vec!["selfcare".into(), "budget".into(), "desksetup".into()],
                },
                HashtagGroup {
                    id: "g-reach".into(),
                    title: "Reach".into(),
                    tags: vec!["morning".into(), "vlog".into(), "slow".into()],
                },
                HashtagGroup {
                    id: "g-branded".into(),
                    title: "Branded".into(),
                    tags: vec!["feed".into(), "aesthetic".into()],
                },
                HashtagGroup {
                    id: "g-event".into(),
                    title: "Event".into(),
                    tags: vec!["march".into()],
                },
            ],
            metrics: vec![
                Metric {
                    post_id: "p-morning".into(),
                    platform: "Instagram".into(),
                    likes: 1200,
                    views: 15000,
                },
                Metric {
                    post_id: "p-morning".into(),
                    platform: "TikTok".into(),
                    likes: 3400,
                    views: 42000,
                },
                Metric {
                    post_id: "p-night".into(),
                    platform: "Youtube".into(),
                    likes: 300,
                    views: 5200,
                },
            ],
            txns: vec![
                Txn {
                    id: "t-1".into(),
                    date: "2026-01-05".into(),
                    amount: 15000.0,
                    kind: "IN".into(),
                    category: "Sponsorship".into(),
                    sub: "Brand A".into(),
                },
                Txn {
                    id: "t-2".into(),
                    date: "2026-01-12".into(),
                    amount: 3200.0,
                    kind: "OUT".into(),
                    category: "Gear".into(),
                    sub: "Mic".into(),
                },
                Txn {
                    id: "t-3".into(),
                    date: "2026-02-03".into(),
                    amount: 15000.0,
                    kind: "IN".into(),
                    category: "Sponsorship".into(),
                    sub: "Brand A".into(),
                },
                Txn {
                    id: "t-4".into(),
                    date: "2026-02-09".into(),
                    amount: 1500.0,
                    kind: "OUT".into(),
                    category: "Props".into(),
                    sub: "Set decor".into(),
                },
                Txn {
                    id: "t-5".into(),
                    date: "2026-02-15".into(),
                    amount: 4800.0,
                    kind: "OUT".into(),
                    category: "Editing".into(),
                    sub: "Freelancer".into(),
                },
                Txn {
                    id: "t-6".into(),
                    date: "2026-02-20".into(),
                    amount: 2200.0,
                    kind: "IN".into(),
                    category: "Affiliate".into(),
                    sub: "Links".into(),
                },
            ],
            workspaces: vec![Workspace {
                id: DEFAULT_WORKSPACE_ID.into(),
                name: "workspace".into(),
                created: stamp(),
                owner: String::new(),
                members: vec![],
                brand: Brand {
                    channel: "Studio Channel".into(),
                    positioning: "Slow living, honest reviews".into(),
                    slogan: "Make room for slow".into(),
                    audience: "20–34, city creatives".into(),
                    voice: "Warm, direct, no hype".into(),
                    dos: vec!["Show the process".into(), "Credit sources".into()],
                    donts: vec!["No clickbait".into(), "No fake urgency".into()],
                    palette: vec![
                        "#111111".into(),
                        "#555555".into(),
                        "#999999".into(),
                        "#CCCCCC".into(),
                    ],
                    fonts: vec!["Noto Sans Thai".into(), "DejaVu Sans".into()],
                    logos: vec![],
                    moodboard: vec![],
                    radius: 12,
                    fill_opacity: 100,
                    stroke_width: 1,
                    shadow: "soft".into(),
                },
                connections: {
                    // Demo seed: the Meta slot arrives connected so the
                    // wireframe shows a realistic account.
                    let mut conns = blank_connections();
                    if let Some(c) = conns.iter_mut().find(|c| c.id == "meta") {
                        c.status = "connected".into();
                        c.handle = "https://www.facebook.com/profile.php?id=102400000000001".into();
                        c.external_id = "102400000000001".into();
                        c.scopes = vec![
                            "pages_show_list".into(),
                            "pages_read_engagement".into(),
                            "pages_manage_posts".into(),
                            "read_insights".into(),
                            "business_management".into(),
                            "instagram_basic".into(),
                            "instagram_manage_insights".into(),
                            "ads_read".into(),
                            "ads_management".into(),
                        ];
                        c.token_type = "long-lived".into();
                        c.expires_at = Some("2026-10-18".into());
                        c.last_sync = Some("2026-03-01 06:00".into());
                        c.media_count = 214;
                    }
                    conns
                },
                live: seed_live(),
                ads: seed_ads(),
                ads_manage: false,
            }],
            content: seed_content(),
            campaigns: seed_campaigns(),
            accounts: {
                let mut accounts = HashMap::new();
                for (name, email) in [
                    ("Studio Owner", "owner@studio.local"),
                    ("Editor Earn", "editor@studio.local"),
                ] {
                    accounts.insert(
                        email.to_string(),
                        Account {
                            name: name.into(),
                            email: email.into(),
                            password_hash: hash_password("demo1234"),
                            plan: default_plan(),
                            last_active: None,
                        },
                    );
                }
                accounts
            },
            sessions: HashMap::new(),
            oauth: crate::oauth::OauthSettings::from_env(),
            oauth_states: HashMap::new(),
            oauth_picks: HashMap::new(),
            platform_tokens: HashMap::new(),
            platform_tokens_enc: HashMap::new(),
            token_salt: crate::secrets::to_hex(&crate::secrets::new_salt()),
            token_sealing_key: None,
            login_attempts: HashMap::new(),
            demo_mode: true,
            allow_registration: true,
            admin_token: None,
            fonts_dir: None,
            images_dir: None,
        }
    }

    /// A clean production workspace: the same config defaults as [`Self::seed`]
    /// (platforms, pillars, statuses, roles) but without the wireframe demo
    /// rows, demo users, or demo accounts. `apply_env` then adds the bootstrap
    /// Owner from `ADMIN_EMAIL`/`ADMIN_PASSWORD`.
    pub fn fresh() -> Self {
        let mut store = Self::seed();
        store.posts.clear();
        store.ideas.clear();
        store.hashtags.clear();
        store.metrics.clear();
        store.txns.clear();
        store.content.clear();
        store.campaigns.clear();
        // Keep one disconnected stub per platform so the UI can connect them
        // (the demo handles/tokens are dropped).
        for workspace in &mut store.workspaces {
            workspace.connections = workspace
                .connections
                .iter()
                .map(Self::disconnected_stub)
                .collect();
            workspace.brand = Brand::empty();
            workspace.live = crate::model::LiveData::default();
            workspace.ads = crate::model::AdsData::default();
            workspace.ads_manage = false;
        }
        store.setup.users.clear();
        store.setup.owner = String::new();
        store.accounts.clear();
        store.sessions.clear();
        store.demo_mode = false;
        store.allow_registration = false;
        store
    }
}

/// True when the process should boot with the wireframe demo seed
/// (`DEMO_MODE=1`); production deployments leave it unset.
pub fn demo_env() -> bool {
    env_bool("DEMO_MODE", false)
}

fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_writes_owner_only_files() {
        let dir = std::env::temp_dir().join(format!("cp-store-test-{}", random_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.json");

        write_atomic(&path, br#"{"ok":true}"#).unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), br#"{"ok":true}"#);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "snapshot must not be group/world readable");
        }
        // no temp files left behind
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp file was not renamed away");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn snapshot_never_contains_session_tokens() {
        let mut store = Store::seed();
        let token = store.open_session("owner@studio.local");
        assert_eq!(token.len(), 64);

        let json = String::from_utf8(store.snapshot_bytes().unwrap()).unwrap();
        assert!(
            !json.contains(&token),
            "live token leaked into the snapshot"
        );
        assert!(
            json.contains(&session_key(&token)),
            "only the hash is persisted"
        );
    }

    #[test]
    fn platform_tokens_persist_sealed_and_reload() {
        let secret = "unit-test-admin-secret-0123456789";
        let mut store = Store::seed();
        store.enable_token_vault(secret);
        store.set_platform_token(DEFAULT_WORKSPACE_ID, "meta", "EAAB-page-token");
        assert_eq!(
            store
                .platform_token(DEFAULT_WORKSPACE_ID, "meta")
                .as_deref(),
            Some("EAAB-page-token")
        );

        let bytes = store.snapshot_bytes().unwrap();
        let json = String::from_utf8(bytes.clone()).unwrap();
        assert!(
            !json.contains("EAAB-page-token"),
            "plaintext token leaked into the snapshot"
        );
        assert!(json.contains("platform_tokens_enc"), "{json}");

        let dir = std::env::temp_dir().join(format!("cp-vault-test-{}", random_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.json");
        std::fs::write(&path, &bytes).unwrap();

        let mut reloaded = Store::load_or_seed(&path).unwrap();
        reloaded.enable_token_vault(secret);
        assert_eq!(
            reloaded
                .platform_token(DEFAULT_WORKSPACE_ID, "meta")
                .as_deref(),
            Some("EAAB-page-token"),
            "sealed token must survive a restart"
        );

        // A different deployment secret cannot open the vault.
        let mut stranger = Store::load_or_seed(&path).unwrap();
        stranger.enable_token_vault("another-admin-secret-9876543210");
        assert!(stranger
            .platform_token(DEFAULT_WORKSPACE_ID, "meta")
            .is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn sessions_survive_a_restart_as_hashes() {
        let mut store = Store::seed();
        let token = store.open_session("owner@studio.local");
        assert!(store.session(&token).is_some(), "session opens");

        let bytes = store.snapshot_bytes().unwrap();
        let json = String::from_utf8(bytes.clone()).unwrap();
        assert!(
            !json.contains(&token),
            "the raw bearer token must never be persisted"
        );

        let dir = std::env::temp_dir().join(format!("cp-session-test-{}", random_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.json");
        std::fs::write(&path, &bytes).unwrap();

        let mut reloaded = Store::load_or_seed(&path).unwrap();
        assert!(
            reloaded.session(&token).is_some(),
            "a restart keeps the user signed in"
        );
        reloaded.revoke_session(&token);
        assert!(reloaded.session(&token).is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn platform_tokens_stay_in_memory_without_a_vault() {
        let mut store = Store::seed();
        store.set_platform_token(DEFAULT_WORKSPACE_ID, "meta", "EAAB-page-token");
        let json = String::from_utf8(store.snapshot_bytes().unwrap()).unwrap();
        assert!(!json.contains("EAAB-page-token"));
        assert!(
            !json.contains("platform_tokens_enc"),
            "nothing sealed without a vault"
        );
    }

    #[test]
    fn clearing_tokens_removes_both_copies() {
        let mut store = Store::seed();
        store.enable_token_vault("unit-test-admin-secret-0123456789");
        store.set_platform_token(DEFAULT_WORKSPACE_ID, "meta", "tok");
        store.clear_platform_token(DEFAULT_WORKSPACE_ID, "meta");
        assert!(store.platform_token(DEFAULT_WORKSPACE_ID, "meta").is_none());
        assert!(store.platform_tokens_enc.is_empty());

        store.set_platform_token(DEFAULT_WORKSPACE_ID, "meta", "tok2");
        store.clear_workspace_tokens(DEFAULT_WORKSPACE_ID);
        assert!(store.platform_tokens.is_empty());
        assert!(store.platform_tokens_enc.is_empty());
    }
}
