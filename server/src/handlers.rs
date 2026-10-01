//! HTTP handlers. Each one locks the in-memory store, mutates or reads it, and
//! returns JSON with camelCase fields (matching the frontend types).
//!
//! Security posture: mutations are permission-gated (when `authRequired` is
//! on), passwords are hashed/verified off the async runtime, login attempts are
//! rate-limited, and every field is validated before it reaches the store.
use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{ApiError, AppJson, AppQuery};
use crate::model::*;
use crate::store::{
    hash_password, session_key, stamp, uid, verify_password, Account, Store, PASSWORD_MAX_CHARS,
    PASSWORD_MIN_CHARS,
};
use crate::AppState;

/// Short free-text field limit.
const MAX_TEXT: usize = 200;
/// Long free-text (captions, notes, ideas) limit.
const MAX_LONG_TEXT: usize = 5000;
/// List field limits.
const MAX_LIST_ITEMS: usize = 100;
const MAX_LIST_ITEM: usize = 200;

#[derive(Deserialize)]
pub struct MonthQuery {
    pub month: Option<u32>,
}

#[derive(Deserialize)]
pub struct AddOptionBody {
    pub list: String,
    pub item: String,
}

#[derive(Deserialize)]
pub struct AddTagBody {
    pub tag: String,
}

#[derive(Deserialize)]
pub struct UserBody {
    pub name: String,
    #[serde(default)]
    pub role: String,
}

#[derive(Deserialize)]
pub struct RoleBody {
    pub name: String,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
pub struct RegisterBody {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordBody {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct UpdateProfileBody {
    pub name: String,
}

#[derive(Deserialize)]
pub struct LockBody {
    #[serde(default)]
    pub user: String,
}

#[derive(Deserialize)]
pub struct ImportBody {
    pub platform: String,
    pub month: u32,
}

// ---- shared helpers ----

/// Extracts the token from an `Authorization: Bearer <token>` header.
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Resolves a bearer token to a live session's account email, removing the
/// session from the store when it has expired.
fn require_session(store: &mut Store, headers: &HeaderMap) -> Result<String, ApiError> {
    let token =
        bearer_token(headers).ok_or_else(|| ApiError::unauthorized("missing bearer token"))?;
    let session = store
        .session(token)
        .cloned()
        .ok_or_else(|| ApiError::unauthorized("invalid token"))?;
    if session.expires <= Utc::now() {
        store.revoke_session(token);
        return Err(ApiError::unauthorized("invalid token"));
    }
    Ok(session.email)
}

/// The display name of the account behind an email.
fn account_name(store: &Store, email: &str) -> Option<String> {
    store.accounts.get(email).map(|a| a.name.clone())
}

/// The wire-facing user for a name: its directory entry, or a Viewer fallback
/// for accounts whose directory user was removed.
fn user_view(store: &Store, name: &str) -> User {
    store
        .setup
        .users
        .iter()
        .find(|u| u.name == name)
        .cloned()
        .unwrap_or(User {
            name: name.to_string(),
            role: "Viewer".into(),
        })
}

/// Wire-facing user JSON with the account's SaaS plan attached. Auth responses
/// use this so the UI can show the current package.
fn user_json(store: &Store, email: &str) -> Option<serde_json::Value> {
    let name = account_name(store, email)?;
    let mut value = serde_json::to_value(user_view(store, &name)).ok()?;
    if let (Some(obj), Some(account)) = (value.as_object_mut(), store.accounts.get(email)) {
        obj.insert("plan".to_string(), serde_json::json!(account.plan));
    }
    Some(value)
}

fn role_has(setup: &SetupConfig, role: &str, perm: &str) -> bool {
    setup
        .roles
        .iter()
        .find(|r| r.name == role)
        .map(|r| r.permissions.iter().any(|p| p == perm))
        .unwrap_or(false)
}

/// Constant-time string comparison (for the operator recovery token).
fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Read-access check shared by the read-auth middleware and `get_setup`.
/// While `authRequired` is off (demo mode) everything is readable; otherwise a
/// live bearer token — or the operator recovery token — is required.
pub(crate) fn has_read_access(store: &Store, headers: &HeaderMap) -> bool {
    if let Some(expected) = store.admin_token.as_deref() {
        if let Some(provided) = headers.get("x-admin-token").and_then(|v| v.to_str().ok()) {
            if constant_time_eq(provided, expected) {
                return true;
            }
        }
    }
    if !store.setup.auth_required {
        return true;
    }
    bearer_token(headers)
        .and_then(|token| store.session(token))
        .is_some_and(|session| session.expires > Utc::now())
}

/// Authorization gate for mutating handlers. While `authRequired` is off (demo
/// mode) everything is allowed and `None` is returned; otherwise the bearer
/// token's account must resolve to a directory role carrying `perm` —
/// missing/expired/unknown token → 401, role without the permission → 403, and
/// the account email is returned so handlers can derive the actor's name
/// instead of trusting client input. A valid `X-Admin-Token` (env
/// `ADMIN_TOKEN`) bypasses role checks as a break-glass recovery path.
/// Email of the signed-in account while auth is on. `None` in demo mode and
/// for the operator recovery token — both of which see every workspace.
pub(crate) fn current_account(store: &Store, headers: &HeaderMap) -> Option<String> {
    if !store.setup.auth_required {
        return None;
    }
    bearer_token(headers)
        .and_then(|token| store.session(token))
        .filter(|session| session.expires > Utc::now())
        .map(|session| session.email.clone())
}

/// Resolves the workspace a request targets: the `X-Workspace-Id` header when
/// it names a workspace the account may use, otherwise the account's first
/// workspace. Mutating handlers resolve the id before taking a mutable borrow.
pub(crate) fn active_workspace_id(store: &Store, headers: &HeaderMap) -> Result<String, ApiError> {
    let requested = headers.get("x-workspace-id").and_then(|v| v.to_str().ok());
    let account = current_account(store, headers);
    store
        .resolve_workspace_id(requested, account.as_deref())
        .ok_or_else(|| ApiError::bad_request("no workspace exists — create one first"))
}

pub(crate) fn require_perm(
    store: &mut Store,
    headers: &HeaderMap,
    perm: &str,
) -> Result<Option<String>, ApiError> {
    if let Some(expected) = store.admin_token.clone() {
        if let Some(provided) = headers.get("x-admin-token").and_then(|v| v.to_str().ok()) {
            if constant_time_eq(provided, &expected) {
                tracing::warn!(permission = perm, "admin recovery token used");
                return Ok(None);
            }
        }
    }
    if !store.setup.auth_required {
        return Ok(None);
    }
    let email = require_session(store, headers)?;
    let allowed = account_name(store, &email)
        .and_then(|name| {
            store
                .setup
                .users
                .iter()
                .find(|u| u.name == name)
                .and_then(|u| store.setup.roles.iter().find(|r| r.name == u.role))
                .map(|r| r.permissions.iter().any(|p| p == perm))
        })
        .unwrap_or(false);
    if allowed {
        Ok(Some(email))
    } else {
        tracing::warn!(permission = perm, "permission denied");
        Err(ApiError::forbidden(&format!("missing permission: {perm}")))
    }
}

fn validate_roles(roles: &[Role]) -> Result<(), ApiError> {
    for role in roles {
        if role.name.trim().is_empty() {
            return Err(ApiError::bad_request("role name is required"));
        }
        validate_len("role name", &role.name, MAX_TEXT)?;
        if role.permissions.len() > KNOWN_PERMISSIONS.len() {
            return Err(ApiError::bad_request("too many permissions"));
        }
        for perm in &role.permissions {
            if !KNOWN_PERMISSIONS.contains(&perm.as_str()) {
                return Err(ApiError::bad_request(&format!(
                    "unknown permission '{perm}'"
                )));
            }
        }
    }
    Ok(())
}

/// Guards the full-roles replacement on `PATCH /api/setup` with the same rules
/// as `POST/DELETE /api/setup/roles`: real names, known permissions, no
/// duplicates, at least one role, and every role a directory user holds must
/// survive the replacement.
fn validate_role_set(store: &Store, roles: &[Role]) -> Result<(), ApiError> {
    validate_roles(roles)?;
    if roles.is_empty() {
        return Err(ApiError::bad_request("at least one role is required"));
    }
    for (i, role) in roles.iter().enumerate() {
        if roles[i + 1..].iter().any(|r| r.name == role.name) {
            return Err(ApiError::bad_request(&format!(
                "duplicate role name: {}",
                role.name
            )));
        }
    }
    for user in &store.setup.users {
        if !roles.iter().any(|r| r.name == user.role) {
            return Err(ApiError::bad_request(&format!(
                "role is in use: {}",
                user.role
            )));
        }
    }
    Ok(())
}

fn validate_len(field: &str, value: &str, max: usize) -> Result<(), ApiError> {
    if value.chars().count() > max {
        return Err(ApiError::bad_request(&format!(
            "{field} must be at most {max} characters"
        )));
    }
    Ok(())
}

fn validate_month(month: u32) -> Result<(), ApiError> {
    if !(1..=12).contains(&month) {
        return Err(ApiError::bad_request("month must be between 1 and 12"));
    }
    Ok(())
}

fn validate_date(field: &str, value: &str) -> Result<(), ApiError> {
    if value.is_empty() {
        return Ok(());
    }
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| ApiError::bad_request(&format!("{field} must be a YYYY-MM-DD date")))
}

fn validate_time(field: &str, value: &str) -> Result<(), ApiError> {
    if value.is_empty() {
        return Ok(());
    }
    chrono::NaiveTime::parse_from_str(value, "%H:%M")
        .map(|_| ())
        .map_err(|_| ApiError::bad_request(&format!("{field} must be a HH:MM time")))
}

fn validate_list(field: &str, values: &[String]) -> Result<(), ApiError> {
    if values.len() > MAX_LIST_ITEMS {
        return Err(ApiError::bad_request(&format!(
            "{field} must have at most {MAX_LIST_ITEMS} items"
        )));
    }
    for value in values {
        validate_len(field, value, MAX_LIST_ITEM)?;
    }
    Ok(())
}

fn validate_post(post: &Post, setup: &SetupConfig) -> Result<(), ApiError> {
    validate_month(post.month)?;
    if post.topic.trim().is_empty() {
        return Err(ApiError::bad_request("topic is required"));
    }
    validate_len("topic", &post.topic, MAX_TEXT)?;
    validate_len("pillar", &post.pillar, MAX_TEXT)?;
    validate_len("format", &post.format, MAX_TEXT)?;
    validate_len("goal", &post.goal, MAX_TEXT)?;
    validate_len("hook", &post.hook, MAX_LONG_TEXT)?;
    validate_len("caption", &post.caption, MAX_LONG_TEXT)?;
    validate_len("cta", &post.cta, MAX_TEXT)?;
    validate_len("hashtagGroup", &post.hashtag_group, MAX_TEXT)?;
    validate_len("note", &post.note, MAX_LONG_TEXT)?;
    validate_len("imageUrl", &post.image_url, 2000)?;
    if let Some(date) = &post.date {
        validate_date("date", date)?;
    }
    validate_time("time", &post.time)?;
    if !post.status.trim().is_empty()
        && !setup.statuses.is_empty()
        && !setup.statuses.iter().any(|s| s == &post.status)
    {
        return Err(ApiError::bad_request(&format!(
            "unknown status '{}'",
            post.status
        )));
    }
    validate_list("hashtags", &post.hashtags)?;
    validate_list("platforms", &post.platforms)?;
    for platform in &post.platforms {
        if !setup.platforms.is_empty() && !setup.platforms.iter().any(|p| p == platform) {
            return Err(ApiError::bad_request(&format!(
                "unknown platform '{platform}'"
            )));
        }
    }
    Ok(())
}

// ---- health ----

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "service": "content-planner-server" }))
}

/// Readiness: the store is reachable and serving.
pub async fn ready(State(s): State<AppState>) -> Json<serde_json::Value> {
    let store = s.read().await;
    Json(json!({
        "status": "ready",
        "service": "content-planner-server",
        "version": env!("CARGO_PKG_VERSION"),
        "demoMode": store.demo_mode,
        "authRequired": store.setup.auth_required,
        "static": crate::static_dir().is_some(),
    }))
}

/// Opening http://localhost:8787 in a browser says hello.
pub async fn root() -> &'static str {
    "Server on"
}

// ---- API documentation (Swagger UI, assets vendored under server/swagger/) ----

/// Interactive API documentation.
pub async fn api_docs() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("../swagger/index.html"),
    )
}

/// Swagger UI stylesheet.
pub async fn api_docs_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../swagger/swagger-ui.css"),
    )
}

/// Swagger UI bundle.
pub async fn api_docs_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../swagger/swagger-ui-bundle.js"),
    )
}

