//! Assembles the whole mirror for one login: accounts first, then every
//! account's structure + insights. One account failing never blanks the
//! others; the result is newest-first and snapshot-bounded.

use crate::ads::parse::permission_hint;
use crate::model::{Ad, AdCampaign, AdInsight, AdSet, AdsData};
use crate::oauth::OauthSettings;

use super::accounts::{fetch_ad_accounts, fetch_error_hint};
use super::structure::fetch_account_ads;
use super::{KEEP_ADS, KEEP_ADSETS, KEEP_CAMPAIGNS};

fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

pub async fn fetch_meta_ads(oauth: &OauthSettings, token: &str) -> Result<AdsData, String> {
    let fetched_at = now_secs();
    let accounts = fetch_ad_accounts(oauth, token)
        .await
        .map_err(fetch_error_hint)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ads::fetch::test_support::{fake_full_graph, fake_permission_denied_graph};

    #[tokio::test]
    async fn pulls_structure_settings_and_insights() {
        let base = fake_full_graph().await;
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
    async fn asks_for_reconnect_on_permission_errors() {
        let base = fake_permission_denied_graph().await;
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&base);
        let error = fetch_meta_ads(&oauth, "token").await.unwrap_err();
        assert_eq!(error, "reconnect Meta to grant ads access");
    }
}
