//! Meta Ads mirror: read the ad accounts, campaigns, ad sets, ads and their
//! settings + last-30-days performance into the workspace, and (only after an
//! explicit opt-in) change campaigns through the same login.
//!
//! Rules (same as `crate::live`):
//! - The store lock is never held across network I/O.
//! - A failed refresh never wipes the previous mirror: it records
//!   [`AdsData::error`] and leaves the data in place.
//! - Everything is best-effort: a metric the API does not report stays `0`.
//! - Reads are batched per ad account (5 calls) because the Marketing API's
//!   development tier allows only 60 points per ad account per 300 s window.
//! - Writes are 3 points each and always appended to [`AdsData::audit`].
use std::time::Duration;

use serde_json::{json, Value};

use crate::model::{Ad, AdAccount, AdCampaign, AdInsight, AdSet, AdsAudit, AdsData};
use crate::oauth::OauthSettings;
use crate::AppState;

/// Structure kept per workspace (bounds the snapshot).
const KEEP_ACCOUNTS: usize = 5;
const KEEP_CAMPAIGNS: usize = 200;
const KEEP_ADSETS: usize = 500;
const KEEP_ADS: usize = 1000;
/// Audit entries kept (newest first).
pub const AUDIT_LIMIT: usize = 50;

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

/// `POST {GRAPH}{path}` with form fields + token. Returns the response body.
async fn graph_post(
    base: &str,
    oauth: &OauthSettings,
    path: &str,
    fields: &[(&str, String)],
    token: &str,
) -> Result<Value, String> {
    let form: Vec<(&str, &str)> = fields
        .iter()
        .map(|(k, v)| (*k, v.as_str()))
        .chain(std::iter::once(("access_token", token)))
        .collect();
    let res = oauth
        .http()
        .post(format!("{base}{path}"))
        .form(&form)
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
                .or_else(|| v.as_f64().map(|f| f.max(0.0) as u64))
        })
        .unwrap_or(0)
}

fn as_f64(value: Option<&Value>) -> f64 {
    value
        .and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
        })
        .unwrap_or(0.0)
}

fn str_of<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn rows<'a>(body: &'a Value, key: &str) -> &'a [Value] {
    body.get(key)
        .and_then(Value::as_array)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

/// Graph `account_status` codes -> readable words.
fn account_status(code: u64) -> String {
    match code {
        1 => "active",
        2 => "disabled",
        3 => "unsettled",
        7 => "pending_risk_review",
        8 => "pending_settlement",
        9 => "in_grace_period",
        100 => "pending_closure",
        101 => "closed",
        _ => "unknown",
    }
    .to_string()
}