/// Swagger UI bootstrap (points the UI at `/api/openapi.yaml`).
pub async fn api_docs_init() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../swagger/swagger-init.js"),
    )
}

/// The OpenAPI 3.0 document that describes this API.
pub async fn openapi_spec() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/yaml; charset=utf-8")],
        include_str!("../openapi.yaml"),
    )
}

/// JSON 404 for unmatched routes.
pub async fn not_found() -> ApiError {
    ApiError::not_found("route not found")
}

// ---- setup ----

pub async fn get_setup(State(s): State<AppState>, headers: HeaderMap) -> Json<SetupConfig> {
    let store = s.read().await;
    let mut setup = store.setup.clone();
    let authorized = has_read_access(&store, &headers);
    // The navbar label is the active workspace's name, so the settings field
    // always shows the workspace the user is actually editing. Anonymous
    // callers only get the config they need for the sign-in screen, so their
    // workspace name stays the legacy default.
    if authorized {
        let requested = headers.get("x-workspace-id").and_then(|v| v.to_str().ok());
        let account = current_account(&store, &headers);
        if let Some(id) = store.resolve_workspace_id(requested, account.as_deref()) {
            if let Some(ws) = store.workspace(&id) {
                setup.workspace_name = ws.name.clone();
            }
        }
    }
    if !authorized {
        // Pre-login callers get the workspace/branding config they need to
        // render the sign-in screen — never the staff directory or role matrix.
        setup.users.clear();
        setup.roles.clear();
    }
    Json(setup)
}

pub async fn patch_setup(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(patch): AppJson<SetupPatch>,
) -> Result<Json<SetupConfig>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "setup.write")?;
    // Stage every change and validate before committing, so a rejected patch
    // can never leave a partially-applied configuration behind.
    let mut next = store.setup.clone();
    if let Some(v) = patch.language {
        let v = v.trim().to_uppercase();
        if v != "EN" {
            return Err(ApiError::bad_request("language must be EN"));
        }
        next.language = v;
    }
    if let Some(v) = patch.year {
        if !(1970..=9999).contains(&v) {
            return Err(ApiError::bad_request("year must be between 1970 and 9999"));
        }
        next.year = v;
    }
    if let Some(v) = patch.owner {
        let v = v.trim().to_string();
        if v.is_empty() {
            return Err(ApiError::bad_request("owner is required"));
        }
        validate_len("owner", &v, MAX_TEXT)?;
        next.owner = v;
    }
    let mut workspace_rename: Option<String> = None;
    if let Some(v) = patch.workspace_name {
        let v = v.trim().to_string();
        if v.is_empty() {
            return Err(ApiError::bad_request("workspaceName is required"));
        }
        validate_len("workspaceName", &v, 80)?;
        next.workspace_name = v.clone();
        workspace_rename = Some(v);
    }
    if let Some(v) = patch.show_editable_colors {
        next.show_editable_colors = v;
    }
    if let Some(v) = patch.auth_required {
        next.auth_required = v;
    }
    if let Some(roles) = patch.roles {
        validate_role_set(&store, &roles)?;
        next.roles = roles;
    }
    store.setup = next;
    // Renaming the "workspace" in Settings renames the active workspace, so
    // the navbar label and the settings field always agree.
    if let Some(name) = workspace_rename {
        if let Ok(id) = active_workspace_id(&store, &headers) {
            if let Some(ws) = store.workspace_mut(&id) {
                ws.name = name;
            }
        }
    }
    Ok(Json(store.setup.clone()))
}

pub async fn add_option(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<AddOptionBody>,
) -> Result<Json<SetupConfig>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "setup.write")?;
    let item = body.item.trim().to_string();
    validate_len("option", &item, MAX_LIST_ITEM)?;
    {
        let list = match body.list.as_str() {
            "pillars" => &mut store.setup.pillars,
            "formats" => &mut store.setup.formats,
            "goals" => &mut store.setup.goals,
            "statuses" => &mut store.setup.statuses,
            "platforms" => &mut store.setup.platforms,
            other => {
                return Err(ApiError::bad_request(&format!(
                    "unknown option list '{other}'"
                )))
            }
        };
        if !item.is_empty() && !list.contains(&item) {
            list.push(item);
        }
    }
    Ok(Json(store.setup.clone()))
}

pub async fn add_user(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<UserBody>,
) -> Result<Json<SetupConfig>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "users.manage")?;
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::bad_request("name is required"));
    }
    validate_len("name", &name, MAX_TEXT)?;
    if store
        .setup
        .users
        .iter()
        .any(|u| u.name.eq_ignore_ascii_case(&name))
    {
        return Err(ApiError::bad_request("user already exists"));
    }
    let role = if body.role.trim().is_empty() {
        "Editor".to_string()
    } else {
        body.role.trim().to_string()
    };
    if !store.setup.roles.iter().any(|r| r.name == role) {
        return Err(ApiError::bad_request(&format!("unknown role '{role}'")));
    }
    store.setup.users.push(User { name, role });
    Ok(Json(store.setup.clone()))
}

pub async fn remove_user(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<SetupConfig>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "users.manage")?;
    let name = name.trim().to_string();
    if !store.setup.users.iter().any(|u| u.name == name) {
        return Err(ApiError::not_found("user not found"));
    }
    if store.setup.users.len() <= 1 {
        return Err(ApiError::bad_request("cannot remove the last user"));
    }
    if store.setup.auth_required {
        let target_admin = store
            .setup
            .users
            .iter()
            .find(|u| u.name == name)
            .map(|u| role_has(&store.setup, &u.role, "users.manage"))
            .unwrap_or(false);
        let admins = store
            .setup
            .users
            .iter()
            .filter(|u| role_has(&store.setup, &u.role, "users.manage"))
            .count();
        if target_admin && admins <= 1 {
            return Err(ApiError::bad_request(
                "cannot remove the last administrator",
            ));
        }
    }
    store.setup.users.retain(|u| u.name != name);
    Ok(Json(store.setup.clone()))
}

pub async fn add_role(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<RoleBody>,
) -> Result<Json<SetupConfig>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "users.manage")?;
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::bad_request("role name is required"));
    }
    if store.setup.roles.iter().any(|r| r.name == name) {
        return Err(ApiError::bad_request("role already exists"));
    }
    let role = Role {
        name,
        permissions: body.permissions,
    };
    validate_roles(std::slice::from_ref(&role))?;
    store.setup.roles.push(role);
    Ok(Json(store.setup.clone()))
}

pub async fn remove_role(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<SetupConfig>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "users.manage")?;
    let name = name.trim().to_string();
    if !store.setup.roles.iter().any(|r| r.name == name) {
        return Err(ApiError::not_found("role not found"));
    }
    if store.setup.roles.len() <= 1 {
        return Err(ApiError::bad_request("cannot remove the last role"));
    }
    if store.setup.users.iter().any(|u| u.role == name) {
        return Err(ApiError::bad_request("role is assigned to users"));
    }
    store.setup.roles.retain(|r| r.name != name);
    Ok(Json(store.setup.clone()))
}

// ---- auth (email + password, Argon2id, random expiring sessions) ----

fn validate_password(field: &str, password: &str) -> Result<(), ApiError> {
    let chars = password.chars().count();
    if chars < PASSWORD_MIN_CHARS {
        return Err(ApiError::bad_request(&format!(
            "{field} must be at least {PASSWORD_MIN_CHARS} characters"
        )));
    }
    if chars > PASSWORD_MAX_CHARS {
        return Err(ApiError::bad_request(&format!(
            "{field} must be at most {PASSWORD_MAX_CHARS} characters"
        )));
    }
    Ok(())
}

pub async fn register(
    State(s): State<AppState>,
    AppJson(body): AppJson<RegisterBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::bad_request("name is required"));
    }
    validate_len("name", &name, MAX_TEXT)?;
    let email = body.email.trim().to_lowercase();
    if email.len() > 254 || !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(ApiError::bad_request("a valid email is required"));
    }
    validate_password("password", &body.password)?;

    {
        let store = s.read().await;
        if !store.allow_registration {
            return Err(ApiError::forbidden("registration is disabled"));
        }
        if store.accounts.contains_key(&email) {
            return Err(ApiError::conflict("email already registered"));
        }
        if store
            .accounts
            .values()
            .any(|a| a.name.eq_ignore_ascii_case(&name))
        {
            return Err(ApiError::conflict("name already taken"));
        }
    }

    // Hash off the async runtime, then re-check (another request may have won
    // the race while hashing).
    let password = body.password.clone();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|_| ApiError::internal("password hashing failed"))?;

    let mut store = s.write().await;
    if store.accounts.contains_key(&email) {
        return Err(ApiError::conflict("email already registered"));
    }
    if store
        .accounts
        .values()
        .any(|a| a.name.eq_ignore_ascii_case(&name))
    {
        return Err(ApiError::conflict("name already taken"));
    }
    store.accounts.insert(
        email.clone(),
        Account {
            name: name.clone(),
            email: email.clone(),
            password_hash,
            plan: "free".into(),
            last_active: None,
        },
    );
    if !store
        .setup
        .users
        .iter()
        .any(|u| u.name.eq_ignore_ascii_case(&name))
    {
        // Self-serve signup never grants elevated access: a new account is a
        // read-only Viewer until an Owner assigns a role. Registering with the
        // name of an existing directory entry (an invite) keeps that role —
        // that path is handled above and no user is pushed here.
        store.setup.users.push(User {
            name: name.clone(),
            role: "Viewer".into(),
        });
    }
    let token = store.open_session(&email);
    tracing::info!(email = %email, "account registered");
    Ok(Json(
        json!({ "token": token, "user": user_json(&store, &email) }),
    ))
}

pub async fn login(
    State(s): State<AppState>,
    AppJson(body): AppJson<LoginBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let email = body.email.trim().to_lowercase();
    if email.len() > 254 || !email.contains('@') {
        return Err(ApiError::unauthorized("invalid email or password"));
    }

    let (hash, known) = {
        let store = s.read().await;
        if let Some(retry) = store.login_retry_after(&email) {
            return Err(ApiError::too_many_requests(&format!(
                "too many failed attempts — try again in {retry} seconds"
            )));
        }
        match store.accounts.get(&email) {
            Some(account) => (account.password_hash.clone(), true),
            // Verify against a dummy hash so response time does not reveal
            // whether the account exists.
            None => (crate::store::dummy_password_hash().to_string(), false),
        }
    };

    let password = body.password;
    let verified = tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .map_err(|_| ApiError::internal("password verification failed"))?;

    if !known || !verified {
        let mut store = s.write().await;
        store.record_login_failure(&email);
        tracing::warn!(email = %email, "failed login");
        return Err(ApiError::unauthorized("invalid email or password"));
    }

    let mut store = s.write().await;
    store.clear_login_failures(&email);
    if !store.accounts.contains_key(&email) {
        return Err(ApiError::unauthorized("invalid email or password"));
    }
    let token = store.open_session(&email);
    tracing::info!(email = %email, "login");
    Ok(Json(
        json!({ "token": token, "user": user_json(&store, &email) }),
    ))
}

