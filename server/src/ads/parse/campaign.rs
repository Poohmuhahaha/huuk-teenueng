//! Campaign row from `/act_{id}/campaigns`.

use serde_json::Value;

use crate::model::AdCampaign;

use super::primitives::{as_u64, str_of};

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
