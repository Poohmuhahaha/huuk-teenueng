//! Domain types. Field names are camelCase on the wire to match the frontend.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    #[serde(default)]
    pub id: String,
    pub month: u32,
    pub topic: String,
    pub pillar: String,
    pub format: String,
    pub goal: String,
    pub date: Option<String>,
    pub time: String,
    pub status: String,
    pub hook: String,
    pub caption: String,
    pub cta: String,
    pub hashtag_group: String,
    pub hashtags: Vec<String>,
    pub image_url: String,
    pub note: String,
    pub done: bool,
    pub platforms: Vec<String>,
    /// Always serialized (`lockedBy: null` when unlocked) — simpler for the frontend.
    pub locked_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupConfig {
    pub language: String,
    pub year: i32,
    pub owner: String,
    /// Navbar workspace label; editable in Settings (older snapshots default it).
    #[serde(default = "default_workspace_name")]
    pub workspace_name: String,
    pub pillars: Vec<String>,
    pub formats: Vec<String>,
    pub goals: Vec<String>,
    pub statuses: Vec<String>,
    pub platforms: Vec<String>,
    pub show_editable_colors: bool,
    pub users: Vec<User>,
    pub auth_required: bool,
    /// Whether the login page offers self-service registration (env-driven).
    #[serde(default)]
    pub allow_registration: bool,
    pub roles: Vec<Role>,
}

/// A self-hosted brand font uploaded from the Brand Identity page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontAsset {
    /// Stored file name (slug + extension).
    pub name: String,
    /// CSS family name derived from the upload.
    pub family: String,
    /// Public URL the app loads with @font-face.
    pub url: String,
    pub size: u64,
    pub uploaded_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub name: String,
    pub role: String,
}

fn default_workspace_name() -> String {
    "workspace".to_string()
}

/// Permissions the server understands; role payloads are validated against this list.
pub const KNOWN_PERMISSIONS: [&str; 15] = [
    "setup.write",
    "users.manage",
    "posts.write",
    "posts.lock",
    "metrics.import",
    "finance.write",
    "brand.write",
    "ideas.write",
    "hashtags.write",
    "platforms.manage",
    "content.write",
    "content.publish",
    "content.delete",
    "campaigns.write",
    "campaigns.delete",
];

/// Statuses a content item can be in (CMS workflow).
pub const CONTENT_STATUSES: [&str; 5] = ["draft", "review", "scheduled", "published", "archived"];

/// Content kinds the studio understands.
pub const CONTENT_KINDS: [&str; 3] = ["article", "page", "note"];

