//! Shared guard + audit for the opt-in writes: the manage opt-in must be on,
//! the campaign must exist in this workspace's mirror, and a usable Meta user
//! token must be present. Everything is gathered under one short store lock;
//! network I/O always happens after the lock is released.

use crate::ads::{meta_user_token, AUDIT_LIMIT};
use crate::model::AdsAudit;
use crate::AppState;

/// Everything a management call needs, gathered under one short lock.
pub(crate) struct ManageContext {
    pub(crate) oauth: crate::oauth::OauthSettings,
    pub(crate) token: String,
    pub(crate) account_id: String,
}

pub(crate) fn manage_context(
    store: &mut crate::store::Store,
    workspace_id: &str,
) -> Result<ManageContext, String> {
    let workspace = store
        .workspace(workspace_id)
        .ok_or_else(|| "workspace not found".to_string())?;
    if !workspace.ads_manage {
        return Err("turn on campaign management for this workspace first".into());
    }
    let account_id = workspace
        .ads
        .accounts
        .first()
        .map(|a| a.id.clone())
        .ok_or_else(|| "sync Meta Ads first".to_string())?;
    let token = meta_user_token(store, workspace_id).ok_or_else(|| {
        "reconnect Meta to grant access — the stored token is missing".to_string()
    })?;
    Ok(ManageContext {
        oauth: store.oauth.clone(),
        token,
        account_id,
    })
}

/// Verifies the campaign exists in the mirror and belongs to this workspace.
pub(crate) fn known_campaign(
    store: &crate::store::Store,
    workspace_id: &str,
    campaign_id: &str,
) -> bool {
    store
        .workspace(workspace_id)
        .is_some_and(|w| w.ads.campaigns.iter().any(|c| c.id == campaign_id))
}

/// One short lock that verifies the campaign and gathers everything a write
/// needs. Network I/O always happens after this lock is released.
pub(crate) async fn begin_write(
    state: &AppState,
    workspace_id: &str,
    campaign_id: &str,
) -> Result<ManageContext, String> {
    let mut store = state.write().await;
    if !known_campaign(&store, workspace_id, campaign_id) {
        return Err("campaign not found in this workspace's mirror".into());
    }
    manage_context(&mut store, workspace_id)
}

/// Records one management action in the workspace audit trail (newest first).
pub(crate) fn push_audit(
    workspace: &mut crate::model::Workspace,
    action: &str,
    target: &str,
    detail: &str,
    actor: &str,
) {
    workspace.ads.audit.insert(
        0,
        AdsAudit {
            at: crate::store::stamp(),
            actor: actor.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            detail: detail.to_string(),
        },
    );
    workspace.ads.audit.truncate(AUDIT_LIMIT);
}