/// Ad account row from `/me/adaccounts`.
pub fn parse_ad_account(raw: &Value) -> Option<AdAccount> {
    let id = str_of(raw, "id");
    if id.is_empty() {
        return None;
    }
    let business = raw
        .get("business")
        .and_then(|b| b.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    Some(AdAccount {
        id: id.to_string(),
        name: str_of(raw, "name").to_string(),
        status: account_status(as_u64(raw.get("account_status"))),
        currency: str_of(raw, "currency").to_string(),
        timezone: str_of(raw, "timezone_name").to_string(),
        business: business.to_string(),
    })
}

/// Campaign row from `/act_{id}/campaigns`.
pub fn parse_campaign(raw: &Value, account_id: &str) -> Option<AdCampaign> {
    let id = str_of(raw, "id");
    if id.is_empty() {
        return None;
    }
    let special: Vec<String> = raw
        .get("special_ad_categories")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Some(AdCampaign {
        id: id.to_string(),
        account_id: account_id.to_string(),
        name: str_of(raw, "name").to_string(),
        objective: str_of(raw, "objective").to_string(),
        status: str_of(raw, "status").to_string(),
        effective_status: str_of(raw, "effective_status").to_string(),
        buying_type: str_of(raw, "buying_type").to_string(),
        daily_budget: as_u64(raw.get("daily_budget")),
        lifetime_budget: as_u64(raw.get("lifetime_budget")),
        budget_remaining: as_u64(raw.get("budget_remaining")),
        bid_strategy: str_of(raw, "bid_strategy").to_string(),
        special_ad_categories: special,
        start_time: str_of(raw, "start_time").to_string(),
        stop_time: str_of(raw, "stop_time").to_string(),
        created_time: str_of(raw, "created_time").to_string(),
    })
}

/// Human summary of an ad set's targeting (never the raw blob).
pub fn summarize_targeting(raw: &Value) -> String {
    let Some(t) = raw.get("targeting") else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    let age_min = as_u64(t.get("age_min"));
    let age_max = as_u64(t.get("age_max"));
    if age_min > 0 || age_max > 0 {
        parts.push(format!("{age_min}-{age_max}"));
    }
    if let Some(genders) = t.get("genders").and_then(Value::as_array) {
        let names: Vec<&str> = genders
            .iter()
            .filter_map(Value::as_u64)
            .map(|g| match g {
                1 => "men",
                2 => "women",
                _ => "all",
            })
            .collect();
        if !names.is_empty() {
            parts.push(names.join("/"));
        }
    }
    if let Some(geo) = t.get("geo_locations") {
        let mut places: Vec<String> = Vec::new();
        for key in ["countries", "regions", "cities"] {
            if let Some(list) = geo.get(key).and_then(Value::as_array) {
                for item in list {
                    if let Some(s) = item.as_str() {
                        places.push(s.to_string());
                    } else if let Some(name) = item.get("name").and_then(Value::as_str) {
                        places.push(name.to_string());
                    }
                }
            }
        }
        if !places.is_empty() {
            parts.push(places.join(", "));
        }
    }
    let interests = t
        .get("interests")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    if interests > 0 {
        parts.push(format!("{interests} interests"));
    }
    let custom = t
        .get("custom_audiences")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    if custom > 0 {
        parts.push(format!("{custom} custom audiences"));
    }
    parts.join(" · ")
}

fn summarize_promoted(raw: &Value) -> String {
    let Some(p) = raw.get("promoted_object") else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    if let Some(id) = p.get("page_id").and_then(Value::as_str) {
        parts.push(format!("page {id}"));
    }
    if let Some(id) = p.get("pixel_id").and_then(Value::as_str) {
        parts.push(format!("pixel {id}"));
    }
    if let Some(id) = p.get("application_id").and_then(Value::as_str) {
        parts.push(format!("app {id}"));
    }
    parts.join(", ")
}

/// Ad set row from `/act_{id}/adsets`.
pub fn parse_adset(raw: &Value, campaign_id: &str) -> Option<AdSet> {
    let id = str_of(raw, "id");
    if id.is_empty() {
        return None;
    }
    Some(AdSet {
        id: id.to_string(),
        campaign_id: campaign_id.to_string(),
        name: str_of(raw, "name").to_string(),
        status: str_of(raw, "status").to_string(),
        effective_status: str_of(raw, "effective_status").to_string(),
        daily_budget: as_u64(raw.get("daily_budget")),
        lifetime_budget: as_u64(raw.get("lifetime_budget")),
        optimization_goal: str_of(raw, "optimization_goal").to_string(),
        billing_event: str_of(raw, "billing_event").to_string(),
        bid_amount: as_u64(raw.get("bid_amount")),
        start_time: str_of(raw, "start_time").to_string(),
        end_time: str_of(raw, "end_time").to_string(),
        targeting: summarize_targeting(raw),
        promoted_object: summarize_promoted(raw),
    })
}

/// Ad row from `/act_{id}/ads` (creative flattened).
pub fn parse_ad(raw: &Value, adset_id: &str) -> Option<Ad> {
    let id = str_of(raw, "id");
    if id.is_empty() {
        return None;
    }
    let creative = raw.get("creative").cloned().unwrap_or(Value::Null);
    Some(Ad {
        id: id.to_string(),
        adset_id: adset_id.to_string(),
        name: str_of(raw, "name").to_string(),
        status: str_of(raw, "status").to_string(),
        effective_status: str_of(raw, "effective_status").to_string(),
        creative_title: str_of(&creative, "title").to_string(),
        creative_body: str_of(&creative, "body").to_string(),
        image_url: str_of(&creative, "image_url").to_string(),
        thumbnail_url: str_of(&creative, "thumbnail_url").to_string(),
        story_id: {
            let story = str_of(&creative, "effective_object_story_id");
            if story.is_empty() {
                str_of(&creative, "object_story_id").to_string()
            } else {
                story.to_string()
            }
        },
        preview_url: {
            // A shareable preview is nicer than a raw image for the drawer.
            let thumb = str_of(&creative, "thumbnail_url");
            if thumb.is_empty() {
                str_of(&creative, "image_url").to_string()
            } else {
                thumb.to_string()
            }
        },
    })
}

/// Result metric priority per objective: the first action present wins.
fn result_action(objective: &str) -> (&'static str, &'static str) {
    match objective {
        "OUTCOME_LEADS" => ("lead", "Leads"),
        "OUTCOME_SALES" => ("purchase", "Purchases"),
        "OUTCOME_APP_PROMOTION" => ("app_install", "App installs"),
        "OUTCOME_AWARENESS" => ("post_engagement", "Engagements"),
        "OUTCOME_ENGAGEMENT" => ("post_engagement", "Engagements"),
        "OUTCOME_TRAFFIC" => ("link_click", "Link clicks"),
        _ => ("link_click", "Link clicks"),
    }
}

/// Last-30-days insight row from `/act_{id}/insights?level=campaign`.
pub fn parse_insight(raw: &Value, objective: &str) -> AdInsight {
    let campaign_id = str_of(raw, "campaign_id").to_string();
    let actions = raw.get("actions").and_then(Value::as_array);
    let (key, label) = result_action(objective);
    let results = actions
        .and_then(|rows| rows.iter().find(|row| str_of(row, "action_type") == key))
        .map(|row| as_u64(row.get("value")))
        .unwrap_or(0);
    let roas = raw
        .get("purchase_roas")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .map(|row| as_f64(row.get("value")))
        .unwrap_or(0.0);
    AdInsight {
        campaign_id,
        spend: as_f64(raw.get("spend")),
        impressions: as_u64(raw.get("impressions")),
        reach: as_u64(raw.get("reach")),
        frequency: as_f64(raw.get("frequency")),
        clicks: as_u64(raw.get("clicks")),
        ctr: as_f64(raw.get("ctr")),
        cpc: as_f64(raw.get("cpc")),
        cpm: as_f64(raw.get("cpm")),
        results,
        result_label: label.to_string(),
        roas,
    }
}

/// Turns a permission error into reconnect guidance (the UI shows this text).
pub fn permission_hint(error: &str) -> Option<String> {
    let lower = error.to_lowercase();
    if lower.contains("ads_read")
        || lower.contains("ads_management")
        || lower.contains("permission")
        || lower.contains("does not have permission")
    {
        return Some("reconnect Meta to grant ads access".into());
    }
    None
}

const ACCOUNT_FIELDS: &str =
    "id,account_id,name,account_status,currency,timezone_name,business{id,name}";
const CAMPAIGN_FIELDS: &str = "id,name,objective,status,effective_status,buying_type,daily_budget,lifetime_budget,budget_remaining,bid_strategy,special_ad_categories,start_time,stop_time,created_time";
const ADSET_FIELDS: &str = "id,name,status,effective_status,daily_budget,lifetime_budget,optimization_goal,billing_event,bid_amount,start_time,end_time,promoted_object,targeting";
const AD_FIELDS: &str = "id,name,status,effective_status,creative{id,name,title,body,image_url,thumbnail_url,object_story_id,effective_object_story_id}";
const INSIGHT_FIELDS: &str =
    "campaign_id,spend,impressions,reach,frequency,clicks,ctr,cpc,cpm,actions,purchase_roas";

/// Fetches every ad account the login can see (with the Business-portfolio
/// fallback that `/me/accounts` needs for Pages, mirrored here).
pub async fn fetch_ad_accounts(
    oauth: &OauthSettings,
    token: &str,
) -> Result<Vec<AdAccount>, String> {
    let base = oauth.graph_base.as_str();
    let body = graph_get(
        base,
        oauth,
        "/me/adaccounts",
        &[("fields", ACCOUNT_FIELDS), ("limit", "50")],
        token,
    )
    .await?;
    let mut accounts: Vec<AdAccount> = rows(&body, "data")
        .iter()
        .filter_map(parse_ad_account)
        .collect();

    if accounts.is_empty() {
        // Business-owned accounts are omitted from /me/adaccounts without this
        // fallback (the same limitation we hit with Pages).
        if let Ok(businesses) = graph_get(
            base,
            oauth,
            "/me/businesses",
            &[
                (
                    "fields",
                    "id,name,owned_ad_accounts{id,name,account_status,currency,timezone_name,business{id,name}},client_ad_accounts{id,name,account_status,currency,timezone_name,business{id,name}}",
                ),
                ("limit", "50"),
            ],
            token,
        )
        .await
        {
            for business in rows(&businesses, "data") {
                for key in ["owned_ad_accounts", "client_ad_accounts"] {
                    if let Some(list) = business.get(key).and_then(|v| v.get("data")) {
                        if let Some(list) = list.as_array() {
                            accounts.extend(list.iter().filter_map(parse_ad_account));
                        }
                    }
                }
            }
            accounts.sort_by(|a, b| a.id.cmp(&b.id));
            accounts.dedup_by(|a, b| a.id == b.id);
        }
    }
    accounts.truncate(KEEP_ACCOUNTS);
    Ok(accounts)
}

/// One ad account's structure + last-30-days insights (5 read calls).
async fn fetch_account_ads(
    oauth: &OauthSettings,
    token: &str,
    account: &AdAccount,
) -> Result<(Vec<AdCampaign>, Vec<AdSet>, Vec<Ad>, Vec<AdInsight>), String> {
    let base = oauth.graph_base.as_str();
    let act = &account.id;

    let campaigns_body = graph_get(
        base,
        oauth,
        &format!("/{act}/campaigns"),
        &[("fields", CAMPAIGN_FIELDS), ("limit", "100")],
        token,
    )
    .await?;
    let campaigns: Vec<AdCampaign> = rows(&campaigns_body, "data")
        .iter()
        .filter_map(|raw| parse_campaign(raw, act))
        .collect();

    let adsets_body = graph_get(
        base,
        oauth,
        &format!("/{act}/adsets"),
        &[("fields", ADSET_FIELDS), ("limit", "200")],
        token,
    )
    .await?;
    let adsets: Vec<AdSet> = rows(&adsets_body, "data")
        .iter()
        .filter_map(|raw| parse_adset(raw, str_of(raw, "campaign_id")))
        .collect();

    let ads_body = graph_get(
        base,
        oauth,
        &format!("/{act}/ads"),
        &[("fields", AD_FIELDS), ("limit", "200")],
        token,
    )
    .await?;
    let ads: Vec<Ad> = rows(&ads_body, "data")
        .iter()
        .filter_map(|raw| parse_ad(raw, str_of(raw, "adset_id")))
        .collect();

    // Insights are the one call that can legitimately fail on its own (no
    // spend yet, unsupported objective); keep the structure in that case.
    let objective_of = |campaign_id: &str| -> String {
        campaigns
            .iter()
            .find(|c| c.id == campaign_id)
            .map(|c| c.objective.clone())
            .unwrap_or_default()
    };
    let insights = match graph_get(
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
    };

    Ok((campaigns, adsets, ads, insights))
}

/// Fetches the whole ads mirror for one workspace's login.
pub async fn fetch_meta_ads(oauth: &OauthSettings, token: &str) -> Result<AdsData, String> {
    let fetched_at = now_secs();
    let accounts = fetch_ad_accounts(oauth, token)
        .await
        .map_err(|error| permission_hint(&error).unwrap_or(error))?;

    let mut campaigns: Vec<AdCampaign> = Vec::new();
    let mut adsets: Vec<AdSet> = Vec::new();
    let mut ads: Vec<Ad> = Vec::new();
    let mut insights: Vec<AdInsight> = Vec::new();
    for account in &accounts {
        match fetch_account_ads(oauth, token, account).await {
            Ok((mut c, mut s, mut a, mut i)) => {
                campaigns.append(&mut c);
                adsets.append(&mut s);
                ads.append(&mut a);
                insights.append(&mut i);
            }
            Err(error) => {
                // One account failing must not blank the others.
                tracing::warn!(account = %account.id, %error, "ads fetch failed for account");
                if let Some(hint) = permission_hint(&error) {
                    return Err(hint);
                }
            }
        }
    }

    campaigns.sort_by(|a, b| b.created_time.cmp(&a.created_time));
    campaigns.truncate(KEEP_CAMPAIGNS);
    adsets.truncate(KEEP_ADSETS);
    ads.truncate(KEEP_ADS);
    Ok(AdsData {
        fetched_at: Some(fetched_at),
        accounts,
        campaigns,
        adsets,
        ads,
        insights,
        audit: Vec::new(),
        error: String::new(),
    })
}

/// Meta Ads endpoints need the *user* token: the Page token that mirrors Page
/// content cannot list ad accounts. Older connections stored only one token
/// (the user token) under the plain key, so fall back to it.
fn meta_user_token(store: &mut crate::store::Store, workspace_id: &str) -> Option<String> {
    store
        .platform_token(workspace_id, "meta#user")
        .or_else(|| store.platform_token(workspace_id, "meta"))
}

/// Refreshes one workspace's ads mirror. Never holds the store lock across
/// network I/O; keeps the audit trail and the manage opt-in intact.
pub async fn refresh_workspace_ads(
    state: &AppState,
    workspace_id: &str,
) -> Result<AdsData, String> {
    let (oauth, token) = {
        let mut store = state.write().await;
        if store.workspace(workspace_id).is_none() {
            return Err("workspace not found".into());
        }
        let token = meta_user_token(&mut store, workspace_id);
        (store.oauth.clone(), token)
    };
    let Some(token) = token else {
        return Err("reconnect Meta to grant access — the stored token is missing".into());
    };

    match fetch_meta_ads(&oauth, &token).await {
        Ok(mut ads) => {
            let mut store = state.write().await;
            if let Some(workspace) = store.workspace_mut(workspace_id) {
                // Preserve what the mirror must never lose on a refresh.
                ads.audit = std::mem::take(&mut workspace.ads.audit);
                workspace.ads = ads.clone();
            }
            tracing::info!(
                workspace = %workspace_id,
                accounts = ads.accounts.len(),
                campaigns = ads.campaigns.len(),
                "ads mirror refreshed"
            );
            Ok(ads)
        }
        Err(error) => {
            tracing::warn!(workspace = %workspace_id, %error, "ads refresh failed");
            let mut store = state.write().await;
            if let Some(workspace) = store.workspace_mut(workspace_id) {
                workspace.ads.error = error.clone();
            }
            Err(error)
        }
    }
}

/// Records one management action in the workspace audit trail (newest first).
fn push_audit(
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

/// Everything a management call needs, gathered under one short lock.
struct ManageContext {
    oauth: OauthSettings,
    token: String,
    account_id: String,
}

fn manage_context(
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
fn known_campaign(store: &crate::store::Store, workspace_id: &str, campaign_id: &str) -> bool {
    store
        .workspace(workspace_id)
        .is_some_and(|w| w.ads.campaigns.iter().any(|c| c.id == campaign_id))
}

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
    let ctx = {
        let mut store = state.write().await;
        if !known_campaign(&store, workspace_id, campaign_id) {
            return Err("campaign not found in this workspace's mirror".into());
        }
        manage_context(&mut store, workspace_id)?
    };
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
    let ctx = {
        let mut store = state.write().await;
        if !known_campaign(&store, workspace_id, campaign_id) {
            return Err("campaign not found in this workspace's mirror".into());
        }
        manage_context(&mut store, workspace_id)?
    };
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
    let ctx = {
        let mut store = state.write().await;
        if !known_campaign(&store, workspace_id, campaign_id) {
            return Err("campaign not found in this workspace's mirror".into());
        }
        manage_context(&mut store, workspace_id)?
    };
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

/// Background ads refresh: every `ADS_REFRESH_MIN` minutes (default 60, 0
/// disables), for workspaces with a usable Meta token.
pub fn spawn_ads_loop(state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let minutes = std::env::var("ADS_REFRESH_MIN")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(60);
        if minutes == 0 {
            tracing::info!("ads refresh loop disabled (ADS_REFRESH_MIN=0)");
            return;
        }
        // Stagger behind the live loop so both do not hit Graph together.
        tokio::time::sleep(Duration::from_secs(45)).await;
        let mut ticker = tokio::time::interval(Duration::from_secs(minutes * 60));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            let workspace_ids: Vec<String> = {
                let mut store = state.write().await;
                store
                    .workspaces
                    .iter()
                    .filter(|w| {
                        w.connections
                            .iter()
                            .any(|c| c.id == "meta" && c.status == "connected")
                    })
                    .map(|w| w.id.clone())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .filter(|id| meta_user_token(&mut store, id).is_some())
                    .collect()
            };
            for id in workspace_ids {
                let _ = refresh_workspace_ads(&state, &id).await;
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ad_account_decodes_status_and_business() {
        let raw = json!({
            "id": "act_123", "name": "Meni Ads", "account_status": 1,
            "currency": "THB", "timezone_name": "Asia/Bangkok",
            "business": { "id": "1", "name": "Meni Co" }
        });
        let account = parse_ad_account(&raw).unwrap();
        assert_eq!(account.id, "act_123");
        assert_eq!(account.status, "active");
        assert_eq!(account.currency, "THB");
        assert_eq!(account.business, "Meni Co");
    }

    #[test]
    fn campaign_parses_settings_and_budgets() {
        let raw = json!({
            "id": "c1", "name": "Spring", "objective": "OUTCOME_TRAFFIC",
            "status": "ACTIVE", "effective_status": "ACTIVE",
            "daily_budget": "50000", "lifetime_budget": 0,
            "budget_remaining": "42000", "bid_strategy": "LOWEST_COST_WITHOUT_CAP",
            "special_ad_categories": ["NONE"], "start_time": "2026-09-01T00:00:00+0000"
        });
        let campaign = parse_campaign(&raw, "act_123").unwrap();
        assert_eq!(campaign.account_id, "act_123");
        assert_eq!(campaign.daily_budget, 50000, "string budgets parse");
        assert_eq!(campaign.special_ad_categories, vec!["NONE"]);
    }

    #[test]
    fn adset_targeting_summarizes_never_dumps_the_blob() {
        let raw = json!({
            "id": "s1", "name": "Audience", "status": "ACTIVE",
            "optimization_goal": "LINK_CLICKS", "billing_event": "IMPRESSIONS",
            "promoted_object": { "page_id": "p1", "pixel_id": "px1" },
            "targeting": {
                "age_min": 25, "age_max": 45, "genders": [1],
                "geo_locations": { "countries": ["TH"] },
                "interests": [{ "id": "1", "name": "Coffee" }, { "id": "2", "name": "Tea" }]
            }
        });
        let adset = parse_adset(&raw, "c1").unwrap();
        assert_eq!(adset.campaign_id, "c1");
        assert_eq!(adset.targeting, "25-45 · men · TH · 2 interests");
        assert_eq!(adset.promoted_object, "page p1, pixel px1");
    }

    #[test]
    fn ad_flattens_creative_and_prefers_effective_story() {
        let raw = json!({
            "id": "a1", "name": "Ad", "status": "PAUSED",
            "creative": {
                "title": "Hello", "body": "Body text",
                "image_url": "https://img.example/i.jpg",
                "thumbnail_url": "https://img.example/t.jpg",
                "object_story_id": "p1_1", "effective_object_story_id": "p1_2"
            }
        });
        let ad = parse_ad(&raw, "s1").unwrap();
        assert_eq!(ad.adset_id, "s1");
        assert_eq!(ad.creative_title, "Hello");
        assert_eq!(ad.story_id, "p1_2");
        assert_eq!(ad.preview_url, "https://img.example/t.jpg");
    }

    #[test]
    fn insight_picks_the_result_action_for_the_objective() {
        let raw = json!({
            "campaign_id": "c1", "spend": "123.45", "impressions": "1000",
            "reach": "800", "frequency": "1.25", "clicks": "40",
            "ctr": "4.0", "cpc": "3.09", "cpm": "123.45",
            "actions": [
                { "action_type": "link_click", "value": "33" },
                { "action_type": "landing_page_view", "value": "20" }
            ],
            "purchase_roas": [{ "action_type": "omni_purchase", "value": "3.2" }]
        });
        let insight = parse_insight(&raw, "OUTCOME_TRAFFIC");
        assert_eq!(insight.spend, 123.45);
        assert_eq!(insight.results, 33);
        assert_eq!(insight.result_label, "Link clicks");
        assert_eq!(insight.roas, 3.2);

        let leads = parse_insight(
            &json!({
                "campaign_id": "c2",
                "actions": [{ "action_type": "lead", "value": 7 }]
            }),
            "OUTCOME_LEADS",
        );
        assert_eq!(leads.results, 7);
        assert_eq!(leads.result_label, "Leads");
    }

    #[test]
    fn permission_errors_become_reconnect_guidance() {
        let error = "graph error: (#200) Requires ads_read permission";
        assert_eq!(
            permission_hint(error).as_deref(),
            Some("reconnect Meta to grant ads access")
        );
        assert!(permission_hint("graph error: rate limit").is_none());
    }

    /// Fake Graph with one ad account, one campaign, one ad set, one ad, and a
    /// campaign-level insight row.
    async fn fake_graph() -> String {
        use axum::{routing::get, Json, Router};
        let app = Router::new()
            .route(
                "/me/adaccounts",
                get(|| async {
                    Json(json!({ "data": [{
                        "id": "act_1", "name": "Meni Ads", "account_status": 1,
                        "currency": "THB", "timezone_name": "Asia/Bangkok"
                    }]}))
                }),
            )
            .route(
                "/act_1/campaigns",
                get(|| async {
                    Json(json!({ "data": [{
                        "id": "c1", "name": "Spring", "objective": "OUTCOME_TRAFFIC",
                        "status": "ACTIVE", "daily_budget": "50000",
                        "created_time": "2026-09-01T00:00:00+0000"
                    }]}))
                }),
            )
            .route(
                "/act_1/adsets",
                get(|| async {
                    Json(json!({ "data": [{
                        "id": "s1", "campaign_id": "c1", "name": "Audience",
                        "status": "ACTIVE", "optimization_goal": "LINK_CLICKS",
                        "billing_event": "IMPRESSIONS",
                        "targeting": { "age_min": 25, "age_max": 45, "geo_locations": { "countries": ["TH"] } }
                    }]}))
                }),
            )
            .route(
                "/act_1/ads",
                get(|| async {
                    Json(json!({ "data": [{
                        "id": "a1", "adset_id": "s1", "name": "Ad", "status": "ACTIVE",
                        "creative": { "title": "Hi", "thumbnail_url": "https://img.example/t.jpg", "effective_object_story_id": "p1_9" }
                    }]}))
                }),
            )
            .route(
                "/act_1/insights",
                get(|| async {
                    Json(json!({ "data": [{
                        "campaign_id": "c1", "spend": "100", "impressions": "500",
                        "clicks": "25", "actions": [{ "action_type": "link_click", "value": "20" }]
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

    #[tokio::test]
    async fn fetch_meta_ads_pulls_structure_settings_and_insights() {
        let base = fake_graph().await;
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&base);
        let ads = fetch_meta_ads(&oauth, "token").await.unwrap();

        assert_eq!(ads.accounts.len(), 1);
        assert_eq!(ads.campaigns.len(), 1);
        assert_eq!(ads.campaigns[0].daily_budget, 50000);
        assert_eq!(ads.adsets.len(), 1);
        assert_eq!(ads.adsets[0].targeting, "25-45 · TH");
        assert_eq!(ads.ads.len(), 1);
        assert_eq!(ads.ads[0].story_id, "p1_9");
        assert_eq!(ads.insights.len(), 1);
        assert_eq!(ads.insights[0].results, 20, "link_click result");
        assert!(ads.fetched_at.is_some());
        assert!(ads.error.is_empty());
    }

    #[tokio::test]
    async fn fetch_meta_ads_falls_back_to_business_portfolios() {
        use axum::{routing::get, Json, Router};
        let app = Router::new()
            .route(
                "/me/adaccounts",
                get(|| async { Json(json!({ "data": [] })) }),
            )
            .route(
                "/me/businesses",
                get(|| async {
                    Json(json!({ "data": [{
                        "id": "b1", "name": "Meni Co",
                        "owned_ad_accounts": { "data": [{
                            "id": "act_9", "name": "Owned", "account_status": 1,
                            "currency": "THB", "timezone_name": "Asia/Bangkok"
                        }]}
                    }]}))
                }),
            )
            .route(
                "/act_9/campaigns",
                get(|| async { Json(json!({ "data": [] })) }),
            )
            .route(
                "/act_9/adsets",
                get(|| async { Json(json!({ "data": [] })) }),
            )
            .route("/act_9/ads", get(|| async { Json(json!({ "data": [] })) }))
            .route(
                "/act_9/insights",
                get(|| async { Json(json!({ "data": [] })) }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&format!("http://{addr}"));
        let ads = fetch_meta_ads(&oauth, "token").await.unwrap();
        assert_eq!(ads.accounts.len(), 1, "business-owned account found");
        assert_eq!(ads.accounts[0].id, "act_9");
    }

    #[tokio::test]
    async fn fetch_meta_ads_asks_for_reconnect_on_permission_errors() {
        use axum::{routing::get, Json, Router};
        let app = Router::new().route(
            "/me/adaccounts",
            get(|| async {
                Json(json!({ "error": { "message": "(#200) Requires ads_read permission", "code": 200 } }))
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&format!("http://{addr}"));
        let error = fetch_meta_ads(&oauth, "token").await.unwrap_err();
        assert_eq!(error, "reconnect Meta to grant ads access");
    }

    /// Fake Graph that only accepts `user-token` on `/me/adaccounts`, so a
    /// Page token reaching the ads mirror is a test failure.
    async fn fake_user_token_graph() -> String {
        use axum::{extract::Query, routing::get, Json, Router};
        use std::collections::HashMap;
        let app = Router::new()
            .route(
                "/me/adaccounts",
                get(|Query(q): Query<HashMap<String, String>>| async move {
                    if q.get("access_token").map(String::as_str) != Some("user-token") {
                        return Json(json!({ "error": {
                            "message": "(#200) Requires ads_read permission", "code": 200
                        }}));
                    }
                    Json(json!({ "data": [{
                        "id": "act_7", "name": "Owned", "account_status": 1,
                        "currency": "THB", "timezone_name": "Asia/Bangkok"
                    }]}))
                }),
            )
            .route(
                "/act_7/campaigns",
                get(|| async { Json(json!({ "data": [] })) }),
            )
            .route(
                "/act_7/adsets",
                get(|| async { Json(json!({ "data": [] })) }),
            )
            .route("/act_7/ads", get(|| async { Json(json!({ "data": [] })) }))
            .route(
                "/act_7/insights",
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
    async fn ads_refresh_uses_the_user_token_not_the_page_token() {
        let base = fake_user_token_graph().await;
        let state = crate::new_state();
        {
            let mut store = state.write().await;
            store.oauth.set_graph_base(&base);
            store.enable_token_vault("unit-test-admin-secret-0123456789");
            store.set_platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta", "page-token");
            store.set_platform_token(
                crate::store::DEFAULT_WORKSPACE_ID,
                "meta#user",
                "user-token",
            );
        }
        let ads = refresh_workspace_ads(&state, crate::store::DEFAULT_WORKSPACE_ID)
            .await
            .unwrap();
        assert_eq!(ads.accounts.len(), 1);
        assert_eq!(ads.accounts[0].id, "act_7");
    }

    #[tokio::test]
    async fn ads_refresh_falls_back_to_a_single_legacy_token() {
        // Connections made before the Page/user split stored only the user
        // token under the plain key.
        let base = fake_user_token_graph().await;
        let state = crate::new_state();
        {
            let mut store = state.write().await;
            store.oauth.set_graph_base(&base);
            store.enable_token_vault("unit-test-admin-secret-0123456789");
            store.set_platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta", "user-token");
        }
        let ads = refresh_workspace_ads(&state, crate::store::DEFAULT_WORKSPACE_ID)
            .await
            .unwrap();
        assert_eq!(ads.accounts.len(), 1);
    }

    #[test]
    fn boost_end_clamps_to_a_sane_window() {
        let end = boost_end(7);
        assert!(end.ends_with('Z'), "ISO-8601 UTC: {end}");
        // Clamping is visible in the date being in the future, not "now".
        assert!(end > chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }
}