/// Maximum stored revisions per content item (oldest dropped).
pub const MAX_CONTENT_REVISIONS: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub name: String,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Idea {
    #[serde(default)]
    pub id: String,
    pub topic: String,
    pub format: String,
    pub idea: String,
    pub link: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashtagGroup {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub post_id: String,
    pub platform: String,
    pub likes: u64,
    pub views: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Txn {
    #[serde(default)]
    pub id: String,
    pub date: String,
    pub amount: f64,
    pub kind: String,
    pub category: String,
    pub sub: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brand {
    pub channel: String,
    pub positioning: String,
    pub slogan: String,
    pub audience: String,
    pub voice: String,
    pub dos: Vec<String>,
    pub donts: Vec<String>,
    pub palette: Vec<String>,
    pub fonts: Vec<String>,
    /// Logo image URLs (main, secondary, social — blanks allowed).
    #[serde(default)]
    pub logos: Vec<String>,
    /// Moodboard image URLs (references, product shots, ...).
    #[serde(default)]
    pub moodboard: Vec<String>,
    /// Corner radius in px for the whole site (0–24, default 12).
    #[serde(default = "default_radius")]
    pub radius: u8,
    /// Accent fill strength in percent for solid surfaces (5–100, default 100).
    #[serde(default = "default_fill_opacity")]
    pub fill_opacity: u8,
    /// Card/panel border width in px (0–3, default 1).
    #[serde(default = "default_stroke_width")]
    pub stroke_width: u8,
    /// Shadow depth: none | soft | strong (default soft).
    #[serde(default = "default_shadow")]
    pub shadow: String,
}

fn default_radius() -> u8 {
    12
}
fn default_fill_opacity() -> u8 {
    100
}
fn default_stroke_width() -> u8 {
    1
}
fn default_shadow() -> String {
    "soft".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformConnection {
    pub id: String,
    pub status: String,
    pub handle: String,
    pub external_id: String,
    pub scopes: Vec<String>,
    pub token_type: String,
    pub expires_at: Option<String>,
    pub last_sync: Option<String>,
    pub media_count: u32,
    pub note: String,
}

/// One post mirrored from a connected platform. Read-only: refreshed by sync
/// and the background live-refresh loop, never edited in the app.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LivePost {
    /// Platform post id (unique per platform).
    pub id: String,
    /// Which connection produced it: "meta" for the Facebook Page,
    /// "instagram" for media of the linked Instagram account.
    pub platform: String,
    /// post | photo | video | reel | carousel
    pub kind: String,
    pub caption: String,
    pub media_url: String,
    pub thumbnail_url: String,
    pub permalink: String,
    pub created_at: String,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    /// Plays when the platform reports them; 0 = not reported.
    pub views: u64,
    /// Reaction counts by type (like, love, haha, wow, sad, angry) when Meta
    /// reports them; empty when the post has none or insights are unavailable.
    #[serde(default)]
    pub reactions: std::collections::BTreeMap<String, u64>,
    /// Accounts reached (Instagram reports it; Facebook deprecated it).
    #[serde(default)]
    pub reach: u64,
    /// Saves (Instagram reports it; 0 elsewhere).
    #[serde(default)]
    pub saves: u64,
    /// Total clicks (`post_clicks_by_type`: link clicks, photo views, ...).
    #[serde(default)]
    pub clicks: u64,
    /// Link clicks alone (a subset of `clicks`).
    #[serde(default)]
    pub link_clicks: u64,
    /// Video length in seconds (videos/reels only; 0 = not a video).
    #[serde(default)]
    pub video_length: u64,
    /// Average watch time in seconds (videos/reels only; 0 = not reported).
    #[serde(default)]
    pub video_avg_watch_time: f64,
    /// Attachment title and destination (link posts carry the target URL).
    #[serde(default)]
    pub attachment_title: String,
    #[serde(default)]
    pub link_url: String,
    /// Unix seconds of the refresh that captured this post.
    pub fetched_at: i64,
}

/// Account-level stats captured on the last refresh.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LiveAccount {
    pub platform: String,
    pub id: String,
    pub username: String,
    pub followers: u64,
    pub posts: u64,
    /// Page-level insights (Meta reports 28-day/lifetime windows).
    #[serde(default)]
    pub engagements: u64,
    /// Net new follows over the reported window (can be negative).
    #[serde(default)]
    pub net_follows: i64,
    #[serde(default)]
    pub page_views: u64,
    #[serde(default)]
    pub video_views: u64,
}

/// Mirror of the connected platform's content for one workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LiveData {
    /// Unix seconds of the last successful refresh (`null` = never).
    pub fetched_at: Option<i64>,
    pub accounts: Vec<LiveAccount>,
    pub posts: Vec<LivePost>,
    /// Last refresh failure, kept so the UI can explain stale content.
    #[serde(default)]
    pub error: String,
}

// ---- Meta Ads mirror (read) + management (write) ----
//
// Names are prefixed with `Ad` because `Campaign` above is the *content*
// planner campaign; these are the platform's advertising objects.

/// A Meta ad account the login can see.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdAccount {
    /// `act_<id>` — the id every Graph ads endpoint uses.
    pub id: String,
    pub name: String,
    /// active | disabled | unsettled | ... (Graph `account_status` decoded).
    pub status: String,
    pub currency: String,
    pub timezone: String,
    /// Owning Business portfolio name, when the account belongs to one.
    #[serde(default)]
    pub business: String,
}

/// One ad campaign with its settings. Budgets are in the account's minor
/// currency unit (as Graph reports them), e.g. 50000 = 500.00 THB.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdCampaign {
    pub id: String,
    pub account_id: String,
    pub name: String,
    /// Graph objective, e.g. OUTCOME_TRAFFIC / OUTCOME_SALES.
    pub objective: String,
    /// Configured status: ACTIVE | PAUSED | DELETED | ARCHIVED.
    pub status: String,
    /// Effective status (may differ: campaign paused because the account is).
    #[serde(default)]
    pub effective_status: String,
    #[serde(default)]
    pub buying_type: String,
    #[serde(default)]
    pub daily_budget: u64,
    #[serde(default)]
    pub lifetime_budget: u64,
    #[serde(default)]
    pub budget_remaining: u64,
    #[serde(default)]
    pub bid_strategy: String,
    #[serde(default)]
    pub special_ad_categories: Vec<String>,
    #[serde(default)]
    pub start_time: String,
    #[serde(default)]
    pub stop_time: String,
    #[serde(default)]
    pub created_time: String,
}

