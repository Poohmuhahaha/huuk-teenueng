//! Real OAuth 2.0 authorization-code flow for social platforms.
//!
//! The flow is production-shaped but credential-optional:
//! - No client credentials in the environment → `mode: "mock"` (the wireframe simulation
//!   keeps working, and the callback marks the platform connected locally).
//! - Credentials present (`META_APP_ID`/`META_APP_SECRET`, `GOOGLE_CLIENT_ID`/`GOOGLE_CLIENT_SECRET`,
//!   `TIKTOK_CLIENT_KEY`/`TIKTOK_CLIENT_SECRET`) → `mode: "redirect"`: the frontend sends the
//!   browser to the provider's hosted login, and the callback exchanges the code for a token,
//!   resolves the account, and stores it on the connection.
//!
//! Security notes:
//! - `state` is 128 bits of CSPRNG randomness, single-use, expires after
//!   [`STATE_TTL_MIN`] minutes (CSRF protection).
//! - Credentials are only ever entered on the provider's own domain — this server never sees a
//!   social password (the frontend hides its demo login form in redirect mode).
//! - Client secrets stay server-side; the frontend only receives the authorize URL.
//! - The store lock is released before any outbound HTTP call, and every
//!   provider request has connect/read timeouts, so a slow provider cannot
//!   block the whole API.
//! - Access tokens live in memory (wireframe store); a real deployment persists
//!   them in a secrets manager or encrypted at rest.
use std::collections::HashMap;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Redirect,
    Json,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use url::Url;

use crate::error::{ApiError, AppJson, AppQuery};
use crate::store::{random_hex, stamp, uid, Store};
use crate::AppState;

/// How long a pending `state` stays valid.
pub const STATE_TTL_MIN: i64 = 10;

pub(crate) const GRAPH: &str = "https://graph.facebook.com/v26.0";

#[derive(Debug, Clone, Copy)]
pub struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    /// Which environment credential pair signs this provider.
    pub cred_key: &'static str,
    pub authorize_url: &'static str,
    pub token_url: &'static str,
    pub scopes: &'static [&'static str],
    pub scope_sep: &'static str,
    pub extra_auth_params: &'static [(&'static str, &'static str)],
}

pub const PROVIDERS: &[Provider] = &[
    Provider {
        // One Meta login covers the Facebook Page and its linked Instagram
        // account (the Instagram Graph API is part of the same platform), so
        // the app exposes a single "Meta" connection.
        id: "meta",
        name: "Meta",
        cred_key: "meta",
        authorize_url: "https://www.facebook.com/v26.0/dialog/oauth",
        token_url: "https://graph.facebook.com/v26.0/oauth/access_token",
        scopes: &[
            "pages_show_list",
            "pages_read_engagement",
            "pages_manage_posts",
            "read_insights",
            // Meta's `/me/accounts` omits Pages that belong to a Business
            // portfolio unless this scope is granted (documented limitation).
            "business_management",
            "instagram_basic",
            "instagram_manage_insights",
            // Meta Ads mirror + management (see `crate::ads`): reading
            // campaigns/settings/insights needs ads_read, changing them needs
            // ads_management. Both work for the owner's own ad accounts at the
            // Marketing API's default (Limited) tier — no App Review.
            "ads_read",
            "ads_management",
        ],
        scope_sep: ",",
        // `rerequest` re-shows the consent dialog on every retry so a Page
        // unticked in a previous attempt can be selected again.
        extra_auth_params: &[("auth_type", "rerequest")],
    },
    Provider {
        id: "youtube",
        name: "YouTube",
        cred_key: "google",
        authorize_url: "https://accounts.google.com/o/oauth2/v2/auth",
        token_url: "https://oauth2.googleapis.com/token",
        scopes: &[
            "https://www.googleapis.com/auth/youtube.readonly",
            "https://www.googleapis.com/auth/yt-analytics.readonly",
        ],
        scope_sep: " ",
        extra_auth_params: &[("access_type", "offline"), ("prompt", "consent")],
    },
    Provider {
        id: "tiktok",
        name: "TikTok",
        cred_key: "tiktok",
        authorize_url: "https://www.tiktok.com/v2/auth/authorize/",
        token_url: "https://open.tiktokapis.com/v2/oauth/token/",
        scopes: &["user.info.basic", "video.list"],
        scope_sep: ",",
        extra_auth_params: &[],
    },
];

pub fn provider(id: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|p| p.id == id)
}

/// OAuth client credentials + URLs (from the environment; empty ⇒ mock mode).
#[derive(Clone)]
pub struct OauthSettings {
    pub public_url: String,
    pub frontend_url: String,
    /// Graph API base used for Meta account/live calls. Overridable with
    /// `META_GRAPH_BASE` so staging and tests can point at a fake Graph.
    pub graph_base: String,
    client_ids: HashMap<String, String>,
    client_secrets: HashMap<String, String>,
    client: reqwest::Client,
}

impl std::fmt::Debug for OauthSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print client secrets.
        let configured: Vec<&str> = self
            .client_ids
            .keys()
            .filter(|key| self.client_secrets.get(*key).is_some_and(|s| !s.is_empty()))
            .map(String::as_str)
            .collect();
        f.debug_struct("OauthSettings")
            .field("public_url", &self.public_url)
            .field("frontend_url", &self.frontend_url)
            .field("configured", &configured)
            .finish()
    }
}