pub async fn me(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    let email = require_session(&mut store, &headers)?;
    let user = user_json(&store, &email).ok_or_else(|| ApiError::unauthorized("invalid token"))?;
    Ok(Json(json!({ "user": user })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanBody {
    pub plan: String,
}

/// Selects the SaaS package for the signed-in account (free/pro/business).
pub async fn set_plan(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<PlanBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let plan = body.plan.trim().to_lowercase();
    const PLANS: [&str; 3] = ["free", "pro", "business"];
    if !PLANS.contains(&plan.as_str()) {
        return Err(ApiError::bad_request("unknown plan"));
    }
    let mut store = s.write().await;
    let email = require_session(&mut store, &headers)?;
    if let Some(account) = store.accounts.get_mut(&email) {
        account.plan = plan.clone();
    }
    let user = user_json(&store, &email).ok_or_else(|| ApiError::unauthorized("invalid token"))?;
    tracing::info!(email = %email, plan = %plan, "plan selected");
    Ok(Json(json!({ "user": user })))
}

pub async fn logout(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    let token = bearer_token(&headers)
        .ok_or_else(|| ApiError::unauthorized("missing bearer token"))?
        .to_string();
    require_session(&mut store, &headers)?;
    store.revoke_session(&token);
    Ok(Json(json!({ "ok": true })))
}

/// Revokes every session of the caller's account, including the current one.
pub async fn logout_all(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    let email = require_session(&mut store, &headers)?;
    store.sessions.retain(|_, session| session.email != email);
    Ok(Json(json!({ "ok": true })))
}

/// Lists registered accounts for the Setup screen (no hashes, ever).
pub async fn list_accounts(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "users.manage")?;
    let accounts: Vec<(String, String)> = store
        .accounts
        .values()
        .map(|a| (a.email.clone(), a.name.clone()))
        .collect();
    let mut rows: Vec<serde_json::Value> = accounts
        .into_iter()
        .map(|(email, name)| {
            let sessions = store.live_session_count(&email);
            json!({ "name": name, "email": email, "sessions": sessions })
        })
        .collect();
    rows.sort_by_key(|row| row["name"].as_str().unwrap_or_default().to_lowercase());
    Ok(Json(json!(rows)))
}

/// Deletes an account and revokes all of its sessions. The directory entry is
/// untouched — directory roles and credentials are separate concerns.
pub async fn delete_account(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(email): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    let caller = require_perm(&mut store, &headers, "users.manage")?;
    let email = email.trim().to_lowercase();
    if caller.as_deref() == Some(email.as_str()) {
        return Err(ApiError::bad_request("cannot delete your own account"));
    }
    let Some(account) = store.accounts.get(&email).cloned() else {
        return Err(ApiError::not_found("account not found"));
    };
    if store.setup.auth_required {
        let admins: Vec<String> = store
            .setup
            .users
            .iter()
            .filter(|u| role_has(&store.setup, &u.role, "users.manage"))
            .map(|u| u.name.clone())
            .collect();
        let admin_accounts = store
            .accounts
            .values()
            .filter(|a| admins.contains(&a.name))
            .count();
        if admins.contains(&account.name) && admin_accounts <= 1 {
            return Err(ApiError::bad_request(
                "cannot delete the last administrator account",
            ));
        }
    }
    store.accounts.remove(&email);
    store.sessions.retain(|_, session| session.email != email);
    // Roles live on the directory entry — drop it too, so a revoked account
    // does not linger as a phantom user (keep at least one directory entry).
    if store.setup.users.len() > 1 {
        store
            .setup
            .users
            .retain(|u| !u.name.eq_ignore_ascii_case(&account.name));
    }
    tracing::info!(email = %email, "account deleted");
    Ok(Json(json!({ "ok": true })))
}

/// Updates the caller's own display name. Needs a live session (even in demo
/// mode) but no permission — everyone can edit their own profile. Renaming the
/// matching directory entry keeps the role mapping intact, and active post
/// locks held under the old name are rewritten so they are not orphaned.
pub async fn update_profile(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<UpdateProfileBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    let email = require_session(&mut store, &headers)?;
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::bad_request("name is required"));
    }
    validate_len("name", &name, MAX_TEXT)?;
    if store
        .accounts
        .values()
        .any(|a| a.email != email && a.name.eq_ignore_ascii_case(&name))
    {
        return Err(ApiError::conflict("name already taken"));
    }
    let old_name = {
        let account = store
            .accounts
            .get_mut(&email)
            .ok_or_else(|| ApiError::unauthorized("invalid token"))?;
        let old = account.name.clone();
        account.name = name.clone();
        old
    };
    if old_name != name {
        let old_entry = store.setup.users.iter().position(|u| u.name == old_name);
        let new_entry = store.setup.users.iter().position(|u| u.name == name);
        match (old_entry, new_entry) {
            (Some(i), Some(j)) if i != j => {
                return Err(ApiError::conflict("name is already in use"));
            }
            (Some(i), _) => {
                store.setup.users[i].name = name.clone();
            }
            (None, Some(_)) => {
                return Err(ApiError::conflict("name is already in use"));
            }
            (None, None) => {}
        }
        for post in store.posts.iter_mut() {
            if post.locked_by.as_deref() == Some(old_name.as_str()) {
                post.locked_by = Some(name.clone());
            }
        }
    }
    Ok(Json(json!({ "user": user_json(&store, &email) })))
}

pub async fn change_password(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<ChangePasswordBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    validate_password("new password", &body.new_password)?;
    let (email, hash) = {
        let mut store = s.write().await;
        let email = require_session(&mut store, &headers)?;
        let account = store
            .accounts
            .get(&email)
            .ok_or_else(|| ApiError::unauthorized("invalid token"))?;
        (email, account.password_hash.clone())
    };

    let old_password = body.old_password;
    let new_password = body.new_password;
    let (verified, new_hash) = tokio::task::spawn_blocking(move || {
        (
            verify_password(&old_password, &hash),
            hash_password(&new_password),
        )
    })
    .await
    .map_err(|_| ApiError::internal("password hashing failed"))?;

    let mut store = s.write().await;
    if !verified {
        return Err(ApiError::unauthorized("old password is incorrect"));
    }
    let account = store
        .accounts
        .get_mut(&email)
        .ok_or_else(|| ApiError::unauthorized("invalid token"))?;
    account.password_hash = new_hash;
    let current = session_key(bearer_token(&headers).unwrap_or_default());
    store
        .sessions
        .retain(|key, session| key == &current || session.email != email);
    tracing::info!(email = %email, "password changed");
    Ok(Json(json!({ "ok": true })))
}

// ---- posts ----

pub async fn list_posts(
    State(s): State<AppState>,
    AppQuery(q): AppQuery<MonthQuery>,
) -> Json<Vec<Post>> {
    if let Some(month) = q.month {
        if !(1..=12).contains(&month) {
            // Return an empty list rather than leaking a 400 for a read filter.
            return Json(Vec::new());
        }
    }
    let store = s.read().await;
    Json(
        store
            .posts
            .iter()
            .filter(|p| q.month.is_none_or(|month| p.month == month))
            .cloned()
            .collect(),
    )
}

pub async fn add_post(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(mut post): AppJson<Post>,
) -> Result<Json<Post>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "posts.write")?;
    post.id = uid("p");
    post.locked_by = None;
    validate_post(&post, &store.setup)?;
    store.posts.push(post.clone());
    Ok(Json(post))
}

pub async fn update_post(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(patch): AppJson<PostPatch>,
) -> Result<Json<Post>, ApiError> {
    let mut store = s.write().await;
    let actor_email = require_perm(&mut store, &headers, "posts.write")?;
    let actor_name = actor_email.as_ref().and_then(|e| account_name(&store, e));
    let setup = store.setup.clone();
    let post = store
        .posts
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| ApiError::not_found("post not found"))?;
    if let Some(holder) = &post.locked_by {
        let actor = actor_name.as_deref().or(patch.user.as_deref());
        if actor != Some(holder.as_str()) {
            return Err(ApiError::conflict(&format!("post is locked by {holder}")));
        }
    }
    // Apply to a staged copy first: an invalid patch must not corrupt the row.
    let mut candidate = post.clone();
    patch.apply(&mut candidate);
    validate_post(&candidate, &setup)?;
    *post = candidate;
    Ok(Json(post.clone()))
}

/// `DELETE /api/posts/{id}` — removes a planned row. A row locked by someone
/// else cannot be deleted (same rule as editing).
pub async fn delete_post(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    let actor_email = require_perm(&mut store, &headers, "posts.write")?;
    let actor_name = actor_email.as_ref().and_then(|e| account_name(&store, e));
    let index = store
        .posts
        .iter()
        .position(|p| p.id == id)
        .ok_or_else(|| ApiError::not_found("post not found"))?;
    if let Some(holder) = &store.posts[index].locked_by {
        if actor_name.as_deref() != Some(holder.as_str()) {
            return Err(ApiError::conflict(&format!("post is locked by {holder}")));
        }
    }
    store.posts.remove(index);
    tracing::info!(post = %id, "post deleted");
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn lock_post(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<LockBody>,
) -> Result<Json<Post>, ApiError> {
    let mut store = s.write().await;
    let actor_email = require_perm(&mut store, &headers, "posts.lock")?;
    // With auth on the holder is the signed-in account; in demo mode the
    // wireframe keeps the client-supplied name.
    let holder = match actor_email {
        Some(email) => {
            account_name(&store, &email).ok_or_else(|| ApiError::unauthorized("invalid token"))?
        }
        None => body.user.trim().to_string(),
    };
    if holder.is_empty() {
        return Err(ApiError::bad_request("user is required to lock a row"));
    }
    validate_len("user", &holder, MAX_TEXT)?;
    let post = store
        .posts
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| ApiError::not_found("post not found"))?;
    if let Some(current) = &post.locked_by {
        if current != &holder {
            return Err(ApiError::conflict(&format!("post is locked by {current}")));
        }
    }
    post.locked_by = Some(holder);
    Ok(Json(post.clone()))
}

pub async fn unlock_post(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<LockBody>,
) -> Result<Json<Post>, ApiError> {
    let mut store = s.write().await;
    let actor_email = require_perm(&mut store, &headers, "posts.lock")?;
    let holder = match actor_email {
        Some(email) => {
            account_name(&store, &email).ok_or_else(|| ApiError::unauthorized("invalid token"))?
        }
        None => body.user.trim().to_string(),
    };
    if holder.is_empty() {
        return Err(ApiError::bad_request("user is required to unlock a row"));
    }
    let post = store
        .posts
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| ApiError::not_found("post not found"))?;
    if let Some(current) = &post.locked_by {
        if current != &holder {
            return Err(ApiError::conflict(&format!("post is locked by {current}")));
        }
    }
    post.locked_by = None;
    Ok(Json(post.clone()))
}

// ---- ideas ----

pub async fn list_ideas(State(s): State<AppState>) -> Json<Vec<Idea>> {
    Json(s.read().await.ideas.clone())
}

pub async fn add_idea(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(mut idea): AppJson<Idea>,
) -> Result<Json<Idea>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "ideas.write")?;
    idea.topic = idea.topic.trim().to_string();
    if idea.topic.is_empty() {
        return Err(ApiError::bad_request("topic is required"));
    }
    validate_len("topic", &idea.topic, MAX_TEXT)?;
    validate_len("format", &idea.format, MAX_TEXT)?;
    validate_len("idea", &idea.idea, MAX_LONG_TEXT)?;
    validate_len("link", &idea.link, 2000)?;
    idea.id = uid("i");
    store.ideas.push(idea.clone());
    Ok(Json(idea))
}

pub async fn toggle_idea(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Idea>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "ideas.write")?;
    let idea = store
        .ideas
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| ApiError::not_found("idea not found"))?;
    idea.done = !idea.done;
    Ok(Json(idea.clone()))
}

pub async fn promote_idea(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppQuery(q): AppQuery<MonthQuery>,
) -> Result<Json<Post>, ApiError> {
    let month = q
        .month
        .ok_or_else(|| ApiError::bad_request("month is required"))?;
    validate_month(month)?;
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "ideas.write")?;
    let idea = store
        .ideas
        .iter()
        .find(|i| i.id == id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("idea not found"))?;
    let pillar = store
        .setup
        .pillars
        .first()
        .cloned()
        .unwrap_or_else(|| "Pillar I".into());
    let goal = store.setup.goals.first().cloned().unwrap_or_default();
    let group = store
        .hashtags
        .first()
        .map(|g| g.title.clone())
        .unwrap_or_default();
    let post = Post {
        id: uid("p"),
        month,
        topic: idea.topic.clone(),
        pillar,
        format: idea.format.clone(),
        goal,
        date: None,
        time: "09:00".into(),
        status: "Start".into(),
        hook: String::new(),
        caption: idea.idea.clone(),
        cta: String::new(),
        hashtag_group: group,
        hashtags: Vec::new(),
        image_url: idea.link.clone(),
        note: format!("promoted from {}", idea.id),
        done: false,
        platforms: Vec::new(),
        locked_by: None,
    };
    validate_post(&post, &store.setup)?;
    if let Some(i) = store.ideas.iter_mut().find(|i| i.id == id) {
        i.done = true;
    }
    store.posts.push(post.clone());
    Ok(Json(post))
}

// ---- hashtag groups ----

pub async fn list_tags(State(s): State<AppState>) -> Json<Vec<HashtagGroup>> {
    Json(s.read().await.hashtags.clone())
}

pub async fn add_tag(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<AddTagBody>,
) -> Result<Json<HashtagGroup>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "hashtags.write")?;
    let tag = body.tag.trim().trim_start_matches('#').to_string();
    validate_len("tag", &tag, MAX_LIST_ITEM)?;
    let group = store
        .hashtags
        .iter_mut()
        .find(|g| g.id == id)
        .ok_or_else(|| ApiError::not_found("hashtag group not found"))?;
    if !tag.is_empty() && !group.tags.contains(&tag) {
        group.tags.push(tag);
    }
    Ok(Json(group.clone()))
}

// ---- metrics ----

pub async fn list_metrics(State(s): State<AppState>) -> Json<Vec<Metric>> {
    Json(s.read().await.metrics.clone())
}

