//! Workspace CRUD: separate brand identities, each with its own social
//! connections and provider tokens. Requests select a workspace with the
//! `X-Workspace-Id` header; without one the first workspace is used.
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{ApiError, AppJson};
use crate::handlers::{current_account, has_read_access, require_perm};
use crate::model::WorkspaceSummary;
use crate::store::{blank_workspace_for, random_hex, Store};
use crate::AppState;

/// Longest accepted workspace name (characters).
const MAX_WORKSPACE_NAME: usize = 80;
/// Upper bound on workspaces per deployment.
const MAX_WORKSPACES: usize = 50;

fn validate_name(raw: &str) -> Result<String, ApiError> {
    let name = raw.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::bad_request("workspace name is required"));
    }
    if name.chars().count() > MAX_WORKSPACE_NAME {
        return Err(ApiError::bad_request("workspace name is too long (max 80)"));
    }
    Ok(name)
}

/// `GET /api/workspaces` — the account's workspaces with connection counts.
/// Accounts only ever see their own; the operator token sees all.
pub async fn list(State(s): State<AppState>, headers: HeaderMap) -> Json<Vec<WorkspaceSummary>> {
    let store = s.read().await;
    let account = current_account(&store, &headers);
    Json(
        store
            .visible_workspaces(account.as_deref())
            .into_iter()
            .map(|w| w.summary(account.as_deref()))
            .collect(),
    )
}

#[derive(Deserialize)]
pub struct WorkspaceBody {
    pub name: String,
}

/// `POST /api/workspaces` — creates a workspace owned by the signed-in
/// account, with a blank brand and one disconnected slot per platform.
pub async fn create(
    State(s): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<WorkspaceBody>,
) -> Result<Json<WorkspaceSummary>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "platforms.manage")?;
    let name = validate_name(&body.name)?;
    let account = current_account(&store, &headers);
    if store.visible_workspaces(account.as_deref()).len() >= MAX_WORKSPACES {
        return Err(ApiError::bad_request("too many workspaces (max 50)"));
    }
    let owner = account.as_deref().unwrap_or_default();
    let workspace = blank_workspace_for(&format!("ws-{}", random_hex(6)), &name, owner);
    let summary = workspace.summary(account.as_deref());
    store.workspaces.push(workspace);
    tracing::info!(workspace = %summary.id, name = %summary.name, owner = %owner, "workspace created");
    Ok(Json(summary))
}

/// `PATCH /api/workspaces/{id}` — renames a workspace the account owns.
pub async fn rename(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<WorkspaceBody>,
) -> Result<Json<WorkspaceSummary>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "platforms.manage")?;
    let name = validate_name(&body.name)?;
    let account = current_account(&store, &headers);
    let workspace = store
        .workspaces
        .iter_mut()
        .find(|w| w.id == id && w.visible_to(account.as_deref()))
        .ok_or_else(|| ApiError::not_found("workspace not found"))?;
    workspace.name = name;
    Ok(Json(workspace.summary(account.as_deref())))
}

/// `DELETE /api/workspaces/{id}` — deletes a workspace and everything scoped to
/// it. The last workspace can never be deleted.
pub async fn remove(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let mut store = s.write().await;
    require_perm(&mut store, &headers, "platforms.manage")?;
    let account = current_account(&store, &headers);
    let Some(position) = store
        .workspaces
        .iter()
        .position(|w| w.id == id && w.visible_to(account.as_deref()))
    else {
        return Err(ApiError::not_found("workspace not found"));
    };
    if store.visible_workspaces(account.as_deref()).len() <= 1 {
        return Err(ApiError::bad_request(
            "the last workspace cannot be deleted — create another one first",
        ));
    }
    store.workspaces.remove(position);
    // Drop everything ephemeral that belonged to the workspace.
    store.clear_workspace_tokens(&id);
    store.oauth_picks.retain(|_, pick| pick.workspace_id != id);
    store
        .oauth_states
        .retain(|_, state| state.workspace_id != id);
    let fallback = store
        .visible_workspaces(account.as_deref())
        .first()
        .map(|w| w.id.clone())
        .unwrap_or_default();
    tracing::info!(workspace = %id, "workspace deleted");
    Ok(Json(json!({ "deleted": id, "fallback": fallback })))
}

// ---- members ----

/// One account on a workspace's member list.
#[derive(Debug, Clone, Serialize)]
pub struct MemberView {
    pub email: String,
    pub name: String,
    pub role: String,
}

/// The workspace owner as shown above the member list.
#[derive(Debug, Clone, Serialize)]
pub struct OwnerView {
    pub email: String,
    pub name: String,
}

/// Wire shape of the member endpoints.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceMembers {
    pub owner: OwnerView,
    pub members: Vec<MemberView>,
}