impl OauthSettings {
    /// Shared HTTP client (connect/read timeouts), used by the live mirror too.
    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.client
    }

    pub fn from_env() -> Self {
        let env = |k: &str| std::env::var(k).unwrap_or_default();
        let mut client_ids = HashMap::new();
        let mut client_secrets = HashMap::new();
        for (key, id_var, secret_var) in [
            ("meta", "META_APP_ID", "META_APP_SECRET"),
            ("google", "GOOGLE_CLIENT_ID", "GOOGLE_CLIENT_SECRET"),
            ("tiktok", "TIKTOK_CLIENT_KEY", "TIKTOK_CLIENT_SECRET"),
        ] {
            client_ids.insert(key.to_string(), env(id_var));
            client_secrets.insert(key.to_string(), env(secret_var));
        }
        Self {
            public_url: std::env::var("PUBLIC_URL")
                .unwrap_or_else(|_| "http://localhost:8787".into()),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".into()),
            graph_base: std::env::var("META_GRAPH_BASE")
                .ok()
                .map(|v| v.trim().trim_end_matches('/').to_string())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| GRAPH.to_string()),
            client_ids,
            client_secrets,
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(20))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub fn credentials(&self, p: &Provider) -> Option<(&str, &str)> {
        let id = self.client_ids.get(p.cred_key)?.as_str();
        let secret = self.client_secrets.get(p.cred_key)?.as_str();
        (!id.is_empty() && !secret.is_empty()).then_some((id, secret))
    }

    /// Points Meta calls at another Graph-compatible base (tests, staging).
    pub fn set_graph_base(&mut self, base: &str) {
        self.graph_base = base.trim_end_matches('/').to_string();
    }

    /// Inject or replace credentials at runtime (used by tests and future admin config).
    pub fn set_credentials(&mut self, cred_key: &str, client_id: &str, client_secret: &str) {
        self.client_ids
            .insert(cred_key.to_string(), client_id.to_string());
        self.client_secrets
            .insert(cred_key.to_string(), client_secret.to_string());
    }

    pub fn configured(&self, p: &Provider) -> bool {
        self.credentials(p).is_some()
    }

    pub fn callback_url(&self, p: &Provider) -> String {
        format!(
            "{}/api/oauth/{}/callback",
            self.public_url.trim_end_matches('/'),
            p.id
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthState {
    pub platform: String,
    pub created: DateTime<Utc>,
    /// Workspace the connect flow was started from; the callback connects the
    /// account into this workspace only.
    #[serde(default)]
    pub workspace_id: String,
}

/// An unfinished OAuth connect: the provider returned several connectable
/// accounts and the user has not chosen one yet. Memory-only (never
/// persisted, never sent to clients — the user token stays server-side).
#[derive(Debug, Clone)]
pub struct PendingPick {
    pub platform: String,
    pub token: String,
    pub accounts: Vec<Account>,
    pub created: DateTime<Utc>,
    /// Workspace the connect flow was started from.
    pub workspace_id: String,
}

/// How long a pending page pick stays valid.
pub const PICK_TTL_MIN: i64 = 10;

impl OAuthState {
    pub fn new(platform: &str, workspace_id: &str) -> Self {
        Self {
            platform: platform.to_string(),
            created: Utc::now(),
            workspace_id: workspace_id.to_string(),
        }
    }
}

// ---- handlers ----

/// `GET /api/oauth/{platform}` — tells the frontend which flow to show.
pub async fn status(
    State(s): State<AppState>,
    Path(platform): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let p = provider(&platform).ok_or_else(|| ApiError::not_found("unknown platform"))?;
    let store = s.read().await;
    let configured = store.oauth.configured(p);
    Ok(Json(json!({
        "platform": p.id,
        "mode": if configured { "redirect" } else { "mock" },
        "configured": configured,
        "authorizeHost": Url::parse(p.authorize_url).ok().and_then(|u| u.host_str().map(str::to_string)),
    })))
}

/// `GET /api/oauth/{platform}/start` — begins the flow (creates the CSRF `state`).
/// Gated by `platforms.manage` when `authRequired` is on. The callback cannot be
/// header-gated (it is a browser redirect) — it is protected by the single-use
/// CSPRNG `state`.
pub async fn start(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(platform): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let p = provider(&platform).ok_or_else(|| ApiError::not_found("unknown platform"))?;
    let mut store = s.write().await;
    crate::handlers::require_perm(&mut store, &headers, "platforms.manage")?;
    // The account lands in the workspace that started the flow.
    let workspace_id = crate::handlers::active_workspace_id(&store, &headers)?;
    store.prune_ephemeral();
    let Some(client_id) = store.oauth.credentials(p).map(|(id, _)| id.to_string()) else {
        return Ok(Json(json!({ "mode": "mock" })));
    };
    let state = format!("st-{}", random_hex(16));
    store.oauth_states.insert(
        state.clone(),
        OAuthState {
            platform: p.id.to_string(),
            created: Utc::now(),
            workspace_id,
        },
    );
    let redirect_uri = store.oauth.callback_url(p);
    let mut url =
        Url::parse(p.authorize_url).map_err(|_| ApiError::bad_request("invalid authorize url"))?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("client_id", &client_id);
        q.append_pair("redirect_uri", &redirect_uri);
        q.append_pair("response_type", "code");
        q.append_pair("scope", &p.scopes.join(p.scope_sep));
        q.append_pair("state", &state);
        for (k, v) in p.extra_auth_params {
            q.append_pair(k, v);
        }
    }
    Ok(Json(json!({ "mode": "redirect", "url": url.to_string() })))
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// `GET /api/oauth/{platform}/callback` — provider redirect target.
pub async fn callback(
    State(s): State<AppState>,
    Path(platform): Path<String>,
    AppQuery(q): AppQuery<CallbackQuery>,
) -> Result<Redirect, ApiError> {
    let p = provider(&platform).ok_or_else(|| ApiError::not_found("unknown platform"))?;

    // Validate (and consume) the single-use state under a short lock, then
    // release it before any network I/O.
    let (oauth, workspace_id) = {
        let mut store = s.write().await;
        store.prune_ephemeral();
        let state = q
            .state
            .clone()
            .ok_or_else(|| ApiError::bad_request("missing state"))?;
        let entry = store
            .oauth_states
            .get(&state)
            .cloned()
            .ok_or_else(|| ApiError::bad_request("invalid or expired state"))?;
        if entry.platform != p.id {
            return Err(ApiError::bad_request("state does not match platform"));
        }
        if Utc::now() - entry.created > ChronoDuration::minutes(STATE_TTL_MIN) {
            store.oauth_states.remove(&state);
            return Err(ApiError::bad_request("state expired"));
        }
        store.oauth_states.remove(&state);
        // The workspace may have been deleted mid-flow; fall back to the
        // first live workspace so the connection is not lost.
        let workspace_id = store
            .resolve_workspace_id(Some(&entry.workspace_id), None)
            .unwrap_or_default();
        (store.oauth.clone(), workspace_id)
    };

    enum Outcome {
        Accounts(Vec<Account>, String, Option<i64>),
        MockConnected,
        Failed(String),
    }

    let outcome = if let Some(err) = q.error.clone() {
        Outcome::Failed(q.error_description.clone().unwrap_or(err))
    } else if oauth.configured(p) {
        match exchange_code(p, &oauth, q.code.as_deref().unwrap_or_default()).await {
            Ok((access_token, expires_in)) => {
                match fetch_accounts(p, &oauth, &access_token).await {
                    Ok(accounts) => Outcome::Accounts(accounts, access_token, expires_in),
                    Err(e) => Outcome::Failed(e),
                }
            }
            Err(e) => Outcome::Failed(e),
        }
    } else if q.code.is_some() {
        Outcome::MockConnected
    } else {
        Outcome::Failed("missing authorization code".into())
    };

    // Set when a connection was established and the first live mirror should be
    // pulled once the store lock is released.
    let mut refresh_after: Option<String> = None;
    match outcome {
        Outcome::Accounts(accounts, token, expires_in) => {
            let mut store = s.write().await;
            if store.workspace(&workspace_id).is_none() {
                tracing::warn!(workspace = %workspace_id, "oauth callback for a missing workspace");
                return Ok(redirect_to_frontend(
                    &oauth.frontend_url,
                    &format!(
                        "oauth={}&status=error&reason={}",
                        p.id,
                        url_escape(
                            "this workspace no longer exists — switch to a live workspace and reconnect"
                        )
                    ),
                ));
            }
            if accounts.len() == 1 {
                let mut account = accounts.into_iter().next().unwrap();
                if let Some(secs) = expires_in {
                    account.expires_at = Some(expires_at_from(secs));
                }
                apply_account(&mut store, &workspace_id, p, account);
                // The user token first (ads and other user-level calls), then
                // the Page token for Page content — fetched without the lock.
                store.set_platform_token(&workspace_id, p.id, &token);
                store.set_platform_token(&workspace_id, &format!("{}#user", p.id), &token);
                let page_id = store
                    .workspace(&workspace_id)
                    .and_then(|w| w.connections.iter().find(|c| c.id == p.id))
                    .map(|c| c.external_id.clone())
                    .unwrap_or_default();
                drop(store);
                store_page_token(&s, p, &workspace_id, &token, &page_id).await;
                refresh_after = Some(workspace_id.clone());
            } else {
                // Several connectable accounts: park the token server-side and
                // let the user pick one in the app.
                store.prune_ephemeral();
                let pick = format!("pick-{}", random_hex(16));
                store.oauth_picks.insert(
                    pick.clone(),
                    PendingPick {
                        platform: p.id.to_string(),
                        token,
                        accounts,
                        created: Utc::now(),
                        workspace_id: workspace_id.clone(),
                    },
                );
                return Ok(redirect_to_frontend(
                    &oauth.frontend_url,
                    &format!("oauth={}&status=pick&pick={pick}", p.id),
                ));
            }
        }
        Outcome::MockConnected => {
            let mut store = s.write().await;
            if mark_connected(&mut store, &workspace_id, p.id).is_none() {
                return Err(ApiError::not_found("platform not found"));
            }
        }
        Outcome::Failed(reason) => {
            tracing::warn!(platform = p.id, reason = %reason, "oauth callback failed");
            let message = user_facing_reason(&reason);
            return Ok(redirect_to_frontend(
                &oauth.frontend_url,
                &format!(
                    "oauth={}&status=error&reason={}",
                    p.id,
                    url_escape(&truncate(&message, 300))
                ),
            ));
        }
    }

    if let Some(workspace_id) = refresh_after {
        spawn_live_refresh(&s, p, &workspace_id);
    }
    tracing::info!(platform = p.id, "oauth connection established");
    Ok(redirect_to_frontend(
        &oauth.frontend_url,
        &format!("oauth={}&status=ok", p.id),
    ))
}

/// Pulls the first live mirror in the background so the redirect back to the
/// app is not blocked by Graph calls (they can take seconds).
fn spawn_live_refresh(state: &AppState, provider: &Provider, workspace_id: &str) {
    if provider.id != "meta" {
        return;
    }
    let state = state.clone();
    let workspace_id = workspace_id.to_string();
    tokio::spawn(async move {
        if let Err(error) = crate::live::refresh_workspace(&state, &workspace_id).await {
            tracing::warn!(workspace = %workspace_id, %error, "initial live refresh failed");
        }
    });
}

#[derive(Debug, Deserialize)]
pub struct PendingQuery {
    pub pick: Option<String>,
}

/// `GET /api/oauth/{platform}/pending?pick=…` — lists the connect candidates
/// waiting for a choice. The stored user token is never exposed.
pub async fn pending_pick(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(platform): Path<String>,
    AppQuery(q): AppQuery<PendingQuery>,
) -> Result<Json<Value>, ApiError> {
    let p = provider(&platform).ok_or_else(|| ApiError::not_found("unknown platform"))?;
    let pick = q
        .pick
        .clone()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| ApiError::bad_request("missing pick id"))?;
    let store = s.read().await;
    if !crate::handlers::has_read_access(&store, &headers) {
        return Err(ApiError::unauthorized("authentication required"));
    }
    let Some(entry) = store.oauth_picks.get(&pick) else {
        return Err(ApiError::not_found("unknown or expired pick"));
    };
    if entry.platform != p.id {
        return Err(ApiError::bad_request("pick does not match platform"));
    }
    if Utc::now() - entry.created > ChronoDuration::minutes(PICK_TTL_MIN) {
        return Err(ApiError::bad_request("pick expired — reconnect"));
    }
    Ok(Json(json!({
        "platform": p.id,
        "accounts": entry
            .accounts
            .iter()
            .map(|a| json!({ "handle": a.handle, "external_id": a.external_id }))
            .collect::<Vec<_>>(),
    })))
}

#[derive(Debug, Deserialize)]
pub struct ChooseBody {
    pub pick: String,
    pub external_id: String,
}

/// `POST /api/oauth/{platform}/choose` — finalizes a pending pick by
/// connecting the chosen account.
pub async fn choose_page(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(platform): Path<String>,
    AppJson(body): AppJson<ChooseBody>,
) -> Result<Json<Value>, ApiError> {
    let p = provider(&platform).ok_or_else(|| ApiError::not_found("unknown platform"))?;
    let mut store = s.write().await;
    crate::handlers::require_perm(&mut store, &headers, "platforms.manage")?;
    let entry = store
        .oauth_picks
        .get(&body.pick)
        .cloned()
        .ok_or_else(|| ApiError::not_found("unknown or expired pick"))?;
    if entry.platform != p.id {
        return Err(ApiError::bad_request("pick does not match platform"));
    }
    if Utc::now() - entry.created > ChronoDuration::minutes(PICK_TTL_MIN) {
        store.oauth_picks.remove(&body.pick);
        return Err(ApiError::bad_request("pick expired — reconnect"));
    }
    let Some(account) = entry
        .accounts
        .iter()
        .find(|a| a.external_id == body.external_id)
        .cloned()
    else {
        return Err(ApiError::not_found("page is not part of this pick"));
    };
    let workspace_id = store
        .resolve_workspace_id(Some(&entry.workspace_id), None)
        .ok_or_else(|| ApiError::bad_request("workspace no longer exists — reconnect"))?;
    store.oauth_picks.remove(&body.pick);
    apply_account(&mut store, &workspace_id, p, account);
    // User token under the plain key (fallback for user-level calls) and under
    // `#user`; the Page token replaces the plain key once fetched.
    store.set_platform_token(&workspace_id, p.id, &entry.token);
    store.set_platform_token(&workspace_id, &format!("{}#user", p.id), &entry.token);
    drop(store);
    store_page_token(&s, p, &workspace_id, &entry.token, &body.external_id).await;
    let store = s.read().await;
    let connection = store
        .workspace(&workspace_id)
        .and_then(|w| w.connections.iter().find(|c| c.id == p.id))
        .cloned();
    drop(store);
    spawn_live_refresh(&s, p, &workspace_id);
    tracing::info!(platform = p.id, workspace = %workspace_id, "oauth page pick finalized");
    Ok(Json(json!({ "platform": p.id, "connection": connection })))
}

/// Reads one Meta page-listing edge (`/me/accounts`, `/me/assigned_pages`).
async fn meta_pages(
    oauth: &OauthSettings,
    access_token: &str,
    edge: &str,
    fields: &str,
) -> Result<Vec<Value>, String> {
    let body: Value = oauth
        .client
        .get(format!("{}{edge}", oauth.graph_base))
        .query(&[("fields", fields), ("access_token", access_token)])
        .send()
        .await
        .map_err(|e| format!("account request failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("account response was not JSON: {e}"))?;
    if let Some(err) = body.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown Meta error");
        return Err(format!("meta error: {msg}"));
    }
    Ok(body
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

/// Best-effort diagnosis for an empty `/me/accounts` response: logs which
/// Facebook user authorized and which permissions they actually granted, so
/// the operator can tell "wrong account" apart from "unchecked permission"
/// apart from "no Page". Tokens are never logged.
async fn diagnose_meta_account(oauth: &OauthSettings, access_token: &str) {
    let me: Value = match oauth
        .client
        .get(format!("{}/me", oauth.graph_base))
        .query(&[("fields", "id,name"), ("access_token", access_token)])
        .send()
        .await
    {
        Ok(r) => r.json().await.unwrap_or(Value::Null),
        Err(_) => Value::Null,
    };
    let perms: Value = match oauth
        .client
        .get(format!("{}/me/permissions", oauth.graph_base))
        .query(&[("access_token", access_token)])
        .send()
        .await
    {
        Ok(r) => r.json().await.unwrap_or(Value::Null),
        Err(_) => Value::Null,
    };
    let granted: Vec<&str> = perms
        .get("data")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter(|p| p.get("status").and_then(Value::as_str) == Some("granted"))
                .filter_map(|p| p.get("permission").and_then(Value::as_str))
                .collect()
        })
        .unwrap_or_default();
    let fb_user = me.get("name").and_then(|v| v.as_str()).unwrap_or("?");
    let fb_id = me.get("id").and_then(|v| v.as_str()).unwrap_or("?");
    // Also probe the business fallback so the log shows whether the account has
    // portfolio pages at all (the usual reason for an empty /me/accounts).
    let assigned = meta_pages(oauth, access_token, "/me/assigned_pages", "id,name")
        .await
        .map(|pages| pages.len())
        .unwrap_or(0);
    tracing::warn!(
        fb_user = fb_user,
        fb_id = fb_id,
        granted = ?granted,
        assigned_pages = assigned,
        "meta account returned no accessible pages"
    );
}

/// Turns internal OAuth failure reasons into actionable guidance for the app
/// banner. The raw reason stays in the server logs; this is what the user
/// sees after `Connection failed:`.
fn user_facing_reason(reason: &str) -> String {
    if reason.contains("no Facebook page found") {
        return "no Facebook Page found on this account — create one at facebook.com/pages/create (or sign in with an account that manages one), tick the Page on the Meta consent screen, then reconnect".into();
    }
    reason.to_string()
}

fn redirect_to_frontend(frontend_url: &str, query: &str) -> Redirect {
    Redirect::to(&format!(
        "{}/#/profile?{query}",
        frontend_url.trim_end_matches('/')
    ))
}

fn truncate(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

// ---- shared with the connect endpoint ----

/// Flips a connection to connected with the wireframe defaults (used by the mock flow and by
/// `POST /api/platforms/{id}/connect`).
pub fn mark_connected(
    store: &mut Store,
    workspace_id: &str,
    id: &str,
) -> Option<crate::model::PlatformConnection> {
    let c = store
        .workspace_mut(workspace_id)?
        .connections
        .iter_mut()
        .find(|c| c.id == id)?;
    c.status = "connected".into();
    if c.handle.is_empty() {
        c.handle = if c.id == "meta" {
            "https://www.facebook.com/profile.php?id=100000000000001".into()
        } else {
            "Studio Channel".into()
        };
    }
    if c.external_id.is_empty() {
        c.external_id = format!("ext-{}", uid("acct"));
    }
    if c.scopes.is_empty() {
        c.scopes = vec!["basic".into()];
    }
    c.token_type = if c.id == "youtube" || c.id == "tiktok" {
        "refresh".into()
    } else {
        "long-lived".into()
    };
    c.expires_at = Some(crate::store::plus_days(if c.id == "youtube" {
        0
    } else {
        60
    }));
    c.last_sync = Some(stamp());
    Some(c.clone())
}

// ---- provider calls ----

#[derive(Debug, Default, Clone)]
pub struct Account {
    pub handle: String,
    pub external_id: String,
    pub scopes: Vec<String>,
    pub token_type: String,
    pub expires_at: Option<String>,
}

/// The provider's token endpoint. For Meta this follows `META_GRAPH_BASE` so
/// the whole connect flow can be pointed at a fake Graph in tests.
fn token_endpoint(p: &Provider, oauth: &OauthSettings) -> String {
    if p.cred_key == "meta" {
        format!("{}/oauth/access_token", oauth.graph_base)
    } else {
        p.token_url.to_string()
    }
}

/// Meta: exchange a short-lived user token for a long-lived one (~60 days).
/// Without this the stored token dies in 1-2 hours and every mirror stops.
pub(crate) async fn exchange_long_lived(
    p: &Provider,
    oauth: &OauthSettings,
    short_token: &str,
) -> Result<(String, Option<i64>), String> {
    let (client_id, client_secret) = oauth.credentials(p).ok_or("provider not configured")?;
    let res = oauth
        .http()
        .get(token_endpoint(p, oauth))
        .query(&[
            ("grant_type", "fb_exchange_token"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("fb_exchange_token", short_token),
        ])
        .send()
        .await
        .map_err(|e| format!("long-lived exchange failed: {e}"))?;
    let status = res.status();
    let body: Value = res
        .json()
        .await
        .map_err(|e| format!("long-lived exchange was not JSON ({status}): {e}"))?;
    if let Some(err) = body.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown Meta error");
        return Err(format!("long-lived exchange error: {msg}"));
    }
    let token = body
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("no access_token in long-lived exchange: {body}"))?
        .to_string();
    Ok((token, body.get("expires_in").and_then(Value::as_i64)))
}

/// Meta requires a *Page* access token for Page content (new Pages
/// experience): `/{page-id}/posts`, insights and the linked Instagram
/// account's media all reject a user token. The user token stays stored
/// separately (as `meta#user`) for user-level calls like the ads mirror.
pub async fn fetch_page_token(
    oauth: &OauthSettings,
    user_token: &str,
    page_id: &str,
) -> Result<String, String> {
    if page_id.trim().is_empty() {
        return Err("missing page id".into());
    }
    let body: Value = oauth
        .http()
        .get(format!("{}/{}", oauth.graph_base, page_id))
        .query(&[("fields", "access_token")])
        .query(&[("access_token", user_token)])
        .send()
        .await
        .map_err(|e| format!("page token request failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("page token response was not JSON: {e}"))?;
    if let Some(err) = body.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown Meta error");
        return Err(format!("page token error: {msg}"));
    }
    body.get("access_token")
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|t| !t.is_empty())
        .ok_or_else(|| {
            "Meta did not return a Page access token — make sure the login is an admin of the Page"
                .to_string()
        })
}

async fn exchange_code(
    p: &Provider,
    oauth: &OauthSettings,
    code: &str,
) -> Result<(String, Option<i64>), String> {
    if code.trim().is_empty() {
        return Err("missing authorization code".into());
    }
    let (client_id, client_secret) = oauth.credentials(p).ok_or("provider not configured")?;
    let redirect_uri = oauth.callback_url(p);
    let form: Vec<(&str, &str)> = match p.cred_key {
        "meta" => vec![
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri.as_str()),
            ("code", code),
        ],
        "google" => vec![
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
        ],
        _ => vec![
            ("client_key", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
        ],
    };
    let res = oauth
        .client
        .post(token_endpoint(p, oauth))
        .header("Accept", "application/json")
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("token request failed: {e}"))?;
    let status = res.status();
    let body: Value = res
        .json()
        .await
        .map_err(|e| format!("token response was not JSON ({status}): {e}"))?;
    let short = body
        .get("access_token")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("no access_token in response: {body}"))?;
    let expires = body.get("expires_in").and_then(Value::as_i64);

    // Meta: trade the short-lived token for a long-lived one. Failure is not
    // fatal (the short token still works for a while), but it must be visible.
    if p.cred_key == "meta" {
        match exchange_long_lived(p, oauth, &short).await {
            Ok((long, long_expires)) => return Ok((long, long_expires.or(expires))),
            Err(error) => {
                tracing::warn!(%error, "long-lived token exchange failed; using the short-lived token");
            }
        }
    }
    Ok((short, expires))
}