/// Recomputes deterministic pseudo-metrics for every post scheduled in `month`
/// that targets `platform`, then upserts them into the metric table.
pub async fn import_metrics(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<ImportBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let platform = body.platform.trim().to_string();
    if platform.is_empty() {
        return Err(ApiError::bad_request("platform is required"));
    }
    validate_month(body.month)?;
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "metrics.import")?;
    if !store.setup.platforms.is_empty() && !store.setup.platforms.iter().any(|p| p == &platform) {
        return Err(ApiError::bad_request(&format!(
            "unknown platform '{platform}'"
        )));
    }
    let fresh: Vec<Metric> = store
        .posts
        .iter()
        .filter(|p| p.month == body.month && p.platforms.contains(&platform))
        .map(|p| {
            // Same formula as the frontend mock, so both data sources agree.
            let hash: u64 = format!("{}{}", p.id, platform)
                .bytes()
                .fold(0u64, |h, b| (h + b as u64) % 100_000);
            let views = 1000 + (hash % 9000);
            Metric {
                post_id: p.id.clone(),
                platform: platform.clone(),
                likes: views / 12,
                views,
            }
        })
        .collect();
    for metric in &fresh {
        if let Some(row) = store
            .metrics
            .iter_mut()
            .find(|m| m.post_id == metric.post_id && m.platform == metric.platform)
        {
            *row = metric.clone();
        } else {
            store.metrics.push(metric.clone());
        }
    }
    Ok(Json(json!({ "imported": fresh.len(), "metrics": fresh })))
}

// ---- finance ----

pub async fn list_txns(State(s): State<AppState>) -> Json<Vec<Txn>> {
    Json(s.read().await.txns.clone())
}

pub async fn add_txn(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(mut txn): AppJson<Txn>,
) -> Result<Json<Txn>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "finance.write")?;
    if !txn.amount.is_finite() || txn.amount <= 0.0 || txn.amount > 1e12 {
        return Err(ApiError::bad_request("amount must be a positive number"));
    }
    if txn.kind != "IN" && txn.kind != "OUT" {
        return Err(ApiError::bad_request("kind must be IN or OUT"));
    }
    if txn.date.trim().is_empty() {
        return Err(ApiError::bad_request("date is required"));
    }
    validate_date("date", &txn.date)?;
    validate_len("category", &txn.category, MAX_TEXT)?;
    validate_len("sub", &txn.sub, MAX_TEXT)?;
    txn.id = uid("t");
    store.txns.push(txn.clone());
    Ok(Json(txn))
}

// ---- brand ----

pub async fn get_brand(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Brand>, ApiError> {
    let store = s.read().await;
    let id = active_workspace_id(&store, &headers)?;
    Ok(Json(
        store
            .workspace(&id)
            .map(|w| w.brand.clone())
            .unwrap_or_else(Brand::empty),
    ))
}

/// Logo slots on the Brand page (main, secondary, social).
const MAX_LOGO_SLOTS: usize = 3;
/// Moodboard images kept per brand.
const MAX_MOODBOARD_IMAGES: usize = 12;
/// Largest accepted brand image (matches the upload body limit below).
pub const BRAND_IMAGE_MAX_BYTES: usize = 4 * 1024 * 1024;

/// Pixel bounds for one brand image kind.
#[derive(Clone, Copy, Debug)]
struct ImageBounds {
    min: (u32, u32),
    max: (u32, u32),
}

/// Every brand image is capped at 500×500 px: enough for logos and moodboard
/// thumbnails, small enough that the kit stays light. The frontend shows the
/// same numbers (`src/images.ts`).
const BRAND_IMAGE_BOUNDS: ImageBounds = ImageBounds {
    min: (32, 32),
    max: (500, 500),
};

/// Which bounds apply to an upload (both slots share the same rule).
fn image_bounds(kind: &str) -> Option<ImageBounds> {
    match kind {
        "logo" | "moodboard" => Some(BRAND_IMAGE_BOUNDS),
        _ => None,
    }
}

/// Reads pixel dimensions from the file header (PNG, JPEG, GIF, WebP) without
/// decoding the image. `None` = unrecognized/corrupt header.
pub fn image_dimensions(name: &str, bytes: &[u8]) -> Option<(u32, u32)> {
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_lowercase())?;
    match ext.as_str() {
        "png" if bytes.len() >= 24 && bytes.starts_with(&[0x89, b'P', b'N', b'G']) => {
            let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
            let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
            Some((w, h))
        }
        "gif" if bytes.len() >= 10 && bytes.starts_with(b"GIF8") => {
            let w = u16::from_le_bytes(bytes[6..8].try_into().ok()?) as u32;
            let h = u16::from_le_bytes(bytes[8..10].try_into().ok()?) as u32;
            Some((w, h))
        }
        "jpg" | "jpeg" if bytes.len() >= 4 && bytes.starts_with(&[0xFF, 0xD8]) => {
            // Walk the marker segments to the frame header (SOF0..SOF15 minus
            // DHT/JPG/DAC) which carries the dimensions.
            let mut i = 2usize;
            while i + 9 < bytes.len() {
                if bytes[i] != 0xFF {
                    i += 1;
                    continue;
                }
                let marker = bytes[i + 1];
                if (0xC0..=0xCF).contains(&marker)
                    && marker != 0xC4
                    && marker != 0xC8
                    && marker != 0xCC
                {
                    let h = u16::from_be_bytes(bytes[i + 5..i + 7].try_into().ok()?) as u32;
                    let w = u16::from_be_bytes(bytes[i + 7..i + 9].try_into().ok()?) as u32;
                    return Some((w, h));
                }
                if marker == 0xD8 || (0xD0..=0xD7).contains(&marker) {
                    i += 2;
                    continue;
                }
                let len = u16::from_be_bytes(bytes[i + 2..i + 4].try_into().ok()?) as usize;
                if len < 2 {
                    return None;
                }
                i += 2 + len;
            }
            None
        }
        "webp" if bytes.len() >= 30 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" => {
            match &bytes[12..16] {
                b"VP8 " => {
                    let w = u16::from_le_bytes(bytes[26..28].try_into().ok()?) as u32 & 0x3FFF;
                    let h = u16::from_le_bytes(bytes[28..30].try_into().ok()?) as u32 & 0x3FFF;
                    Some((w, h))
                }
                b"VP8L" => {
                    let bits = u32::from_le_bytes(bytes[21..25].try_into().ok()?);
                    let w = (bits & 0x3FFF) + 1;
                    let h = ((bits >> 14) & 0x3FFF) + 1;
                    Some((w, h))
                }
                b"VP8X" => {
                    let w =
                        (bytes[24] as u32 | (bytes[25] as u32) << 8 | (bytes[26] as u32) << 16) + 1;
                    let h =
                        (bytes[27] as u32 | (bytes[28] as u32) << 8 | (bytes[29] as u32) << 16) + 1;
                    Some((w, h))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Enforces the kind's pixel bounds; `Err` carries a user-facing message.
fn check_image_bounds(kind: &str, width: u32, height: u32) -> Result<ImageBounds, ApiError> {
    let bounds = image_bounds(kind)
        .ok_or_else(|| ApiError::bad_request("kind must be logo or moodboard"))?;
    if width < bounds.min.0 || height < bounds.min.1 {
        return Err(ApiError::bad_request(&format!(
            "{kind} images must be at least {}×{} px (got {width}×{height})",
            bounds.min.0, bounds.min.1
        )));
    }
    if width > bounds.max.0 || height > bounds.max.1 {
        return Err(ApiError::bad_request(&format!(
            "{kind} images must be at most {}×{} px (got {width}×{height})",
            bounds.max.0, bounds.max.1
        )));
    }
    Ok(bounds)
}

/// A brand image is either an uploaded file served by this API or an https URL.
fn validate_brand_image_url(url: &str) -> Result<(), ApiError> {
    if url.is_empty() {
        return Ok(());
    }
    if url.starts_with("/api/brand/images/") || url.starts_with("https://") {
        return Ok(());
    }
    Err(ApiError::bad_request(
        "brand images must be uploaded here or use an https URL",
    ))
}

/// Image kinds the brand kit accepts (raster only: no SVG, no scripts).
fn image_kind(name: &str, bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_lowercase())?;
    let png = bytes.starts_with(&[0x89, b'P', b'N', b'G']);
    let jpg = bytes.starts_with(&[0xFF, 0xD8, 0xFF]);
    let gif = bytes.starts_with(b"GIF8");
    let webp = bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP";
    match ext.as_str() {
        "png" if png => Some(("png", "image/png")),
        "jpg" | "jpeg" if jpg => Some(("jpg", "image/jpeg")),
        "gif" if gif => Some(("gif", "image/gif")),
        "webp" if webp => Some(("webp", "image/webp")),
        _ => None,
    }
}

fn image_slug(stem: &str) -> String {
    let slug: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "image".into()
    } else {
        trimmed.chars().take(60).collect()
    }
}

#[derive(Deserialize)]
pub struct ImageUploadBody {
    pub name: String,
    /// Base64 payload (no `data:` URL prefix).
    pub data: String,
    /// Which slot the image is for: `logo` or `moodboard` (both 32–500 px).
    pub kind: String,
}

/// `POST /api/brand/images` — uploads a logo/moodboard image and returns its
/// URL, which the caller then stores in the brand kit.
pub async fn upload_brand_image(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<ImageUploadBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use base64::Engine as _;

    let mut store = s.write().await;
    require_perm(&mut store, &headers, "brand.write")?;
    let dir = store
        .images_dir
        .clone()
        .ok_or_else(|| ApiError::bad_request("image storage is disabled (DATA_FILE is empty)"))?;

    let name = body.name.trim();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body.data.trim())
        .map_err(|_| ApiError::bad_request("invalid image payload"))?;
    if bytes.is_empty() {
        return Err(ApiError::bad_request("image file is empty"));
    }
    if bytes.len() > BRAND_IMAGE_MAX_BYTES {
        return Err(ApiError::bad_request("image exceeds 4 MB"));
    }
    let Some((ext, _mime)) = image_kind(name, &bytes) else {
        return Err(ApiError::bad_request(
            "unsupported image (use .png, .jpg, .webp or .gif)",
        ));
    };
    let kind = body.kind.trim().to_lowercase();
    let (width, height) = image_dimensions(name, &bytes)
        .ok_or_else(|| ApiError::bad_request("could not read the image dimensions"))?;
    check_image_bounds(&kind, width, height)?;

    std::fs::create_dir_all(&dir).map_err(|_| ApiError::internal("cannot create image storage"))?;
    let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
    let mut file = format!("{}.{}", image_slug(stem), ext);
    let mut n = 2;
    while dir.join(&file).exists() {
        file = format!("{}-{n}.{ext}", image_slug(stem));
        n += 1;
    }
    std::fs::write(dir.join(&file), &bytes)
        .map_err(|_| ApiError::internal("cannot store image"))?;
    tracing::info!(image = %file, bytes = bytes.len(), width, height, kind = %kind, "brand image uploaded");
    Ok(Json(serde_json::json!({
        "url": format!("/api/brand/images/{file}"),
        "width": width,
        "height": height,
    })))
}

/// `DELETE /api/brand/images/{name}` — removes an uploaded image.
pub async fn delete_brand_image(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "brand.write")?;
    let dir = store
        .images_dir
        .clone()
        .ok_or_else(|| ApiError::not_found("image not found"))?;
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(ApiError::bad_request("invalid image name"));
    }
    let path = dir.join(&name);
    let bytes = std::fs::read(&path).map_err(|_| ApiError::not_found("image not found"))?;
    if image_kind(&name, &bytes).is_none() {
        return Err(ApiError::not_found("image not found"));
    }
    std::fs::remove_file(&path).map_err(|_| ApiError::internal("cannot delete image"))?;
    tracing::info!(image = %name, "brand image deleted");
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Serves an uploaded brand image (public, immutable — safe to cache forever).
pub async fn brand_image_file(
    State(s): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, ApiError> {
    let dir = {
        let store = s.read().await;
        store
            .images_dir
            .clone()
            .ok_or_else(|| ApiError::not_found("image not found"))?
    };
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(ApiError::not_found("image not found"));
    }
    let bytes = tokio::fs::read(dir.join(&name))
        .await
        .map_err(|_| ApiError::not_found("image not found"))?;
    let Some((_ext, mime)) = image_kind(&name, &bytes) else {
        return Err(ApiError::not_found("image not found"));
    };
    Ok((
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    )
        .into_response())
}

