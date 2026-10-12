//! Single-campaign writes: pause/resume/archive, budget edits, and the paused
//! deep-copy duplicate. Each call validates against the workspace mirror,
//! hits Graph once, then patches the mirror + audit trail.

use serde_json::Value;

use crate::AppState;

use super::super::graph::graph_post;
use super::context::{begin_write, push_audit};

/// Sets a campaign's status (ACTIVE | PAUSED | ARCHIVED).
pub async fn set_campaign_status(
    state: &AppState,
    workspace_id: &str,
    campaign_id: &str,
    status: &str,
    actor: &str,
) -> Result<(), String> {
    let status = status.to_uppercase();
    if !["ACTIVE", "PAUSED", "ARCHIVED"].contains(&status.as_str()) {
        return Err("status must be ACTIVE, PAUSED or ARCHIVED".into());
    }
    let ctx = begin_write(state, workspace_id, campaign_id).await?;
    let base = ctx.oauth.graph_base.clone();
    graph_post(
        &base,
        &ctx.oauth,
        &format!("/{campaign_id}"),
        &[("status", status.clone())],
        &ctx.token,
    )
    .await?;
    let mut store = state.write().await;
    if let Some(workspace) = store.workspace_mut(workspace_id) {
        if let Some(campaign) = workspace
            .ads
            .campaigns
            .iter_mut()
            .find(|c| c.id == campaign_id)
        {
            campaign.status = status.clone();
            campaign.effective_status = status.clone();
        }
        push_audit(
            workspace,
            &format!("campaign {}", status.to_lowercase()),
            campaign_id,
            "",
            actor,
        );
    }
    Ok(())
}

/// Updates a campaign budget (minor currency units).
pub async fn set_campaign_budget(
    state: &AppState,
    workspace_id: &str,
    campaign_id: &str,
    daily: Option<u64>,
    lifetime: Option<u64>,
    actor: &str,
) -> Result<(), String> {
    if daily.is_none() && lifetime.is_none() {
        return Err("provide dailyBudget or lifetimeBudget".into());
    }
    if daily == Some(0) || lifetime == Some(0) {
        return Err("budgets must be greater than zero".into());
    }
    let ctx = begin_write(state, workspace_id, campaign_id).await?;
    let mut fields: Vec<(&str, String)> = Vec::new();
    if let Some(v) = daily {
        fields.push(("daily_budget", v.to_string()));
    }
    if let Some(v) = lifetime {
        fields.push(("lifetime_budget", v.to_string()));
    }
    let base = ctx.oauth.graph_base.clone();
    graph_post(
        &base,
        &ctx.oauth,
        &format!("/{campaign_id}"),
        &fields,
        &ctx.token,
    )
    .await?;
    let mut store = state.write().await;
    if let Some(workspace) = store.workspace_mut(workspace_id) {
        if let Some(campaign) = workspace
            .ads
            .campaigns
            .iter_mut()
            .find(|c| c.id == campaign_id)
        {
            if let Some(v) = daily {
                campaign.daily_budget = v;
            }
            if let Some(v) = lifetime {
                campaign.lifetime_budget = v;
            }
        }
        let detail = match (daily, lifetime) {
            (Some(d), Some(l)) => format!("daily {d}, lifetime {l}"),
            (Some(d), None) => format!("daily {d}"),
            (None, Some(l)) => format!("lifetime {l}"),
            (None, None) => String::new(),
        };
        push_audit(workspace, "budget updated", campaign_id, &detail, actor);
    }
    Ok(())
}

/// Duplicates a campaign (deep copy: campaign + ad sets + ads), paused.
pub async fn duplicate_campaign(
    state: &AppState,
    workspace_id: &str,
    campaign_id: &str,
    actor: &str,
) -> Result<String, String> {
    let ctx = begin_write(state, workspace_id, campaign_id).await?;
    let base = ctx.oauth.graph_base.clone();
    let body = graph_post(
        &base,
        &ctx.oauth,
        &format!("/{campaign_id}/copies"),
        &[
            ("deep_copy", "true".into()),
            ("status_option", "PAUSED".into()),
        ],
        &ctx.token,
    )
    .await?;
    let new_id = body
        .get("copied_campaign_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let mut store = state.write().await;
    if let Some(workspace) = store.workspace_mut(workspace_id) {
        push_audit(
            workspace,
            "campaign duplicated",
            campaign_id,
            &format!("copy {new_id} (paused)"),
            actor,
        );
    }
    Ok(new_id)
}