/// Real expiry date for a token with `expires_in` seconds (Meta's long-lived
/// tokens report ~60 days; the UI must not invent a date).
fn expires_at_from(seconds: i64) -> String {
    let when = Utc::now() + ChronoDuration::seconds(seconds.max(0));
    when.format("%Y-%m-%d").to_string()
}

/// A connect candidate derived from one provider account. For Meta this is one
/// Facebook Page; the Page's public link becomes the handle so the app can link
/// straight to the profile (e.g. `https://www.facebook.com/profile.php?id=…`).
fn meta_page_account(p: &Provider, page: &Value) -> Option<Account> {
    let id = page.get("id").and_then(Value::as_str)?;
    let link = page
        .get("link")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("https://www.facebook.com/profile.php?id={id}"));
    Some(Account {
        handle: link,
        external_id: id.to_string(),
        scopes: p.scopes.iter().map(|s| s.to_string()).collect(),
        token_type: "long-lived".into(),
        expires_at: Some(crate::store::plus_days(60)),
    })
}

async fn fetch_accounts(
    p: &Provider,
    oauth: &OauthSettings,
    access_token: &str,
) -> Result<Vec<Account>, String> {
    match p.id {
        "meta" => {
            let fields = "id,name,link,access_token";
            let mut pages = meta_pages(oauth, access_token, "/me/accounts", fields).await?;
            if pages.is_empty() {
                // Pages that live in a Business portfolio are omitted from
                // `/me/accounts`; `/me/assigned_pages` returns them through
                // task-based access (needs the business_management scope).
                pages = match meta_pages(oauth, access_token, "/me/assigned_pages", fields).await {
                    Ok(pages) => {
                        if !pages.is_empty() {
                            tracing::info!(
                                count = pages.len(),
                                "meta pages resolved via assigned_pages fallback"
                            );
                        }
                        pages
                    }
                    Err(e) => {
                        tracing::info!(
                            edge = "/me/assigned_pages",
                            error = %e,
                            "assigned_pages fallback unavailable"
                        );
                        Vec::new()
                    }
                };
            }
            if pages.is_empty() {
                diagnose_meta_account(oauth, access_token).await;
                return Err("no Facebook page found for this account".into());
            }
            let accounts: Vec<Account> = pages
                .iter()
                .filter_map(|page| meta_page_account(p, page))
                .collect();
            if accounts.is_empty() {
                return Err("no Facebook page found for this account".into());
            }
            Ok(accounts)
        }
        "youtube" => {
            let body: Value = oauth
                .client
                .get("https://www.googleapis.com/youtube/v3/channels")
                .query(&[("part", "snippet"), ("mine", "true")])
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| format!("account request failed: {e}"))?
                .json()
                .await
                .map_err(|e| format!("account response was not JSON: {e}"))?;
            let item = body
                .get("items")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .ok_or("no YouTube channel on this account")?;
            Ok(vec![Account {
                handle: item
                    .get("snippet")
                    .and_then(|s| s.get("title"))
                    .and_then(Value::as_str)
                    .unwrap_or("YouTube Channel")
                    .to_string(),
                external_id: item
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                scopes: p.scopes.iter().map(|s| s.to_string()).collect(),
                token_type: "refresh".into(),
                expires_at: None,
            }])
        }
        _ => {
            let body: Value = oauth
                .client
                .get("https://open.tiktokapis.com/v2/user/info/")
                .query(&[("fields", "open_id,display_name,username")])
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| format!("account request failed: {e}"))?
                .json()
                .await
                .map_err(|e| format!("account response was not JSON: {e}"))?;
            let user = body
                .get("data")
                .and_then(|d| d.get("user"))
                .ok_or("no TikTok user in response")?;
            let username = user.get("username").and_then(Value::as_str).unwrap_or("");
            let display = user
                .get("display_name")
                .and_then(Value::as_str)
                .unwrap_or("");
            Ok(vec![Account {
                handle: if !username.is_empty() {
                    format!("@{username}")
                } else if !display.is_empty() {
                    display.to_string()
                } else {
                    "TikTok account".into()
                },
                external_id: user
                    .get("open_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                scopes: p.scopes.iter().map(|s| s.to_string()).collect(),
                token_type: "refresh".into(),
                expires_at: Some(crate::store::plus_days(365)),
            }])
        }
    }
}

