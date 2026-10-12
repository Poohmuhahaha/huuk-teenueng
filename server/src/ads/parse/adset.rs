//! Ad set row from `/act_{id}/adsets`, with the human targeting summary
//! (the UI never sees the raw targeting blob).

use serde_json::Value;

use crate::model::AdSet;

use super::primitives::{as_u64, str_of};

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