pub async fn save_brand(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(patch): AppJson<BrandPatch>,
) -> Result<Json<Brand>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "brand.write")?;
    let ws_id = active_workspace_id(&store, &headers)?;
    let b = &mut store
        .workspace_mut(&ws_id)
        .ok_or_else(|| ApiError::not_found("workspace not found"))?
        .brand;
    if let Some(v) = patch.channel {
        validate_len("channel", &v, MAX_TEXT)?;
        b.channel = v;
    }
    if let Some(v) = patch.positioning {
        validate_len("positioning", &v, MAX_TEXT)?;
        b.positioning = v;
    }
    if let Some(v) = patch.slogan {
        validate_len("slogan", &v, MAX_TEXT)?;
        b.slogan = v;
    }
    if let Some(v) = patch.audience {
        validate_len("audience", &v, MAX_TEXT)?;
        b.audience = v;
    }
    if let Some(v) = patch.voice {
        validate_len("voice", &v, MAX_TEXT)?;
        b.voice = v;
    }
    if let Some(v) = patch.dos {
        validate_list("do", &v)?;
        b.dos = v;
    }
    if let Some(v) = patch.donts {
        validate_list("don't", &v)?;
        b.donts = v;
    }
    if let Some(v) = patch.palette {
        validate_list("palette", &v)?;
        b.palette = v;
    }
    if let Some(v) = patch.fonts {
        validate_list("fonts", &v)?;
        b.fonts = v;
    }
    if let Some(v) = patch.logos {
        if v.len() > MAX_LOGO_SLOTS {
            return Err(ApiError::bad_request("too many logo slots (max 3)"));
        }
        for url in &v {
            validate_brand_image_url(url)?;
        }
        b.logos = v;
    }
    if let Some(v) = patch.moodboard {
        if v.len() > MAX_MOODBOARD_IMAGES {
            return Err(ApiError::bad_request("too many moodboard images (max 12)"));
        }
        for url in &v {
            validate_brand_image_url(url)?;
        }
        b.moodboard = v;
    }
    if let Some(v) = patch.radius {
        if v > 24 {
            return Err(ApiError::bad_request("corner radius is 0–24 px"));
        }
        b.radius = v;
    }
    if let Some(v) = patch.fill_opacity {
        if !(5..=100).contains(&v) {
            return Err(ApiError::bad_request("accent fill is 5–100%"));
        }
        b.fill_opacity = v;
    }
    if let Some(v) = patch.stroke_width {
        if v > 3 {
            return Err(ApiError::bad_request("stroke width is 0–3 px"));
        }
        b.stroke_width = v;
    }
    if let Some(v) = patch.shadow {
        let v = v.trim().to_lowercase();
        if !["none", "soft", "strong"].contains(&v.as_str()) {
            return Err(ApiError::bad_request("shadow is none, soft or strong"));
        }
        b.shadow = v;
    }
    Ok(Json(b.clone()))
}

// ---- marketing campaigns ----

const CAMPAIGN_STATUSES: [&str; 4] = ["draft", "active", "paused", "completed"];
const CAMPAIGN_METRICS: [&str; 4] = ["views", "likes", "reach", "posts"];

fn parse_campaign_date(raw: Option<&str>) -> Result<Option<chrono::NaiveDate>, ApiError> {
    let Some(raw) = raw.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| ApiError::bad_request("dates must be YYYY-MM-DD"))
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for value in values {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() && !out.contains(&trimmed) {
            out.push(trimmed);
        }
    }
    out
}

fn apply_campaign_patch(target: &mut Campaign, patch: CampaignPatch) {
    if let Some(v) = patch.name {
        target.name = v.trim().to_string();
    }
    if let Some(v) = patch.objective {
        target.objective = v.trim().to_string();
    }
    if let Some(v) = patch.status {
        target.status = v.trim().to_lowercase();
    }
    if let Some(v) = patch.start_date {
        let t = v.trim().to_string();
        target.start_date = if t.is_empty() { None } else { Some(t) };
    }
    if let Some(v) = patch.end_date {
        let t = v.trim().to_string();
        target.end_date = if t.is_empty() { None } else { Some(t) };
    }
    if let Some(v) = patch.platforms {
        target.platforms = dedupe(v);
    }
    if let Some(v) = patch.pillars {
        target.pillars = dedupe(v);
    }
    if let Some(v) = patch.hashtags {
        target.hashtags = dedupe(v);
    }
    if let Some(v) = patch.budget {
        target.budget = v;
    }
    if let Some(v) = patch.goal_metric {
        target.goal_metric = v.trim().to_lowercase();
    }
    if let Some(v) = patch.goal_target {
        target.goal_target = v;
    }
    if let Some(v) = patch.owner {
        target.owner = v.trim().to_string();
    }
    if let Some(v) = patch.notes {
        target.notes = v.trim().to_string();
    }
    if let Some(v) = patch.content_ids {
        target.content_ids = dedupe(v);
    }
}

fn validate_campaign(store: &Store, campaign: &Campaign) -> Result<(), ApiError> {
    if campaign.name.trim().is_empty() {
        return Err(ApiError::bad_request("campaign name is required"));
    }
    validate_len("name", &campaign.name, MAX_TEXT)?;
    validate_len("objective", &campaign.objective, MAX_TEXT)?;
    validate_len("owner", &campaign.owner, MAX_TEXT)?;
    validate_len("notes", &campaign.notes, MAX_LONG_TEXT)?;
    if !CAMPAIGN_STATUSES.contains(&campaign.status.as_str()) {
        return Err(ApiError::bad_request(
            "status must be draft, active, paused or completed",
        ));
    }
    if !CAMPAIGN_METRICS.contains(&campaign.goal_metric.as_str()) {
        return Err(ApiError::bad_request(
            "goalMetric must be views, likes, reach or posts",
        ));
    }
    if !campaign.budget.is_finite() || !(0.0..=1e12).contains(&campaign.budget) {
        return Err(ApiError::bad_request("budget must be a positive number"));
    }
    if !campaign.goal_target.is_finite() || !(0.0..=1e12).contains(&campaign.goal_target) {
        return Err(ApiError::bad_request(
            "goalTarget must be a positive number",
        ));
    }
    let start = parse_campaign_date(campaign.start_date.as_deref())?;
    let end = parse_campaign_date(campaign.end_date.as_deref())?;
    if let (Some(s), Some(e)) = (start, end) {
        if s > e {
            return Err(ApiError::bad_request(
                "endDate must be on or after startDate",
            ));
        }
    }
    validate_list("platforms", &campaign.platforms)?;
    validate_list("pillars", &campaign.pillars)?;
    validate_list("hashtags", &campaign.hashtags)?;
    if campaign.content_ids.len() > MAX_LIST_ITEMS {
        return Err(ApiError::bad_request(
            "contentIds must have at most 100 items",
        ));
    }
    for id in &campaign.content_ids {
        if !store.content.iter().any(|c| &c.id == id) {
            return Err(ApiError::bad_request(&format!(
                "unknown content item: {id}"
            )));
        }
    }
    Ok(())
}

pub async fn list_campaigns(State(s): State<AppState>) -> Json<Vec<Campaign>> {
    Json(s.read().await.campaigns.clone())
}

pub async fn get_campaign(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Campaign>, ApiError> {
    let store = s.read().await;
    store
        .campaigns
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::not_found("campaign not found"))
}

pub async fn create_campaign(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(patch): AppJson<CampaignPatch>,
) -> Result<Json<Campaign>, ApiError> {
    let mut store = s.write().await;
    let caller = require_perm(&mut store, &headers, "campaigns.write")?;
    // Signed-in creator wins; demo/anonymous falls back to the workspace owner.
    let owner = caller
        .as_deref()
        .and_then(|email| account_name(&store, email))
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| store.setup.owner.clone());
    let now = stamp();
    let mut campaign = Campaign {
        id: uid("cmp"),
        name: String::new(),
        objective: String::new(),
        status: "draft".into(),
        start_date: None,
        end_date: None,
        platforms: Vec::new(),
        pillars: Vec::new(),
        hashtags: Vec::new(),
        budget: 0.0,
        goal_metric: "views".into(),
        goal_target: 0.0,
        owner,
        notes: String::new(),
        content_ids: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    apply_campaign_patch(&mut campaign, patch);
    validate_campaign(&store, &campaign)?;
    store.campaigns.push(campaign.clone());
    tracing::info!(campaign = %campaign.id, "campaign created");
    Ok(Json(campaign))
}

pub async fn update_campaign(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(patch): AppJson<CampaignPatch>,
) -> Result<Json<Campaign>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "campaigns.write")?;
    let index = store
        .campaigns
        .iter()
        .position(|c| c.id == id)
        .ok_or_else(|| ApiError::not_found("campaign not found"))?;
    let mut next = store.campaigns[index].clone();
    apply_campaign_patch(&mut next, patch);
    validate_campaign(&store, &next)?;
    next.updated_at = stamp();
    store.campaigns[index] = next.clone();
    tracing::info!(campaign = %next.id, "campaign updated");
    Ok(Json(next))
}

pub async fn delete_campaign(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "campaigns.delete")?;
    let before = store.campaigns.len();
    store.campaigns.retain(|c| c.id != id);
    if store.campaigns.len() == before {
        return Err(ApiError::not_found("campaign not found"));
    }
    tracing::info!(campaign = %id, "campaign deleted");
    Ok(Json(json!({ "ok": true })))
}

// ---- brand fonts (self-hosted uploads) ----

/// Largest decoded font we accept (the JSON body layer allows ~1.4x this).
pub const FONT_MAX_BYTES: usize = 2 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontUploadBody {
    pub name: String,
    /// Base64 payload (the client strips the `data:` URL prefix).
    pub data: String,
}

/// Content type + extension for a supported font file (magic-byte checked).
fn font_kind(name: &str, bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let (ext, mime, ok) = match ext.as_str() {
        "woff2" => ("woff2", "font/woff2", bytes.starts_with(b"wOF2")),
        "woff" => ("woff", "font/woff", bytes.starts_with(b"wOFF")),
        "ttf" => (
            "ttf",
            "font/ttf",
            bytes.starts_with(&[0x00, 0x01, 0x00, 0x00]) || bytes.starts_with(b"true"),
        ),
        "otf" => ("otf", "font/otf", bytes.starts_with(b"OTTO")),
        _ => return None,
    };
    ok.then_some((ext, mime))
}

/// Human family name from an upload: safe characters only, trimmed to 60.
fn font_family(stem: &str) -> String {
    let cleaned: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        "Imported Font".to_string()
    } else {
        collapsed.chars().take(60).collect()
    }
}

fn font_slug(stem: &str) -> String {
    let slug: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "font".to_string()
    } else {
        slug
    }
}

/// Manifest entry: uploads keep their human family name even though the file
/// itself is slugified.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FontMeta {
    name: String,
    family: String,
    uploaded_at: i64,
}

fn manifest_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("fonts.json")
}

fn load_manifest(dir: &std::path::Path) -> Vec<FontMeta> {
    std::fs::read(manifest_path(dir))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save_manifest(dir: &std::path::Path, entries: &[FontMeta]) -> Result<(), ApiError> {
    let bytes = serde_json::to_vec_pretty(entries)
        .map_err(|_| ApiError::internal("cannot serialize font manifest"))?;
    std::fs::write(manifest_path(dir), bytes)
        .map_err(|_| ApiError::internal("cannot store font manifest"))
}

fn read_font_assets(dir: &std::path::Path) -> Vec<FontAsset> {
    let manifest = load_manifest(dir);
    let meta_for = |file: &str| manifest.iter().find(|m| m.name == file);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut fonts: Vec<FontAsset> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let bytes = std::fs::read(e.path()).ok()?;
            let (name_ext, _) = font_kind(&name, &bytes)?;
            let stem = name.trim_end_matches(&format!(".{name_ext}"));
            let meta = meta_for(&name);
            let uploaded_at = meta.map(|m| m.uploaded_at).unwrap_or_else(|| {
                e.metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            });
            Some(FontAsset {
                url: format!("/api/brand/fonts/{name}/file"),
                family: meta
                    .map(|m| m.family.clone())
                    .unwrap_or_else(|| font_family(stem)),
                size: bytes.len() as u64,
                uploaded_at,
                name,
            })
        })
        .collect();
    fonts.sort_by_key(|f| f.family.to_lowercase());
    fonts
}

/// Public list of uploaded fonts (the theme needs it without auth).
pub async fn list_fonts(State(s): State<AppState>) -> Json<Vec<FontAsset>> {
    let store = s.read().await;
    Json(match store.fonts_dir.as_deref() {
        Some(dir) => read_font_assets(dir),
        None => Vec::new(),
    })
}