/// Fetches the Page access token (Meta) and stores it under the plain platform
/// key; the user token stays under `#user`. Failures are logged, not fatal:
/// the live refresh will surface the reason and the user token still covers
/// user-level calls (the ads mirror).
async fn store_page_token(
    state: &AppState,
    p: &Provider,
    workspace_id: &str,
    user_token: &str,
    page_id: &str,
) {
    if p.cred_key != "meta" {
        return;
    }
    let oauth = { state.read().await.oauth.clone() };
    match fetch_page_token(&oauth, user_token, page_id).await {
        Ok(page_token) => {
            let mut store = state.write().await;
            store.set_platform_token(workspace_id, p.id, &page_token);
            tracing::info!(workspace = %workspace_id, page = %page_id, "page access token stored");
        }
        Err(error) => {
            tracing::warn!(workspace = %workspace_id, page = %page_id, %error, "page access token fetch failed");
        }
    }
}

fn apply_account(store: &mut Store, workspace_id: &str, p: &Provider, account: Account) {
    if let Some(c) = store
        .workspace_mut(workspace_id)
        .and_then(|w| w.connections.iter_mut().find(|c| c.id == p.id))
    {
        c.status = "connected".into();
        c.handle = account.handle;
        c.external_id = account.external_id;
        c.scopes = account.scopes;
        c.token_type = account.token_type;
        c.expires_at = account.expires_at;
        c.last_sync = Some(stamp());
    }
}