/// One ad set (audience/budget level under a campaign).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdSet {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub effective_status: String,
    #[serde(default)]
    pub daily_budget: u64,
    #[serde(default)]
    pub lifetime_budget: u64,
    #[serde(default)]
    pub optimization_goal: String,
    #[serde(default)]
    pub billing_event: String,
    #[serde(default)]
    pub bid_amount: u64,
    #[serde(default)]
    pub start_time: String,
    #[serde(default)]
    pub end_time: String,
    /// Short human summary of targeting (age/gender/geo/interests count).
    #[serde(default)]
    pub targeting: String,
    /// Promoted object (page id, pixel id, ...) summarized as text.
    #[serde(default)]
    pub promoted_object: String,
}

/// One ad with a summary of its creative.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Ad {
    pub id: String,
    pub adset_id: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub effective_status: String,
    #[serde(default)]
    pub creative_title: String,
    #[serde(default)]
    pub creative_body: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub thumbnail_url: String,
    /// Page post this ad promotes (matches `LivePost.id` for boosted posts).
    #[serde(default)]
    pub story_id: String,
    #[serde(default)]
    pub preview_url: String,
}

/// Last-30-days performance for one campaign.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdInsight {
    pub campaign_id: String,
    pub spend: f64,
    pub impressions: u64,
    pub reach: u64,
    pub frequency: f64,
    pub clicks: u64,
    pub ctr: f64,
    pub cpc: f64,
    pub cpm: f64,
    /// Meaningful result count for the campaign's objective.
    #[serde(default)]
    pub results: u64,
    /// What the result counts, e.g. "Leads", "Purchases", "Link clicks".
    #[serde(default)]
    pub result_label: String,
    /// Purchase ROAS when Meta reports it (0 = not reported).
    #[serde(default)]
    pub roas: f64,
}

/// One management action, kept for the workspace's audit trail.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdsAudit {
    pub at: String,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub detail: String,
}

/// The workspace's Meta Ads mirror: accounts, structure, settings, and the
/// last-30-days performance. Written by sync / the background refresh and by
/// the management endpoints (which append to `audit`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdsData {
    /// Unix seconds of the last successful refresh (`null` = never).
    pub fetched_at: Option<i64>,
    pub accounts: Vec<AdAccount>,
    pub campaigns: Vec<AdCampaign>,
    pub adsets: Vec<AdSet>,
    pub ads: Vec<Ad>,
    pub insights: Vec<AdInsight>,
    /// Newest first, capped (see `crate::ads::AUDIT_LIMIT`).
    #[serde(default)]
    pub audit: Vec<AdsAudit>,
    /// Last refresh failure, kept so the UI can explain stale data.
    #[serde(default)]
    pub error: String,
}

impl Brand {
    /// A blank brand for freshly created workspaces.
    pub fn empty() -> Self {
        Self {
            channel: String::new(),
            positioning: String::new(),
            slogan: String::new(),
            audience: String::new(),
            voice: String::new(),
            dos: vec![],
            donts: vec![],
            palette: vec![],
            fonts: vec![],
            logos: vec![],
            moodboard: vec![],
            radius: 12,
            fill_opacity: 100,
            stroke_width: 1,
            shadow: "soft".into(),
        }
    }
}

/// A workspace: one brand identity plus its own social media connections.
/// Requests select one with the `X-Workspace-Id` header (falling back to the
/// first workspace); provider tokens are stored per workspace so two brands
/// never share a login.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub created: String,
    /// Account email that owns this workspace. Empty means shared/legacy
    /// (demo deployments and snapshots written before per-account ownership).
    #[serde(default)]
    pub owner: String,
    /// Lowercased account emails the owner added as members. Excludes the
    /// owner; empty for legacy snapshots (the field defaults in).
    #[serde(default)]
    pub members: Vec<String>,
    pub brand: Brand,
    pub connections: Vec<PlatformConnection>,
    /// Content mirrored from the connected platforms (see `crate::live`).
    #[serde(default)]
    pub live: LiveData,
    /// Meta Ads mirror (see `crate::ads`).
    #[serde(default)]
    pub ads: AdsData,
    /// Explicit opt-in: the owner allows Huuk to change campaigns (pause,
    /// budgets, create). Writes stay impossible until this is true.
    #[serde(default)]
    pub ads_manage: bool,
}

/// Wire shape of a workspace for the switcher (no brand/connection payloads).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
    pub id: String,
    pub name: String,
    pub created: String,
    /// Platforms currently connected in this workspace.
    pub connected: usize,
    /// Platform slots the workspace has (usually 4).
    pub total: usize,
    /// Whether the requesting account owns this workspace.
    pub is_owner: bool,
}

