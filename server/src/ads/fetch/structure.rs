//! One ad account's structure + last-30-days insights (5 read calls — the
//! Marketing API's development tier allows 60 points per ad account per
//! 300 s window, so reads stay batched per account).

use crate::ads::parse::{parse_ad, parse_adset, parse_campaign, parse_insight, rows, str_of};
use crate::model::{Ad, AdAccount, AdCampaign, AdInsight, AdSet};
use crate::oauth::OauthSettings;

use super::super::graph::graph_get;

const CAMPAIGN_FIELDS: &str = "id,name,objective,status,effective_status,buying_type,daily_budget,lifetime_budget,budget_remaining,bid_strategy,special_ad_categories,start_time,stop_time,created_time";
const ADSET_FIELDS: &str = "id,name,status,effective_status,daily_budget,lifetime_budget,optimization_goal,billing_event,bid_amount,start_time,end_time,promoted_object,targeting";
const AD_FIELDS: &str = "id,name,status,effective_status,creative{id,name,title,body,image_url,thumbnail_url,object_story_id,effective_object_story_id}";
const INSIGHT_FIELDS: &str =
    "campaign_id,spend,impressions,reach,frequency,clicks,ctr,cpc,cpm,actions,purchase_roas";

pub(super) async fn fetch_account_ads(
    oauth: &OauthSettings,
    token: &str,
    account: &AdAccount,
) -> Result<(Vec<AdCampaign>, Vec<AdSet>, Vec<Ad>, Vec<AdInsight>), String> {
    let base = oauth.graph_base.as_str();
    let act = &account.id;

    let campaigns = fetch_campaigns(oauth, token, base, act).await?;
    let adsets = fetch_adsets(oauth, token, base, act).await?;
    let ads = fetch_ads(oauth, token, base, act).await?;
    let insights = fetch_insights(oauth, token, base, act, &campaigns).await;

    Ok((campaigns, adsets, ads, insights))
}

async fn fetch_campaigns(
    oauth: &OauthSettings,
    token: &str,
    base: &str,
    act: &str,
) -> Result<Vec<AdCampaign>, String> {
    let body = graph_get(
        base,
        oauth,
        &format!("/{act}/campaigns"),
        &[("fields", CAMPAIGN_FIELDS), ("limit", "100")],
        token,
    )
    .await?;
    Ok(rows(&body, "data")
        .iter()
        .filter_map(|raw| parse_campaign(raw, act))
        .collect())
}

async fn fetch_adsets(
    oauth: &OauthSettings,
    token: &str,
    base: &str,
    act: &str,
) -> Result<Vec<AdSet>, String> {
    let body = graph_get(
        base,
        oauth,
        &format!("/{act}/adsets"),
        &[("fields", ADSET_FIELDS), ("limit", "200")],
        token,
    )
    .await?;
    Ok(rows(&body, "data")
        .iter()
        .filter_map(|raw| parse_adset(raw, str_of(raw, "campaign_id")))
        .collect())
}

async fn fetch_ads(
    oauth: &OauthSettings,
    token: &str,
    base: &str,
    act: &str,
) -> Result<Vec<Ad>, String> {
    let body = graph_get(
        base,
        oauth,
        &format!("/{act}/ads"),
        &[("fields", AD_FIELDS), ("limit", "200")],
        token,
    )
    .await?;
    Ok(rows(&body, "data")
        .iter()
        .filter_map(|raw| parse_ad(raw, str_of(raw, "adset_id")))
        .collect())
}

/// Insights are the one call that can legitimately fail on its own (no spend
/// yet, unsupported objective); the structure is kept in that case.
async fn fetch_insights(
    oauth: &OauthSettings,
    token: &str,
    base: &str,
    act: &str,
    campaigns: &[AdCampaign],
) -> Vec<AdInsight> {
    let objective_of = |campaign_id: &str| -> String {
        campaigns
            .iter()
            .find(|c| c.id == campaign_id)
            .map(|c| c.objective.clone())
            .unwrap_or_default()
    };
    match graph_get(
        base,
        oauth,
        &format!("/{act}/insights"),
        &[
            ("level", "campaign"),
            ("date_preset", "last_30d"),
            ("fields", INSIGHT_FIELDS),
            ("limit", "200"),
        ],
        token,
    )
    .await
    {
        Ok(body) => rows(&body, "data")
            .iter()
            .map(|raw| parse_insight(raw, &objective_of(str_of(raw, "campaign_id"))))
            .collect(),
        Err(error) => {
            tracing::warn!(account = %act, %error, "ads insights fetch failed");
            Vec::new()
        }
    }
}
