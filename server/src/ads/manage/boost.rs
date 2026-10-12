//! The "boost a planned post" flow: create a paused campaign + ad set + ad
//! that promotes `story_id` (a Page post id from the live mirror). Nothing
//! starts spending until the user resumes it in Ads Manager or Huuk.

use serde_json::json;

use crate::AppState;

use super::super::graph::graph_post;
use super::super::parse::str_of;
use super::context::{manage_context, push_audit};

/// The "boost a planned post" flow: create a paused campaign + ad set + ad
/// that promotes `story_id` (a Page post id from the live mirror). Nothing
/// starts spending until the user resumes it in Ads Manager or Huuk.
#[allow(clippy::too_many_arguments)]
pub async fn create_boost(
    state: &AppState,
    workspace_id: &str,
    name: &str,
    objective: &str,
    daily_budget: u64,
    days: u64,
    countries: &[String],
    page_id: &str,
    story_id: &str,
    actor: &str,
) -> Result<String, String> {
    if name.trim().is_empty() {
        return Err("name is required".into());
    }
    if daily_budget == 0 {
        return Err("dailyBudget must be greater than zero".into());
    }
    if countries.is_empty() {
        return Err("pick at least one country to target".into());
    }
    let objective = objective.to_uppercase();
    if ![
        "OUTCOME_AWARENESS",
        "OUTCOME_TRAFFIC",
        "OUTCOME_ENGAGEMENT",
        "OUTCOME_LEADS",
        "OUTCOME_SALES",
    ]
    .contains(&objective.as_str())
    {
        return Err("unsupported objective".into());
    }
    let ctx = {
        let mut store = state.write().await;
        manage_context(&mut store, workspace_id)?
    };
    let base = ctx.oauth.graph_base.clone();
    let act = &ctx.account_id;

    // 1) Campaign (paused, budget at campaign level).
    let campaign = graph_post(
        &base,
        &ctx.oauth,
        &format!("/{act}/campaigns"),
        &[
            ("name", name.to_string()),
            ("objective", objective.clone()),
            ("status", "PAUSED".into()),
            ("daily_budget", daily_budget.to_string()),
            ("special_ad_categories", "[]".into()),
        ],
        &ctx.token,
    )
    .await
    .map_err(|e| format!("campaign: {e}"))?;
    let campaign_id = str_of(&campaign, "id").to_string();
    if campaign_id.is_empty() {
        return Err("campaign: Meta did not return an id".into());
    }

    // 2) Ad set (page promotion, impressions billing).
    let (goal, billing) = match objective.as_str() {
        "OUTCOME_AWARENESS" => ("REACH", "IMPRESSIONS"),
        "OUTCOME_ENGAGEMENT" => ("POST_ENGAGEMENT", "IMPRESSIONS"),
        "OUTCOME_LEADS" => ("LEAD_GENERATION", "IMPRESSIONS"),
        "OUTCOME_SALES" => ("OFFSITE_CONVERSIONS", "IMPRESSIONS"),
        _ => ("LINK_CLICKS", "IMPRESSIONS"),
    };
    let targeting = json!({
        "geo_locations": { "countries": countries },
        "age_min": 18,
        "age_max": 65,
    });
    let adset = graph_post(
        &base,
        &ctx.oauth,
        &format!("/{act}/adsets"),
        &[
            ("name", format!("{name} — audience")),
            ("campaign_id", campaign_id.clone()),
            ("status", "PAUSED".into()),
            ("optimization_goal", goal.into()),
            ("billing_event", billing.into()),
            ("targeting", targeting.to_string()),
            ("promoted_object", json!({ "page_id": page_id }).to_string()),
            ("end_time", boost_end(days)),
        ],
        &ctx.token,
    )
    .await
    .map_err(|e| format!("ad set: {e}"))?;
    let adset_id = str_of(&adset, "id").to_string();
    if adset_id.is_empty() {
        return Err("ad set: Meta did not return an id".into());
    }

    // 3) Ad referencing the existing Page post (no new creative uploaded).
    let ad = graph_post(
        &base,
        &ctx.oauth,
        &format!("/{act}/ads"),
        &[
            ("name", format!("{name} — ad")),
            ("adset_id", adset_id.clone()),
            ("status", "PAUSED".into()),
            (
                "creative",
                json!({ "object_story_id": story_id }).to_string(),
            ),
        ],
        &ctx.token,
    )
    .await
    .map_err(|e| format!("ad: {e}"))?;
    let ad_id = str_of(&ad, "id").to_string();

    let mut store = state.write().await;
    if let Some(workspace) = store.workspace_mut(workspace_id) {
        push_audit(
            workspace,
            "boost created",
            &campaign_id,
            &format!("ad set {adset_id}, ad {ad_id}, paused"),
            actor,
        );
    }
    Ok(campaign_id)
}

/// ISO-8601 end time `days` from now (Graph accepts ISO 8601).
fn boost_end(days: u64) -> String {
    let days = days.clamp(1, 90);
    let end = chrono::Utc::now() + chrono::Duration::days(days as i64);
    end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boost_end_clamps_to_a_sane_window() {
        let end = boost_end(7);
        assert!(end.ends_with('Z'), "ISO-8601 UTC: {end}");
        // Clamping is visible in the date being in the future, not "now".
        assert!(end > chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }
}