impl Workspace {
    /// Whether `account` may see and manage this workspace. `None` (demo mode
    /// or the operator recovery token) sees everything; shared workspaces
    /// (empty owner) stay visible to every account; members see the workspace
    /// their owner added them to.
    pub fn visible_to(&self, account: Option<&str>) -> bool {
        self.owner.is_empty()
            || account.is_none()
            || account == Some(self.owner.as_str())
            || account.is_some_and(|a| self.members.iter().any(|m| m.eq_ignore_ascii_case(a)))
    }

    pub fn summary(&self, account: Option<&str>) -> WorkspaceSummary {
        WorkspaceSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            created: self.created.clone(),
            connected: self
                .connections
                .iter()
                .filter(|c| c.status == "connected")
                .count(),
            total: self.connections.len(),
            is_owner: account.is_some()
                && account == Some(self.owner.as_str())
                && !self.owner.is_empty(),
        }
    }
}

// ---- partial-update payloads ----

/// Distinguishes an absent field (`None`) from an explicit `null`
/// (`Some(None)`) so PATCH can clear optional values such as `date`.
fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PostPatch {
    pub month: Option<u32>,
    pub topic: Option<String>,
    pub pillar: Option<String>,
    pub format: Option<String>,
    pub goal: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub date: Option<Option<String>>,
    pub time: Option<String>,
    pub status: Option<String>,
    pub hook: Option<String>,
    pub caption: Option<String>,
    pub cta: Option<String>,
    pub hashtag_group: Option<String>,
    pub hashtags: Option<Vec<String>>,
    pub image_url: Option<String>,
    pub note: Option<String>,
    pub done: Option<bool>,
    pub platforms: Option<Vec<String>>,
    /// Who is patching — consumed by the lock check in `update_post`, never stored.
    pub user: Option<String>,
}

impl PostPatch {
    pub fn apply(self, p: &mut Post) {
        if let Some(v) = self.month {
            p.month = v;
        }
        if let Some(v) = self.topic {
            p.topic = v;
        }
        if let Some(v) = self.pillar {
            p.pillar = v;
        }
        if let Some(v) = self.format {
            p.format = v;
        }
        if let Some(v) = self.goal {
            p.goal = v;
        }
        if let Some(v) = self.date {
            p.date = v;
        }
        if let Some(v) = self.time {
            p.time = v;
        }
        if let Some(v) = self.status {
            p.status = v;
        }
        if let Some(v) = self.hook {
            p.hook = v;
        }
        if let Some(v) = self.caption {
            p.caption = v;
        }
        if let Some(v) = self.cta {
            p.cta = v;
        }
        if let Some(v) = self.hashtag_group {
            p.hashtag_group = v;
        }
        if let Some(v) = self.hashtags {
            p.hashtags = v;
        }
        if let Some(v) = self.image_url {
            p.image_url = v;
        }
        if let Some(v) = self.note {
            p.note = v;
        }
        if let Some(v) = self.done {
            p.done = v;
        }
        if let Some(v) = self.platforms {
            p.platforms = v;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SetupPatch {
    pub language: Option<String>,
    pub year: Option<i32>,
    pub owner: Option<String>,
    pub workspace_name: Option<String>,
    pub show_editable_colors: Option<bool>,
    pub auth_required: Option<bool>,
    pub roles: Option<Vec<Role>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BrandPatch {
    pub channel: Option<String>,
    pub positioning: Option<String>,
    pub slogan: Option<String>,
    pub audience: Option<String>,
    pub voice: Option<String>,
    pub dos: Option<Vec<String>>,
    pub donts: Option<Vec<String>>,
    pub palette: Option<Vec<String>>,
    pub fonts: Option<Vec<String>>,
    pub logos: Option<Vec<String>>,
    pub moodboard: Option<Vec<String>>,
    pub radius: Option<u8>,
    pub fill_opacity: Option<u8>,
    pub stroke_width: Option<u8>,
    pub shadow: Option<String>,
}

// ---- CMS: content items + revisions ----

/// A publishable content item (article/page/note) with an editorial workflow.
/// `version` implements optimistic concurrency: PATCH/publish requests send the
/// version they last read and get a 409 when someone else saved in between.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(default)]
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default = "default_content_kind")]
    pub kind: String,
    #[serde(default = "default_content_status")]
    pub status: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub excerpt: String,
    #[serde(default)]
    pub hero_image_url: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub seo_title: String,
    #[serde(default)]
    pub seo_description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub scheduled_for: Option<String>,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub revisions: Vec<ContentRevision>,
}

fn default_content_kind() -> String {
    "article".into()
}

fn default_content_status() -> String {
    "draft".into()
}

/// A marketing campaign: brief, schedule window, targets and linked content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Campaign {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub objective: String,
    /// draft | active | paused | completed
    #[serde(default = "default_campaign_status")]
    pub status: String,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub pillars: Vec<String>,
    #[serde(default)]
    pub hashtags: Vec<String>,
    #[serde(default)]
    pub budget: f64,
    /// views | likes | reach | posts
    #[serde(default = "default_campaign_metric")]
    pub goal_metric: String,
    #[serde(default)]
    pub goal_target: f64,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub notes: String,
    /// Linked Content Studio items (scheduled on their own dates).
    #[serde(default)]
    pub content_ids: Vec<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CampaignPatch {
    pub name: Option<String>,
    pub objective: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub platforms: Option<Vec<String>>,
    pub pillars: Option<Vec<String>>,
    pub hashtags: Option<Vec<String>>,
    pub budget: Option<f64>,
    pub goal_metric: Option<String>,
    pub goal_target: Option<f64>,
    pub owner: Option<String>,
    pub notes: Option<String>,
    pub content_ids: Option<Vec<String>>,
}

fn default_campaign_status() -> String {
    "draft".into()
}

fn default_campaign_metric() -> String {
    "views".into()
}

/// An immutable snapshot of a content item, captured before each edit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentRevision {
    pub revision: u32,
    pub saved_at: String,
    pub author: String,
    pub note: String,
    pub title: String,
    pub body: String,
    pub excerpt: String,
    pub seo_title: String,
    pub seo_description: String,
    pub tags: Vec<String>,
    pub status: String,
}