#[derive(Deserialize)]
pub struct MemberBody {
    pub email: String,
}

/// The display name behind an account email (blank for unknown accounts).
fn account_name(store: &Store, email: &str) -> String {
    store
        .accounts
        .get(email)
        .map(|a| a.name.clone())
        .unwrap_or_default()
}

/// The directory role of a display name, `Viewer` when the user is gone.
fn role_for(store: &Store, name: &str) -> String {
    store
        .setup
        .users
        .iter()
        .find(|u| u.name == name)
        .map(|u| u.role.clone())
        .unwrap_or_else(|| "Viewer".to_string())
}

fn members_view(store: &Store, id: &str) -> Result<WorkspaceMembers, ApiError> {
    let workspace = store
        .workspace(id)
        .ok_or_else(|| ApiError::not_found("workspace not found"))?;
    Ok(WorkspaceMembers {
        owner: OwnerView {
            email: workspace.owner.clone(),
            name: account_name(store, &workspace.owner),
        },
        members: workspace
            .members
            .iter()
            .map(|email| {
                let name = account_name(store, email);
                MemberView {
                    email: email.clone(),
                    role: role_for(store, &name),
                    name,
                }
            })
            .collect(),
    })
}

/// Owner gate for the member endpoints. Demo mode / the operator token
/// (`None`) and legacy ownerless workspaces pass; otherwise the caller must be
/// the owner — members cannot manage the list. Anonymous callers are rejected
/// before that, since `current_account` also yields `None` without a session.
fn ensure_owner(store: &Store, headers: &HeaderMap, id: &str) -> Result<(), ApiError> {
    let workspace = store
        .workspace(id)
        .ok_or_else(|| ApiError::not_found("workspace not found"))?;
    if !has_read_access(store, headers) {
        return Err(ApiError::unauthorized("authentication required"));
    }
    match current_account(store, headers) {
        Some(email)
            if !workspace.owner.is_empty() && !workspace.owner.eq_ignore_ascii_case(&email) =>
        {
            Err(ApiError::forbidden(
                "only the workspace owner can manage members",
            ))
        }
        _ => Ok(()),
    }
}

/// Trims and lowercases an invited email; rejects malformed values early.
fn normalize_email(raw: &str) -> Result<String, ApiError> {
    let email = raw.trim().to_ascii_lowercase();
    if !email.contains('@') || email.chars().any(char::is_whitespace) {
        return Err(ApiError::bad_request("a valid email address is required"));
    }
    Ok(email)
}

/// `GET /api/workspaces/{id}/members` — the owner plus the added accounts.
pub async fn list_members(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<WorkspaceMembers>, ApiError> {
    let store = s.read().await;
    ensure_owner(&store, &headers, &id)?;
    Ok(Json(members_view(&store, &id)?))
}

/// `POST /api/workspaces/{id}/members` — adds a registered account.
pub async fn add_member(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    AppJson(body): AppJson<MemberBody>,
) -> Result<Json<WorkspaceMembers>, ApiError> {
    let mut store = s.write().await;
    ensure_owner(&store, &headers, &id)?;
    let email = normalize_email(&body.email)?;
    if !store.accounts.contains_key(&email) {
        return Err(ApiError::not_found("no account with that email"));
    }
    {
        let workspace = store
            .workspace_mut(&id)
            .ok_or_else(|| ApiError::not_found("workspace not found"))?;
        if workspace.owner.eq_ignore_ascii_case(&email)
            || workspace
                .members
                .iter()
                .any(|m| m.eq_ignore_ascii_case(&email))
        {
            return Err(ApiError::conflict(
                "that account already owns or belongs to this workspace",
            ));
        }
        workspace.members.push(email);
    }
    tracing::info!(workspace = %id, "workspace member added");
    Ok(Json(members_view(&store, &id)?))
}

/// `DELETE /api/workspaces/{id}/members/{email}` — removes a member.
pub async fn remove_member(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path((id, email)): Path<(String, String)>,
) -> Result<Json<WorkspaceMembers>, ApiError> {
    let mut store = s.write().await;
    ensure_owner(&store, &headers, &id)?;
    let email = email.trim().to_ascii_lowercase();
    {
        let workspace = store
            .workspace_mut(&id)
            .ok_or_else(|| ApiError::not_found("workspace not found"))?;
        let before = workspace.members.len();
        workspace
            .members
            .retain(|m| !m.eq_ignore_ascii_case(&email));
        if workspace.members.len() == before {
            return Err(ApiError::not_found(
                "that account is not a member of this workspace",
            ));
        }
    }
    tracing::info!(workspace = %id, "workspace member removed");
    Ok(Json(members_view(&store, &id)?))
}