pub async fn upload_font(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<FontUploadBody>,
) -> Result<Json<Vec<FontAsset>>, ApiError> {
    use base64::Engine as _;

    let mut store = s.write().await;
    require_perm(&mut store, &headers, "brand.write")?;
    let dir = store
        .fonts_dir
        .clone()
        .ok_or_else(|| ApiError::bad_request("font storage is disabled (DATA_FILE is empty)"))?;

    let name = body.name.trim();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body.data.trim())
        .map_err(|_| ApiError::bad_request("invalid font payload"))?;
    if bytes.is_empty() {
        return Err(ApiError::bad_request("font file is empty"));
    }
    if bytes.len() > FONT_MAX_BYTES {
        return Err(ApiError::bad_request("font file exceeds 2 MB"));
    }
    let Some((ext, _mime)) = font_kind(name, &bytes) else {
        return Err(ApiError::bad_request(
            "unsupported font file (use .woff2, .woff, .ttf or .otf)",
        ));
    };

    std::fs::create_dir_all(&dir).map_err(|_| ApiError::internal("cannot create font storage"))?;
    let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
    let base = format!("{}.{}", font_slug(stem), ext);
    let mut file = base.clone();
    let mut n = 2;
    while dir.join(&file).exists() {
        file = format!("{}-{n}.{ext}", font_slug(stem));
        n += 1;
    }
    std::fs::write(dir.join(&file), &bytes).map_err(|_| ApiError::internal("cannot store font"))?;
    let mut manifest = load_manifest(&dir);
    manifest.retain(|m| m.name != file);
    manifest.push(FontMeta {
        family: font_family(stem),
        uploaded_at: Utc::now().timestamp(),
        name: file.clone(),
    });
    save_manifest(&dir, &manifest)?;
    tracing::info!(font = %file, bytes = bytes.len(), "brand font uploaded");
    Ok(Json(read_font_assets(&dir)))
}

pub async fn delete_font(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<Vec<FontAsset>>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "brand.write")?;
    let dir = store
        .fonts_dir
        .clone()
        .ok_or_else(|| ApiError::not_found("font not found"))?;
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(ApiError::bad_request("invalid font name"));
    }
    let path = dir.join(&name);
    let bytes = std::fs::read(&path).map_err(|_| ApiError::not_found("font not found"))?;
    if font_kind(&name, &bytes).is_none() {
        return Err(ApiError::not_found("font not found"));
    }
    std::fs::remove_file(&path).map_err(|_| ApiError::internal("cannot delete font"))?;
    let mut manifest = load_manifest(&dir);
    manifest.retain(|m| m.name != name);
    save_manifest(&dir, &manifest)?;
    tracing::info!(font = %name, "brand font deleted");
    Ok(Json(read_font_assets(&dir)))
}

/// Serves an uploaded font file (public, immutable — safe to cache forever).
pub async fn font_file(
    State(s): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, ApiError> {
    let dir = {
        let store = s.read().await;
        store
            .fonts_dir
            .clone()
            .ok_or_else(|| ApiError::not_found("font not found"))?
    };
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(ApiError::not_found("font not found"));
    }
    let bytes = tokio::fs::read(dir.join(&name))
        .await
        .map_err(|_| ApiError::not_found("font not found"))?;
    let Some((_ext, mime)) = font_kind(&name, &bytes) else {
        return Err(ApiError::not_found("font not found"));
    };
    Ok((
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    )
        .into_response())
}

// ---- platform connections ----

/// A connection plus runtime-only fields the UI needs. `hasToken` is never
/// persisted: it says whether a usable provider token exists right now, so the
/// app can ask for a reconnect instead of showing a connection that cannot
/// actually sync.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionView {
    #[serde(flatten)]
    pub connection: PlatformConnection,
    pub has_token: bool,
}

pub async fn list_platforms(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Json<Vec<ConnectionView>> {
    let store = s.read().await;
    let workspace_id = active_workspace_id(&store, &headers).ok();
    let connections = workspace_id
        .as_deref()
        .and_then(|id| store.workspace(id))
        .map(|w| w.connections.clone())
        .unwrap_or_default();
    let views = connections
        .into_iter()
        .map(|connection| ConnectionView {
            has_token: workspace_id
                .as_deref()
                .is_some_and(|ws| store.has_platform_token(ws, &connection.id)),
            connection,
        })
        .collect();
    Json(views)
}

/// Accepted public-profile hosts per platform.
fn profile_hosts(id: &str) -> &'static [&'static str] {
    match id {
        // One Meta connection covers the Facebook Page and its linked
        // Instagram account; the Page profile is the linkable handle.
        "meta" => &["facebook.com", "www.facebook.com", "m.facebook.com"],
        "youtube" => &["youtube.com", "www.youtube.com", "youtu.be"],
        "tiktok" => &["tiktok.com", "www.tiktok.com"],
        _ => &[],
    }
}

/// A linkable profile is either an @handle or a profile URL on the right host —
/// never the platform home page (which would make "connected" meaningless).
fn valid_profile_handle(id: &str, handle: &str) -> bool {
    let raw = handle.trim();
    if raw.is_empty() {
        return false;
    }
    if raw.starts_with("http://") || raw.starts_with("https://") {
        let Ok(url) = url::Url::parse(raw) else {
            return false;
        };
        let host = url.host_str().unwrap_or("").to_lowercase();
        if !profile_hosts(id).contains(&host.as_str()) {
            return false;
        }
        let path = url.path().trim_matches('/').to_lowercase();
        return !path.is_empty() && path != "index.html";
    }
    let clean = raw.trim_start_matches('@');
    !clean.is_empty()
        && clean.len() <= 60
        && clean
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectBody {
    /// Public profile URL or @handle; stored so the UI can link to the account.
    #[serde(default)]
    pub handle: Option<String>,
}

pub async fn connect_platform(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<ConnectBody>,
) -> Result<Json<PlatformConnection>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "platforms.manage")?;
    let ws_id = active_workspace_id(&store, &headers)?;
    let configured = crate::oauth::provider(&id).is_some_and(|p| store.oauth.configured(p));
    if configured {
        return Err(ApiError::bad_request(
            "this platform uses the OAuth redirect flow — start it from the platform login page",
        ));
    }
    let mut conn = crate::oauth::mark_connected(&mut store, &ws_id, &id)
        .ok_or_else(|| ApiError::not_found("platform not found"))?;
    if let Some(raw) = body.handle {
        let handle = raw.trim().to_string();
        validate_len("handle", &handle, 200)?;
        if !handle.is_empty() && !valid_profile_handle(&id, &handle) {
            return Err(ApiError::bad_request(
                "enter a profile URL or @handle (not the platform home page)",
            ));
        }
        if let Some(c) = store
            .workspace_mut(&ws_id)
            .and_then(|w| w.connections.iter_mut().find(|c| c.id == id))
        {
            c.handle = handle.clone();
        }
        conn.handle = handle;
    }
    Ok(Json(conn))
}

pub async fn disconnect_platform(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<PlatformConnection>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "platforms.manage")?;
    let ws_id = active_workspace_id(&store, &headers)?;
    let connection = {
        let c = store
            .workspace_mut(&ws_id)
            .and_then(|w| w.connections.iter_mut().find(|c| c.id == id))
            .ok_or_else(|| ApiError::not_found("platform not found"))?;
        c.status = "disconnected".into();
        c.token_type = "—".into();
        c.expires_at = None;
        c.last_sync = None;
        c.clone()
    };
    store.clear_platform_token(&ws_id, &id);
    Ok(Json(connection))
}

