//! Lists the ad accounts one login can see. Business-owned accounts are
//! omitted from `/me/adaccounts`, so an empty first page falls back to
//! `/me/businesses` (the same limitation Pages have).

use crate::ads::parse::{parse_ad_account, permission_hint, rows};
use crate::model::AdAccount;
use crate::oauth::OauthSettings;

use super::super::graph::graph_get;
use super::KEEP_ACCOUNTS;

const ACCOUNT_FIELDS: &str =
    "id,account_id,name,account_status,currency,timezone_name,business{id,name}";

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

/// Maps a raw fetch failure to reconnect guidance when it is a permission
/// problem; anything else keeps its original message.
pub(super) fn fetch_error_hint(error: String) -> String {
    permission_hint(&error).unwrap_or(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ads::fetch::test_support::{fake_business_graph, fake_permission_denied_graph};

    #[tokio::test]
    async fn falls_back_to_business_portfolios() {
        let base = fake_business_graph().await;
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&base);
        let accounts = fetch_ad_accounts(&oauth, "token").await.unwrap();
        assert_eq!(accounts.len(), 1, "business-owned account found");
        assert_eq!(accounts[0].id, "act_9");
    }

    #[tokio::test]
    async fn permission_errors_surface_for_the_caller_to_hint() {
        let base = fake_permission_denied_graph().await;
        let mut oauth = OauthSettings::from_env();
        oauth.set_graph_base(&base);
        let error = fetch_ad_accounts(&oauth, "token").await.unwrap_err();
        assert_eq!(
            fetch_error_hint(error),
            "reconnect Meta to grant ads access"
        );
    }
}