fn url_escape(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fake Graph server for the connect path: `/me/accounts` (or the
    /// assigned-pages fallback) with a Page that has a profile `link`.
    async fn fake_graph(accounts_empty: bool) -> String {
        use axum::{routing::get, Json, Router};
        let page = json!({
            "id": "61592348575800",
            "name": "Artibition",
            "link": "https://www.facebook.com/profile.php?id=61592348575800",
            "access_token": "page-token"
        });
        let accounts_page = page.clone();
        let assigned_page = page.clone();
        let app = Router::new()
            .route(
                "/me/accounts",
                get(move || {
                    let page = accounts_page.clone();
                    async move {
                        if accounts_empty {
                            Json(json!({ "data": [] }))
                        } else {
                            Json(json!({ "data": [page] }))
                        }
                    }
                }),
            )
            .route(
                "/me/assigned_pages",
                get(move || {
                    let page = assigned_page.clone();
                    async move { Json(json!({ "data": [page] })) }
                }),
            )
            .route(
                "/me",
                get(|| async { Json(json!({ "id": "1", "name": "Tester" })) }),
            )
            .route(
                "/me/permissions",
                get(|| async { Json(json!({ "data": [] })) }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn fetch_accounts_uses_the_page_profile_link_as_handle() {
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&fake_graph(false).await);
        let p = provider("meta").unwrap();
        let accounts = fetch_accounts(p, &oauth, "user-token").await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(
            accounts[0].handle, "https://www.facebook.com/profile.php?id=61592348575800",
            "the Page profile URL is the linkable handle"
        );
        assert_eq!(accounts[0].external_id, "61592348575800");
        assert!(accounts[0]
            .scopes
            .iter()
            .any(|s| s == "business_management"));
    }

    #[tokio::test]
    async fn fetch_accounts_falls_back_to_assigned_pages() {
        // Business-portfolio Pages are missing from /me/accounts.
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&fake_graph(true).await);
        let p = provider("meta").unwrap();
        let accounts = fetch_accounts(p, &oauth, "user-token").await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].external_id, "61592348575800");
    }

    #[tokio::test]
    async fn fetch_accounts_without_pages_is_actionable() {
        use axum::{routing::get, Json, Router};
        let app = Router::new()
            .route(
                "/me/accounts",
                get(|| async { Json(json!({ "data": [] })) }),
            )
            .route(
                "/me/assigned_pages",
                get(|| async { Json(json!({ "data": [] })) }),
            )
            .route(
                "/me",
                get(|| async { Json(json!({ "id": "1", "name": "Tester" })) }),
            )
            .route(
                "/me/permissions",
                get(|| async { Json(json!({ "data": [] })) }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&format!("http://{addr}"));
        let p = provider("meta").unwrap();
        let error = fetch_accounts(p, &oauth, "user-token").await.unwrap_err();
        assert!(error.contains("no Facebook page found"), "{error}");
        assert!(user_facing_reason(&error).contains("facebook.com/pages/create"));
    }

    /// Fake Graph for the token dance: POST /oauth/access_token returns a
    /// short-lived token, GET with `fb_exchange_token` returns the long-lived
    /// one, and `/{page}?fields=access_token` returns the Page token.
    async fn fake_token_graph() -> String {
        use axum::{routing::get, routing::post, Json, Router};
        let app = Router::new()
            .route(
                "/oauth/access_token",
                post(|| async {
                    Json(json!({ "access_token": "short-user-token", "expires_in": 3600 }))
                })
                .get(|query: axum::extract::Query<std::collections::HashMap<String, String>>| async move {
                    if query.get("grant_type").map(String::as_str) == Some("fb_exchange_token") {
                        Json(json!({ "access_token": "long-user-token", "expires_in": 5183944 }))
                    } else {
                        Json(json!({ "error": { "message": "unknown grant" } }))
                    }
                }),
            )
            .route(
                "/61592348575800",
                get(|| async { Json(json!({ "access_token": "page-token-xyz" })) }),
            )
            .route(
                "/999",
                get(|| async { Json(json!({ "error": { "message": "no access" } })) }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn exchange_code_trades_the_short_token_for_a_long_lived_one() {
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&fake_token_graph().await);
        oauth.set_credentials("meta", "app-id", "app-secret");
        let p = provider("meta").unwrap();
        let (token, expires) = exchange_code(p, &oauth, "the-code").await.unwrap();
        assert_eq!(token, "long-user-token");
        assert_eq!(expires, Some(5_183_944), "long-lived expiry from Meta");
    }

    #[tokio::test]
    async fn fetch_page_token_reads_the_page_access_token() {
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&fake_token_graph().await);
        let token = fetch_page_token(&oauth, "user-token", "61592348575800")
            .await
            .unwrap();
        assert_eq!(token, "page-token-xyz");

        let error = fetch_page_token(&oauth, "user-token", "999")
            .await
            .unwrap_err();
        assert!(error.contains("no access"), "{error}");
    }

    #[tokio::test]
    async fn store_page_token_keeps_the_user_token_for_ads() {
        let state = crate::new_state();
        {
            let mut store = state.write().await;
            store.oauth.set_graph_base(&fake_token_graph().await);
            store.enable_token_vault("unit-test-admin-secret-0123456789");
            store.set_platform_token(
                crate::store::DEFAULT_WORKSPACE_ID,
                "meta",
                "long-user-token",
            );
            store.set_platform_token(
                crate::store::DEFAULT_WORKSPACE_ID,
                "meta#user",
                "long-user-token",
            );
        }
        let p = provider("meta").unwrap();
        store_page_token(
            &state,
            p,
            crate::store::DEFAULT_WORKSPACE_ID,
            "long-user-token",
            "61592348575800",
        )
        .await;
        let mut store = state.write().await;
        assert_eq!(
            store
                .platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta")
                .as_deref(),
            Some("page-token-xyz"),
            "Page token replaces the plain key"
        );
        assert_eq!(
            store
                .platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta#user")
                .as_deref(),
            Some("long-user-token"),
            "user token stays for the ads mirror"
        );
        // Disconnect drops both copies.
        store.clear_platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta");
        assert!(store
            .platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta")
            .is_none());
        assert!(store
            .platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta#user")
            .is_none());
    }

    #[test]
    fn failure_reasons_guide_the_user() {
        let page = user_facing_reason("no Facebook page found for this account");
        assert!(page.contains("facebook.com/pages/create"), "{page}");
        assert!(page.contains("reconnect"), "{page}");

        // unknown provider errors pass through unchanged
        let other = user_facing_reason("token request failed: boom");
        assert_eq!(other, "token request failed: boom");
    }
}