/// `POST /api/platforms/{id}/sync` — pulls fresh content. Meta is a real Graph
/// refresh (posts + Instagram media + engagement); the other providers do not
/// have a live integration yet and only get their sync stamp bumped.
pub async fn sync_platform(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<PlatformConnection>, ApiError> {
    let ws_id = {
        let mut store = s.write().await;
        require_perm(&mut store, &headers, "platforms.manage")?;
        let ws_id = active_workspace_id(&store, &headers)?;
        let status = store
            .workspace(&ws_id)
            .and_then(|w| w.connections.iter().find(|c| c.id == id))
            .map(|c| c.status.clone())
            .ok_or_else(|| ApiError::not_found("platform not found"))?;
        if status != "connected" {
            return Err(ApiError::bad_request("platform is not connected"));
        }
        ws_id
    };
    if id == "meta" {
        // Network I/O happens without the store lock (see `live`).
        crate::live::refresh_workspace(&s, &ws_id)
            .await
            .map_err(|error| ApiError::bad_request(&error))?;
    } else {
        let mut store = s.write().await;
        if let Some(c) = store
            .workspace_mut(&ws_id)
            .and_then(|w| w.connections.iter_mut().find(|c| c.id == id))
        {
            c.last_sync = Some(stamp());
            c.media_count = c.media_count.saturating_add(3);
        }
    }
    let store = s.read().await;
    let connection = store
        .workspace(&ws_id)
        .and_then(|w| w.connections.iter().find(|c| c.id == id))
        .cloned()
        .ok_or_else(|| ApiError::not_found("platform not found"))?;
    Ok(Json(connection))
}

/// `GET /api/live` — the workspace's mirror of connected-platform content
/// (fetched by sync / the background refresh; never edited in the app).
pub async fn get_live(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::model::LiveData>, ApiError> {
    let store = s.read().await;
    let ws_id = active_workspace_id(&store, &headers)?;
    let live = store
        .workspace(&ws_id)
        .map(|w| w.live.clone())
        .ok_or_else(|| ApiError::not_found("workspace not found"))?;
    Ok(Json(live))
}

// ---- Meta Ads: mirror (read) + management (write) ----

/// `GET /api/ads` — the workspace's Meta Ads mirror. `canManage` tells the UI
/// whether the workspace opt-in is on; the manage endpoints enforce it anyway.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdsView {
    #[serde(flatten)]
    pub ads: crate::model::AdsData,
    pub can_manage: bool,
}

pub async fn get_ads(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdsView>, ApiError> {
    let store = s.read().await;
    let ws_id = active_workspace_id(&store, &headers)?;
    let workspace = store
        .workspace(&ws_id)
        .ok_or_else(|| ApiError::not_found("workspace not found"))?;
    Ok(Json(AdsView {
        ads: workspace.ads.clone(),
        can_manage: workspace.ads_manage,
    }))
}

/// `POST /api/ads/sync` — real Graph refresh of the ads mirror for Meta.
pub async fn sync_ads(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdsView>, ApiError> {
    let ws_id = {
        let mut store = s.write().await;
        require_perm(&mut store, &headers, "platforms.manage")?;
        let ws_id = active_workspace_id(&store, &headers)?;
        let connected = store.workspace(&ws_id).is_some_and(|w| {
            w.connections
                .iter()
                .any(|c| c.id == "meta" && c.status == "connected")
        });
        if !connected {
            return Err(ApiError::bad_request("connect Meta first"));
        }
        ws_id
    };
    // Network I/O happens without the store lock (see `ads`).
    crate::ads::refresh_workspace_ads(&s, &ws_id)
        .await
        .map_err(|error| ApiError::bad_request(&error))?;
    get_ads(State(s), headers).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdsManageBody {
    pub enabled: bool,
}

/// `POST /api/ads/manage` — the explicit opt-in for campaign writes. Kept
/// separate from the mirror so reading can never mutate campaigns by accident.
pub async fn set_ads_manage(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<AdsManageBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "platforms.manage")?;
    let ws_id = active_workspace_id(&store, &headers)?;
    let workspace = store
        .workspace_mut(&ws_id)
        .ok_or_else(|| ApiError::not_found("workspace not found"))?;
    workspace.ads_manage = body.enabled;
    Ok(Json(
        serde_json::json!({ "canManage": workspace.ads_manage }),
    ))
}

#[derive(Deserialize)]
pub struct AdStatusBody {
    pub status: String,
}

/// `POST /api/ads/campaigns/{id}/status` — pause/resume/archive a campaign.
pub async fn set_ad_campaign_status(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<AdStatusBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (ws_id, actor) = {
        let mut store = s.write().await;
        let actor = require_perm(&mut store, &headers, "platforms.manage")?;
        let ws_id = active_workspace_id(&store, &headers)?;
        (ws_id, actor.unwrap_or_else(|| "operator".into()))
    };
    crate::ads::set_campaign_status(&s, &ws_id, &id, &body.status, &actor)
        .await
        .map_err(|error| ApiError::bad_request(&error))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AdBudgetBody {
    /// Minor currency units (as Graph reports budgets), e.g. 50000 = 500.00.
    pub daily_budget: Option<u64>,
    pub lifetime_budget: Option<u64>,
}

/// `POST /api/ads/campaigns/{id}/budget` — update daily/lifetime budget.
pub async fn set_ad_campaign_budget(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<AdBudgetBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (ws_id, actor) = {
        let mut store = s.write().await;
        let actor = require_perm(&mut store, &headers, "platforms.manage")?;
        let ws_id = active_workspace_id(&store, &headers)?;
        (ws_id, actor.unwrap_or_else(|| "operator".into()))
    };
    crate::ads::set_campaign_budget(
        &s,
        &ws_id,
        &id,
        body.daily_budget,
        body.lifetime_budget,
        &actor,
    )
    .await
    .map_err(|error| ApiError::bad_request(&error))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// `POST /api/ads/campaigns/{id}/duplicate` — deep copy, paused.
pub async fn duplicate_ad_campaign(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (ws_id, actor) = {
        let mut store = s.write().await;
        let actor = require_perm(&mut store, &headers, "platforms.manage")?;
        let ws_id = active_workspace_id(&store, &headers)?;
        (ws_id, actor.unwrap_or_else(|| "operator".into()))
    };
    let new_id = crate::ads::duplicate_campaign(&s, &ws_id, &id, &actor)
        .await
        .map_err(|error| ApiError::bad_request(&error))?;
    Ok(Json(serde_json::json!({ "campaignId": new_id })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostBody {
    pub name: String,
    pub objective: String,
    /// Minor currency units.
    pub daily_budget: u64,
    pub days: u64,
    pub countries: Vec<String>,
    /// Page post id from the live mirror.
    pub story_id: String,
}

/// `POST /api/ads/boost` — create a paused campaign that promotes an existing
/// Page post. Nothing spends until the user resumes it.
pub async fn create_ad_boost(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<BoostBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (ws_id, actor, page_id) = {
        let mut store = s.write().await;
        let actor = require_perm(&mut store, &headers, "platforms.manage")?;
        let ws_id = active_workspace_id(&store, &headers)?;
        let page_id = store
            .workspace(&ws_id)
            .and_then(|w| w.connections.iter().find(|c| c.id == "meta"))
            .map(|c| c.external_id.clone())
            .unwrap_or_default();
        (ws_id, actor.unwrap_or_else(|| "operator".into()), page_id)
    };
    if page_id.is_empty() {
        return Err(ApiError::bad_request("reconnect Meta and pick a Page"));
    }
    let campaign_id = crate::ads::create_boost(
        &s,
        &ws_id,
        &body.name,
        &body.objective,
        body.daily_budget,
        body.days,
        &body.countries,
        &page_id,
        &body.story_id,
        &actor,
    )
    .await
    .map_err(|error| ApiError::bad_request(&error))?;
    Ok(Json(serde_json::json!({ "campaignId": campaign_id })))
}

// ---- CMS: content studio ----

#[derive(Deserialize)]
pub struct ContentQuery {
    pub status: Option<String>,
    pub q: Option<String>,
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// URL-safe slug from a title (ASCII; non-ASCII collapses to a placeholder).
fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "content".to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 120
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn unique_slug(existing: &[Content], base: &str, exclude_id: Option<&str>) -> String {
    let taken = |slug: &str| {
        existing
            .iter()
            .any(|c| Some(c.id.as_str()) != exclude_id && c.slug == slug)
    };
    if !taken(base) {
        return base.to_string();
    }
    for n in 2..10_000u32 {
        let candidate = format!("{base}-{n}");
        if !taken(&candidate) {
            return candidate;
        }
    }
    format!("{base}-{}", crate::store::random_hex(4))
}

/// Flips scheduled items whose time has arrived to published (lazy scheduler).
fn promote_due_scheduled(store: &mut Store) {
    let now = Utc::now();
    for c in store.content.iter_mut() {
        if c.status != "scheduled" {
            continue;
        }
        let due = c
            .scheduled_for
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|at| at <= now)
            .unwrap_or(false);
        if due {
            let stamp = now.to_rfc3339_opts(SecondsFormat::Secs, true);
            c.status = "published".into();
            c.published_at = Some(stamp.clone());
            c.scheduled_for = None;
            c.updated_at = stamp;
            c.version += 1;
        }
    }
}

fn validate_content(content: &Content) -> Result<(), ApiError> {
    if content.title.trim().is_empty() {
        return Err(ApiError::bad_request("title is required"));
    }
    validate_len("title", &content.title, 200)?;
    if !valid_slug(&content.slug) {
        return Err(ApiError::bad_request(
            "slug must use lowercase letters, numbers and dashes",
        ));
    }
    if !CONTENT_KINDS.contains(&content.kind.as_str()) {
        return Err(ApiError::bad_request("kind must be article, page or note"));
    }
    if !CONTENT_STATUSES.contains(&content.status.as_str()) {
        return Err(ApiError::bad_request("unknown content status"));
    }
    validate_len("body", &content.body, 200_000)?;
    validate_len("excerpt", &content.excerpt, 500)?;
    validate_len("heroImageUrl", &content.hero_image_url, 2_000)?;
    validate_len("seoTitle", &content.seo_title, 200)?;
    validate_len("seoDescription", &content.seo_description, 500)?;
    if content.tags.len() > 20 {
        return Err(ApiError::bad_request("tags must have at most 20 items"));
    }
    for tag in &content.tags {
        validate_len("tag", tag, 50)?;
    }
    Ok(())
}

fn check_content_version(content: &Content, version: Option<u32>) -> Result<(), ApiError> {
    let version = version.ok_or_else(|| ApiError::bad_request("version is required"))?;
    if content.version != version {
        return Err(ApiError::conflict(&format!(
            "content changed since version {version} (current version is {})",
            content.version
        )));
    }
    Ok(())
}

/// Stores the pre-edit state as a revision (bounded history).
fn push_revision(content: &mut Content, author: &str, note: &str) {
    let next = content
        .revisions
        .iter()
        .map(|r| r.revision)
        .max()
        .unwrap_or(0)
        + 1;
    content.revisions.push(ContentRevision {
        revision: next,
        saved_at: now_iso(),
        author: author.to_string(),
        note: note.to_string(),
        title: content.title.clone(),
        body: content.body.clone(),
        excerpt: content.excerpt.clone(),
        seo_title: content.seo_title.clone(),
        seo_description: content.seo_description.clone(),
        tags: content.tags.clone(),
        status: content.status.clone(),
    });
    if content.revisions.len() > MAX_CONTENT_REVISIONS {
        let excess = content.revisions.len() - MAX_CONTENT_REVISIONS;
        content.revisions.drain(0..excess);
    }
}

/// The display name recorded on content edits: the signed-in account, or the
/// client-supplied/demo fallback while auth is off.
fn content_actor(
    store: &mut Store,
    headers: &HeaderMap,
    perm: &str,
    fallback: &str,
) -> Result<String, ApiError> {
    let email = require_perm(store, headers, perm)?;
    // Even while auth is off, attribute to the signed-in account when the
    // request carries a valid session (client studio demo mode).
    let email = match email {
        Some(email) => Some(email),
        None => bearer_token(headers)
            .and_then(|token| store.session(token))
            .filter(|session| session.expires > Utc::now())
            .map(|session| session.email.clone()),
    };
    if let Some(email) = email {
        return Ok(account_name(store, &email).unwrap_or_else(|| store.setup.owner.clone()));
    }
    let fallback = fallback.trim();
    Ok(if fallback.is_empty() {
        store.setup.owner.clone()
    } else {
        fallback.to_string()
    })
}

/// API view: revision bodies are omitted from the item payload (fetch a
/// revision explicitly when you need its text).
fn content_view(mut content: Content) -> Content {
    for revision in &mut content.revisions {
        revision.body = String::new();
    }
    content
}

pub async fn list_content(
    State(s): State<AppState>,
    AppQuery(q): AppQuery<ContentQuery>,
) -> Json<Vec<ContentSummary>> {
    let mut store = s.write().await;
    promote_due_scheduled(&mut store);
    let needle = q.q.unwrap_or_default().trim().to_lowercase();
    let mut rows: Vec<ContentSummary> = store
        .content
        .iter()
        .filter(|c| q.status.as_deref().is_none_or(|status| c.status == status))
        .filter(|c| {
            needle.is_empty()
                || c.title.to_lowercase().contains(&needle)
                || c.excerpt.to_lowercase().contains(&needle)
                || c.tags.iter().any(|t| t.to_lowercase().contains(&needle))
        })
        .map(Content::summary)
        .collect();
    rows.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Json(rows)
}

pub async fn get_content(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Content>, ApiError> {
    let mut store = s.write().await;
    promote_due_scheduled(&mut store);
    store
        .content
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .map(content_view)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("content not found"))
}

pub async fn create_content(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(mut content): AppJson<Content>,
) -> Result<Json<Content>, ApiError> {
    let mut store = s.write().await;
    let author = content_actor(&mut store, &headers, "content.write", &content.author)?;
    content.id = uid("c");
    content.author = author;
    content.version = 1;
    content.created_at = now_iso();
    content.updated_at = content.created_at.clone();
    content.published_at = None;
    content.scheduled_for = None;
    content.revisions = Vec::new();
    if !CONTENT_STATUSES.contains(&content.status.as_str())
        || !matches!(content.status.as_str(), "draft" | "review")
    {
        content.status = "draft".into();
    }
    if content.slug.trim().is_empty() {
        content.slug = slugify(&content.title);
    }
    content.slug = unique_slug(&store.content, &content.slug, None);
    validate_content(&content)?;
    store.content.push(content.clone());
    Ok(Json(content_view(content)))
}

pub async fn update_content(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(patch): AppJson<ContentPatch>,
) -> Result<Json<Content>, ApiError> {
    let mut store = s.write().await;
    let author = content_actor(&mut store, &headers, "content.write", "")?;
    let index = store
        .content
        .iter()
        .position(|c| c.id == id)
        .ok_or_else(|| ApiError::not_found("content not found"))?;
    check_content_version(&store.content[index], patch.version)?;
    let note = patch.note.clone().unwrap_or_else(|| "Edit".into());
    if let Some(status) = patch.status.as_deref() {
        if !matches!(status, "draft" | "review" | "archived") {
            return Err(ApiError::bad_request(
                "status can only be set to draft, review or archived here",
            ));
        }
    }
    let mut candidate = store.content[index].clone();
    patch.apply(&mut candidate);
    if candidate.slug.trim().is_empty() {
        return Err(ApiError::bad_request("slug is required"));
    }
    candidate.slug = unique_slug(&store.content, &candidate.slug, Some(&id));
    validate_content(&candidate)?;

    push_revision(&mut store.content[index], &author, &note);
    let revisions = store.content[index].revisions.clone();
    candidate.revisions = revisions;
    candidate.version = store.content[index].version + 1;
    candidate.updated_at = now_iso();
    store.content[index] = candidate.clone();
    Ok(Json(content_view(candidate)))
}

pub async fn publish_content(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<VersionBody>,
) -> Result<Json<Content>, ApiError> {
    let mut store = s.write().await;
    let author = content_actor(&mut store, &headers, "content.publish", "")?;
    let index = store
        .content
        .iter()
        .position(|c| c.id == id)
        .ok_or_else(|| ApiError::not_found("content not found"))?;
    check_content_version(&store.content[index], Some(body.version))?;
    if store.content[index].title.trim().is_empty() {
        return Err(ApiError::bad_request("title is required"));
    }
    push_revision(&mut store.content[index], &author, "Published");
    let stamp = now_iso();
    let content = &mut store.content[index];
    content.status = "published".into();
    content.published_at = Some(stamp.clone());
    content.scheduled_for = None;
    content.updated_at = stamp;
    content.version += 1;
    Ok(Json(content_view(content.clone())))
}

pub async fn unpublish_content(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<VersionBody>,
) -> Result<Json<Content>, ApiError> {
    let mut store = s.write().await;
    let author = content_actor(&mut store, &headers, "content.publish", "")?;
    let index = store
        .content
        .iter()
        .position(|c| c.id == id)
        .ok_or_else(|| ApiError::not_found("content not found"))?;
    check_content_version(&store.content[index], Some(body.version))?;
    push_revision(&mut store.content[index], &author, "Unpublished");
    let stamp = now_iso();
    let content = &mut store.content[index];
    content.status = "draft".into();
    content.scheduled_for = None;
    content.updated_at = stamp;
    content.version += 1;
    Ok(Json(content_view(content.clone())))
}

pub async fn schedule_content(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<ScheduleBody>,
) -> Result<Json<Content>, ApiError> {
    let when = DateTime::parse_from_rfc3339(body.scheduled_for.trim())
        .map_err(|_| ApiError::bad_request("scheduledFor must be an RFC 3339 timestamp"))?
        .with_timezone(&Utc);
    if when <= Utc::now() {
        return Err(ApiError::bad_request("scheduledFor must be in the future"));
    }
    let mut store = s.write().await;
    let author = content_actor(&mut store, &headers, "content.publish", "")?;
    let index = store
        .content
        .iter()
        .position(|c| c.id == id)
        .ok_or_else(|| ApiError::not_found("content not found"))?;
    check_content_version(&store.content[index], Some(body.version))?;
    push_revision(&mut store.content[index], &author, "Scheduled");
    let stamp = now_iso();
    let content = &mut store.content[index];
    content.status = "scheduled".into();
    content.scheduled_for = Some(when.to_rfc3339_opts(SecondsFormat::Secs, true));
    content.updated_at = stamp;
    content.version += 1;
    Ok(Json(content_view(content.clone())))
}

pub async fn delete_content(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "content.delete")?;
    let before = store.content.len();
    store.content.retain(|c| c.id != id);
    if store.content.len() == before {
        return Err(ApiError::not_found("content not found"));
    }
    Ok(Json(json!({ "ok": true })))
}

pub async fn duplicate_content(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Content>, ApiError> {
    let mut store = s.write().await;
    let author = content_actor(&mut store, &headers, "content.write", "")?;
    let original = store
        .content
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("content not found"))?;
    let stamp = now_iso();
    let mut copy = original;
    copy.id = uid("c");
    copy.title = format!("{} (copy)", copy.title);
    copy.slug = unique_slug(&store.content, &slugify(&copy.title), None);
    copy.status = "draft".into();
    copy.published_at = None;
    copy.scheduled_for = None;
    copy.version = 1;
    copy.author = author;
    copy.created_at = stamp.clone();
    copy.updated_at = stamp;
    copy.revisions = Vec::new();
    store.content.push(copy.clone());
    Ok(Json(content_view(copy)))
}

pub async fn list_content_revisions(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ContentRevision>>, ApiError> {
    let store = s.read().await;
    let content = store
        .content
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| ApiError::not_found("content not found"))?;
    let mut revisions = content.revisions.clone();
    revisions.sort_by_key(|r| std::cmp::Reverse(r.revision));
    Ok(Json(revisions))
}

pub async fn get_content_revision(
    State(s): State<AppState>,
    Path((id, revision)): Path<(String, u32)>,
) -> Result<Json<ContentRevision>, ApiError> {
    let store = s.read().await;
    let content = store
        .content
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| ApiError::not_found("content not found"))?;
    content
        .revisions
        .iter()
        .find(|r| r.revision == revision)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::not_found("revision not found"))
}

pub async fn restore_content_revision(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path((id, revision)): Path<(String, u32)>,
    AppJson(body): AppJson<VersionBody>,
) -> Result<Json<Content>, ApiError> {
    let mut store = s.write().await;
    let author = content_actor(&mut store, &headers, "content.write", "")?;
    let index = store
        .content
        .iter()
        .position(|c| c.id == id)
        .ok_or_else(|| ApiError::not_found("content not found"))?;
    check_content_version(&store.content[index], Some(body.version))?;
    let revision = store.content[index]
        .revisions
        .iter()
        .find(|r| r.revision == revision)
        .cloned()
        .ok_or_else(|| ApiError::not_found("revision not found"))?;

    push_revision(
        &mut store.content[index],
        &author,
        &format!("Restored revision {}", revision.revision),
    );
    let stamp = now_iso();
    let content = &mut store.content[index];
    content.title = revision.title;
    content.body = revision.body;
    content.excerpt = revision.excerpt;
    content.seo_title = revision.seo_title;
    content.seo_description = revision.seo_description;
    content.tags = revision.tags;
    // Never silently republish restored text.
    content.status = if matches!(revision.status.as_str(), "draft" | "review") {
        revision.status
    } else {
        "draft".into()
    };
    content.scheduled_for = None;
    content.updated_at = stamp;
    content.version += 1;
    Ok(Json(content_view(content.clone())))
}

// ---- public delivery (published content only) ----

pub async fn list_public_content(State(s): State<AppState>) -> Json<Vec<ContentSummary>> {
    let mut store = s.write().await;
    promote_due_scheduled(&mut store);
    let mut rows: Vec<ContentSummary> = store
        .content
        .iter()
        .filter(|c| c.status == "published")
        .map(Content::summary)
        .collect();
    rows.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    Json(rows)
}

pub async fn get_public_content(
    State(s): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = s.write().await;
    promote_due_scheduled(&mut store);
    let content = store
        .content
        .iter()
        .find(|c| c.slug == slug && c.status == "published")
        .ok_or_else(|| ApiError::not_found("content not found"))?;
    Ok(Json(json!({
        "id": content.id,
        "title": content.title,
        "slug": content.slug,
        "kind": content.kind,
        "body": content.body,
        "excerpt": content.excerpt,
        "heroImageUrl": content.hero_image_url,
        "tags": content.tags,
        "author": content.author,
        "publishedAt": content.published_at,
        "updatedAt": content.updated_at,
    })))
}

// ---- client onboarding: admin-created accounts ----

/// Creates a sign-in account and (if needed) its directory entry, so an admin
/// can onboard a client without opening public registration. When no password
/// is supplied, one is generated and returned exactly once.
pub async fn create_account(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<NewAccountBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::bad_request("name is required"));
    }
    validate_len("name", &name, MAX_TEXT)?;
    let email = body.email.trim().to_lowercase();
    if email.len() > 254 || !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(ApiError::bad_request("a valid email is required"));
    }
    let role = if body.role.trim().is_empty() {
        "Viewer".to_string()
    } else {
        body.role.trim().to_string()
    };
    let generated = body.password.is_empty();
    let password = if generated {
        format!("cp-{}", crate::store::random_hex(6))
    } else {
        validate_password("password", &body.password)?;
        body.password.clone()
    };

    {
        let mut store = s.write().await;
        require_perm(&mut store, &headers, "users.manage")?;
        if !store.setup.roles.iter().any(|r| r.name == role) {
            return Err(ApiError::bad_request(&format!("unknown role '{role}'")));
        }
        if store.accounts.contains_key(&email) {
            return Err(ApiError::conflict("email already registered"));
        }
        if store
            .accounts
            .values()
            .any(|a| a.name.eq_ignore_ascii_case(&name))
        {
            return Err(ApiError::conflict("name already taken"));
        }
        // Assign/replace the directory role for this client.
        if let Some(user) = store
            .setup
            .users
            .iter_mut()
            .find(|u| u.name.eq_ignore_ascii_case(&name))
        {
            user.role = role.clone();
        } else {
            store.setup.users.push(User {
                name: name.clone(),
                role: role.clone(),
            });
        }
    }

    let password_for_hash = password.clone();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_for_hash))
        .await
        .map_err(|_| ApiError::internal("password hashing failed"))?;

    let mut store = s.write().await;
    if store.accounts.contains_key(&email) {
        return Err(ApiError::conflict("email already registered"));
    }
    store.accounts.insert(
        email.clone(),
        Account {
            name: name.clone(),
            email: email.clone(),
            password_hash,
            plan: "free".into(),
            last_active: None,
        },
    );
    tracing::info!(email = %email, role = %role, "account created by admin");
    Ok(Json(json!({
        "ok": true,
        "account": { "name": name, "email": email, "role": role, "sessions": 0 },
        "temporaryPassword": if generated { Some(password) } else { None },
    })))
}