/// Lightweight list row (no body/revisions) for the studio sidebar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSummary {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub kind: String,
    pub status: String,
    pub excerpt: String,
    pub hero_image_url: String,
    pub tags: Vec<String>,
    pub author: String,
    pub updated_at: String,
    pub published_at: Option<String>,
    pub scheduled_for: Option<String>,
    pub version: u32,
    pub revision_count: usize,
    pub word_count: usize,
}

impl Content {
    pub fn summary(&self) -> ContentSummary {
        ContentSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            slug: self.slug.clone(),
            kind: self.kind.clone(),
            status: self.status.clone(),
            excerpt: self.excerpt.clone(),
            hero_image_url: self.hero_image_url.clone(),
            tags: self.tags.clone(),
            author: self.author.clone(),
            updated_at: self.updated_at.clone(),
            published_at: self.published_at.clone(),
            scheduled_for: self.scheduled_for.clone(),
            version: self.version,
            revision_count: self.revisions.len(),
            word_count: self.body.split_whitespace().count(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ContentPatch {
    /// Version the client last read — required for the concurrency check.
    pub version: Option<u32>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub kind: Option<String>,
    /// Only `draft`/`review`/`archived` may be set through a patch;
    /// publishing/scheduling use their own endpoints.
    pub status: Option<String>,
    pub body: Option<String>,
    pub excerpt: Option<String>,
    pub hero_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    /// Free-text revision note stored with the pre-edit snapshot.
    pub note: Option<String>,
}

impl ContentPatch {
    /// Applies the editable fields onto a candidate item.
    pub fn apply(self, c: &mut Content) {
        if let Some(v) = self.title {
            c.title = v;
        }
        if let Some(v) = self.slug {
            c.slug = v;
        }
        if let Some(v) = self.kind {
            c.kind = v;
        }
        if let Some(v) = self.status {
            c.status = v;
        }
        if let Some(v) = self.body {
            c.body = v;
        }
        if let Some(v) = self.excerpt {
            c.excerpt = v;
        }
        if let Some(v) = self.hero_image_url {
            c.hero_image_url = v;
        }
        if let Some(v) = self.tags {
            c.tags = v;
        }
        if let Some(v) = self.seo_title {
            c.seo_title = v;
        }
        if let Some(v) = self.seo_description {
            c.seo_description = v;
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleBody {
    pub version: u32,
    pub scheduled_for: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionBody {
    pub version: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAccountBody {
    pub name: String,
    pub email: String,
    pub role: String,
    /// Optional initial password; when omitted the server generates one and
    /// returns it once so the admin can hand it to the client.
    #[serde(default)]
    pub password: String,
}
