//! Shared fakes for the fetch tests: tiny Axum apps pretending to be Graph,
//! so no test needs a real Meta login.

use serde_json::json;

/// One ad account, one campaign, one ad set, one ad, one insight row.
pub(super) async fn fake_full_graph() -> String {
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
    serve(app).await
}

/// Empty `/me/adaccounts` with one Business-portfolio account behind it.
pub(super) async fn fake_business_graph() -> String {
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
    serve(app).await
}

/// Every call fails with a permission error.
pub(super) async fn fake_permission_denied_graph() -> String {
    use axum::{routing::get, Json, Router};
    let app = Router::new().route(
        "/me/adaccounts",
        get(|| async {
            Json(json!({ "error": { "message": "(#200) Requires ads_read permission", "code": 200 } }))
        }),
    );
    serve(app).await
}

/// Only accepts `user-token` on `/me/adaccounts`, so a Page token reaching
/// the ads mirror fails like a permission error.
pub(super) async fn fake_user_token_graph() -> String {
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
    serve(app).await
}

async fn serve(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    format!("http://{addr}")
}