// ---- built SPA serving (no-Docker deployment) ----

/// Fallback for every unmatched route: JSON 404 under `/api`, otherwise the
/// built SPA (or the JSON envelope when no `STATIC_DIR` is configured).
pub async fn serve_fallback(static_dir: Option<std::path::PathBuf>, req: Request) -> Response {
    let path = req.uri().path().to_string();
    if path == "/api" || path.starts_with("/api/") {
        return ApiError::not_found("route not found").into_response();
    }
    let Some(dir) = static_dir else {
        return ApiError::not_found("route not found").into_response();
    };
    if req.method() != Method::GET && req.method() != Method::HEAD {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            Json(json!({ "error": "method not allowed" })),
        )
            .into_response();
    }
    serve_static_file(&dir, &path).await
}

async fn serve_static_file(dir: &std::path::Path, url_path: &str) -> Response {
    let relative = url_path.trim_start_matches('/');
    // Reject traversal attempts (literal or percent-encoded separators/dots).
    let suspicious = relative.split('/').any(|segment| {
        let lower = segment.to_ascii_lowercase();
        segment == ".."
            || lower.contains("%2e")
            || lower.contains("%2f")
            || lower.contains("%5c")
            || segment.contains('\\')
    });
    if relative.contains('\0') || suspicious {
        return ApiError::not_found("not found").into_response();
    }

    let requested = if relative.is_empty() {
        dir.join("index.html")
    } else {
        dir.join(relative)
    };
    let mut file = requested.clone();
    let exists = tokio::fs::metadata(&file)
        .await
        .map(|meta| meta.is_file())
        .unwrap_or(false);
    if !exists {
        // Client-side routes fall back to the app shell; missing assets stay 404.
        if std::path::Path::new(relative).extension().is_some() {
            return ApiError::not_found("not found").into_response();
        }
        file = dir.join("index.html");
    }

    match tokio::fs::read(&file).await {
        Ok(bytes) => {
            let name = file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            let mut response = Response::new(Body::from(bytes));
            let headers = response.headers_mut();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime_for(name)),
            );
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static(cache_for(relative)),
            );
            response
        }
        Err(_) => ApiError::not_found("not found").into_response(),
    }
}

fn mime_for(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn cache_for(relative_path: &str) -> &'static str {
    if relative_path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else if relative_path.is_empty() || relative_path.ends_with(".html") {
        "no-cache"
    } else {
        "public, max-age=3600"
    }
}

#[cfg(test)]
mod image_tests {
    use super::*;

    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&w.to_be_bytes());
        bytes.extend_from_slice(&h.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes
    }

    fn gif(w: u16, h: u16) -> Vec<u8> {
        let mut bytes = b"GIF89a".to_vec();
        bytes.extend_from_slice(&w.to_le_bytes());
        bytes.extend_from_slice(&h.to_le_bytes());
        bytes.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]);
        bytes
    }

    fn jpeg(w: u16, h: u16) -> Vec<u8> {
        let mut bytes = vec![0xFF, 0xD8];
        // APP0 segment to exercise segment skipping.
        bytes.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x10]);
        bytes.extend_from_slice(b"JFIF\0");
        bytes.extend_from_slice(&[0; 9]);
        // SOF0 with dimensions.
        bytes.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 0x08]);
        bytes.extend_from_slice(&h.to_be_bytes());
        bytes.extend_from_slice(&w.to_be_bytes());
        bytes.extend_from_slice(&[0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        bytes
    }

    fn webp_vp8x(w: u32, h: u32) -> Vec<u8> {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes.extend_from_slice(b"WEBP");
        bytes.extend_from_slice(b"VP8X");
        bytes.extend_from_slice(&[10, 0, 0, 0]);
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        let w = w - 1;
        let h = h - 1;
        bytes.extend_from_slice(&[w as u8, (w >> 8) as u8, (w >> 16) as u8]);
        bytes.extend_from_slice(&[h as u8, (h >> 8) as u8, (h >> 16) as u8]);
        bytes
    }

    #[test]
    fn dimensions_are_read_from_every_supported_header() {
        assert_eq!(image_dimensions("a.png", &png(640, 480)), Some((640, 480)));
        assert_eq!(image_dimensions("a.gif", &gif(320, 200)), Some((320, 200)));
        assert_eq!(
            image_dimensions("a.jpg", &jpeg(1200, 630)),
            Some((1200, 630))
        );
        assert_eq!(image_dimensions("a.jpeg", &jpeg(10, 20)), Some((10, 20)));
        assert_eq!(
            image_dimensions("a.webp", &webp_vp8x(800, 600)),
            Some((800, 600))
        );
        assert_eq!(image_dimensions("a.png", b"not an image"), None);
        assert_eq!(image_dimensions("a.bmp", &png(1, 1)), None);
    }

    #[test]
    fn bounds_are_enforced_for_every_brand_image() {
        assert!(check_image_bounds("logo", 500, 500).is_ok());
        assert!(check_image_bounds("logo", 32, 32).is_ok());
        assert!(check_image_bounds("moodboard", 320, 240).is_ok());
        let small = message(check_image_bounds("logo", 16, 16).unwrap_err());
        assert!(small.contains("at least 32×32"), "{small}");
        let big = message(check_image_bounds("logo", 501, 501).unwrap_err());
        assert!(big.contains("at most 500×500"), "{big}");
        let wide = message(check_image_bounds("moodboard", 900, 200).unwrap_err());
        assert!(wide.contains("at most 500×500"), "{wide}");

        let unknown = message(check_image_bounds("banner", 400, 400).unwrap_err());
        assert!(unknown.contains("logo or moodboard"), "{unknown}");
    }

    fn message(error: crate::error::ApiError) -> String {
        use crate::error::ApiError;
        match error {
            ApiError::NotFound(m)
            | ApiError::BadRequest(m)
            | ApiError::Unauthorized(m)
            | ApiError::Forbidden(m)
            | ApiError::Conflict(m)
            | ApiError::TooManyRequests(m)
            | ApiError::Internal(m) => m,
        }
    }
}
