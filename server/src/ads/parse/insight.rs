//! Last-30-days insight row from `/act_{id}/insights?level=campaign`.
//! The counted "result" depends on the campaign objective: the first matching
//! action wins (leads, purchases, installs, engagements, else link clicks).

use serde_json::Value;

use crate::model::AdInsight;

use super::primitives::{as_f64, as_u64, str_of};

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
