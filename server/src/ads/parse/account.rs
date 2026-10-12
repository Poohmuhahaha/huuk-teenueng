//! Ad account row from `/me/adaccounts`.

use serde_json::Value;

use crate::model::AdAccount;

use super::primitives::{as_u64, str_of};

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
}
