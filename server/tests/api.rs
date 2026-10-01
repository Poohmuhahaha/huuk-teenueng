use axum::body::Body;
use axum::http::{Request, StatusCode};
use content_planner_server::{app, new_state};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn body_json(res: axum::response::Response) -> serde_json::Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn get(app: axum::Router, uri: &str) -> axum::response::Response {
    app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn send(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: serde_json::Value,
) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

async fn send_auth(
    app: axum::Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: serde_json::Value,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    app.oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

async fn login(app: axum::Router, email: &str, password: &str) -> String {
    let res = send(
        app,
        "POST",
        "/api/auth/login",
        serde_json::json!({
            "email": email, "password": password
        }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    body_json(res).await["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn health_is_ok() {
    let res = get(app(new_state()), "/api/health").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body_json(res).await["status"], "ok");
}

#[tokio::test]
async fn api_docs_and_openapi_spec_are_served() {
    let app = app(new_state());

    let res = get(app.clone(), "/api/docs").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers()["content-type"]
        .to_str()
        .unwrap()
        .starts_with("text/html"));
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("swagger-ui-bundle.js"));

    let res = get(app.clone(), "/api/docs/swagger-ui-bundle.js").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers()["content-type"]
        .to_str()
        .unwrap()
        .starts_with("text/javascript"));

    let res = get(app, "/api/openapi.yaml").await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let spec = String::from_utf8_lossy(&body);
    assert!(spec.contains("openapi: 3.0.3"));
    assert!(spec.contains("/api/auth/login"));
    assert!(spec.contains("/api/content/{id}/publish"));
}

#[tokio::test]
async fn posts_are_filtered_by_month() {
    let res = get(app(new_state()), "/api/posts?month=2").await;
    assert_eq!(res.status(), StatusCode::OK);
    let posts = body_json(res).await;
    assert_eq!(posts.as_array().unwrap().len(), 5);
    assert_eq!(posts[0]["topic"], "Morning Vlog");
}

#[tokio::test]
async fn post_can_be_patched() {
    let app = app(new_state());
    let res = send(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({
            "status": "Done",
            "done": true
        }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let post = body_json(res).await;
    assert_eq!(post["status"], "Done");
    assert_eq!(post["done"], true);

    // unknown id → 404 with a JSON error
    let missing = send(
        app,
        "PATCH",
        "/api/posts/nope",
        serde_json::json!({ "done": true }),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert!(body_json(missing).await["error"].is_string());
}

#[tokio::test]
async fn setup_option_is_added_once() {
    let app = app(new_state());
    let res = send(
        app.clone(),
        "POST",
        "/api/setup/options",
        serde_json::json!({
            "list": "pillars", "item": "Pillar IV"
        }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_json(res).await["pillars"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "Pillar IV"));

    // duplicate is ignored
    let again = send(
        app,
        "POST",
        "/api/setup/options",
        serde_json::json!({
            "list": "pillars", "item": "Pillar IV"
        }),
    )
    .await;
    let pillars = body_json(again).await["pillars"].as_array().unwrap().len();
    assert_eq!(pillars, 4);
}

#[tokio::test]
async fn idea_can_be_promoted_to_a_post() {
    let app = app(new_state());
    let res = send(
        app,
        "POST",
        "/api/ideas/i-1/promote?month=4",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let post = body_json(res).await;
    assert_eq!(post["month"], 4);
    assert_eq!(post["topic"], "Rainy Day Reads");
    assert_eq!(post["status"], "Start");
}

#[tokio::test]
async fn platform_connect_disconnect_and_sync() {
    let app = app(new_state());
    let res = send(
        app.clone(),
        "POST",
        "/api/platforms/youtube/connect",
        serde_json::json!({}),
    )
    .await;
    let conn = body_json(res).await;
    assert_eq!(conn["status"], "connected");
    assert_eq!(conn["tokenType"], "refresh");

    // A manual handle is stored so the UI can link to the real profile.
    let linked = send(
        app.clone(),
        "POST",
        "/api/platforms/tiktok/connect",
        serde_json::json!({ "handle": "  @my.brand  " }),
    )
    .await;
    assert_eq!(linked.status(), StatusCode::OK);
    assert_eq!(body_json(linked).await["handle"], "@my.brand");

    let too_long = send(
        app.clone(),
        "POST",
        "/api/platforms/meta/connect",
        serde_json::json!({ "handle": "x".repeat(201) }),
    )
    .await;
    assert_eq!(too_long.status(), StatusCode::BAD_REQUEST);

    // The platform home page is not a profile, and the wrong domain is rejected.
    let home = send(
        app.clone(),
        "POST",
        "/api/platforms/meta/connect",
        serde_json::json!({ "handle": "https://www.facebook.com" }),
    )
    .await;
    assert_eq!(home.status(), StatusCode::BAD_REQUEST);

    let wrong_host = send(
        app.clone(),
        "POST",
        "/api/platforms/tiktok/connect",
        serde_json::json!({ "handle": "https://www.facebook.com/someone" }),
    )
    .await;
    assert_eq!(wrong_host.status(), StatusCode::BAD_REQUEST);

    let good_url = send(
        app.clone(),
        "POST",
        "/api/platforms/meta/connect",
        serde_json::json!({ "handle": "https://www.facebook.com/profile.php?id=61592348575800" }),
    )
    .await;
    assert_eq!(good_url.status(), StatusCode::OK);
    assert_eq!(
        body_json(good_url).await["handle"],
        "https://www.facebook.com/profile.php?id=61592348575800"
    );

    let synced = send(
        app.clone(),
        "POST",
        "/api/platforms/youtube/sync",
        serde_json::json!({}),
    )
    .await;
    let conn = body_json(synced).await;
    assert_eq!(conn["mediaCount"], 3);
    assert!(conn["lastSync"].is_string());

    let disconnected = send(
        app,
        "POST",
        "/api/platforms/youtube/disconnect",
        serde_json::json!({}),
    )
    .await;
    let conn = body_json(disconnected).await;
    assert_eq!(conn["status"], "disconnected");
    assert!(conn["expiresAt"].is_null());
}

#[tokio::test]
async fn live_endpoint_returns_the_workspace_mirror() {
    let app = app(new_state());

    let live = body_json(get(app.clone(), "/api/live").await).await;
    assert!(live["fetchedAt"].is_number(), "{live}");
    let posts = live["posts"].as_array().unwrap();
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0]["platform"], "meta", "newest post first");
    assert!(posts.iter().any(|p| p["platform"] == "instagram"));
    assert_eq!(live["accounts"].as_array().unwrap().len(), 2);

    // An empty mirror (fresh store) reports "never fetched" instead of failing.
    let state = new_state();
    {
        let mut store = state.write().await;
        store.workspaces[0].live = Default::default();
    }
    let empty_app = content_planner_server::app(state);
    let live = body_json(get(empty_app, "/api/live").await).await;
    assert!(live["fetchedAt"].is_null());
    assert_eq!(live["posts"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn platform_list_reports_token_presence_without_leaking_it() {
    let state = new_state();
    {
        let mut store = state.write().await;
        store.enable_token_vault("unit-test-admin-secret-0123456789");
        store.set_platform_token(
            content_planner_server::store::DEFAULT_WORKSPACE_ID,
            "meta",
            "EAAB-distinctive-secret-token",
        );
    }
    let app = app(state);
    let platforms = body_json(get(app, "/api/platforms").await).await;
    let find = |id: &str| {
        platforms
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap()
            .clone()
    };
    assert_eq!(find("meta")["hasToken"], true);
    assert_eq!(find("meta")["status"], "connected");
    assert_eq!(find("youtube")["hasToken"], false);
    assert!(
        !platforms
            .to_string()
            .contains("EAAB-distinctive-secret-token"),
        "the API must never return the provider token"
    );
}

#[tokio::test]
async fn ads_endpoint_returns_the_workspace_mirror() {
    let app = app(new_state());

    let ads = body_json(get(app.clone(), "/api/ads").await).await;
    assert!(ads["fetchedAt"].is_number(), "{ads}");
    assert_eq!(ads["accounts"].as_array().unwrap().len(), 1);
    assert_eq!(ads["campaigns"].as_array().unwrap().len(), 2);
    assert_eq!(ads["adsets"].as_array().unwrap().len(), 2);
    assert_eq!(ads["ads"].as_array().unwrap().len(), 2);
    assert_eq!(ads["insights"].as_array().unwrap().len(), 2);
    assert_eq!(ads["canManage"], false, "writes are opt-in");
    let campaign = ads["campaigns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "cmp-ads-1")
        .unwrap();
    assert_eq!(campaign["objective"], "OUTCOME_TRAFFIC");
    assert_eq!(campaign["dailyBudget"], 50000);
    assert_eq!(ads["ads"][0]["storyId"], "102400000000001_9001");
    assert_eq!(ads["insights"][0]["resultLabel"], "Link clicks");

    // An empty mirror (fresh store) reports "never fetched" instead of failing.
    let state = new_state();
    {
        let mut store = state.write().await;
        store.workspaces[0].ads = Default::default();
    }
    let empty_app = content_planner_server::app(state);
    let ads = body_json(get(empty_app, "/api/ads").await).await;
    assert!(ads["fetchedAt"].is_null());
    assert_eq!(ads["campaigns"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn ads_mirror_is_isolated_per_workspace() {
    let state = new_state();
    {
        let mut store = state.write().await;
        store.setup.auth_required = true;
    }
    let app = app(state);
    let token = login(app.clone(), "owner@studio.local", "demo1234").await;

    let created = body_json(
        send_auth(
            app.clone(),
            "POST",
            "/api/workspaces",
            Some(&token),
            serde_json::json!({ "name": "Second Brand" }),
        )
        .await,
    )
    .await;
    let ws_id = created["id"].as_str().unwrap().to_string();

    let scoped = app
        .oneshot(
            Request::builder()
                .uri("/api/ads")
                .header("authorization", format!("Bearer {token}"))
                .header("x-workspace-id", &ws_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let ads = body_json(scoped).await;
    assert!(ads["fetchedAt"].is_null());
    assert_eq!(
        ads["campaigns"].as_array().unwrap().len(),
        0,
        "no bleed from the demo workspace"
    );
}

#[tokio::test]
async fn ads_manage_opt_in_gates_writes() {
    let app = app(new_state());

    // Writes are refused until the workspace explicitly opts in.
    let res = send(
        app.clone(),
        "POST",
        "/api/ads/campaigns/cmp-ads-1/status",
        serde_json::json!({ "status": "PAUSED" }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("management"));

    let res = send(
        app.clone(),
        "POST",
        "/api/ads/manage",
        serde_json::json!({ "enabled": true }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body_json(res).await["canManage"], true);

    // Opted in but no stored token → reconnect guidance, mirror untouched.
    let res = send(
        app.clone(),
        "POST",
        "/api/ads/campaigns/cmp-ads-1/status",
        serde_json::json!({ "status": "PAUSED" }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("reconnect"));

    let ads = body_json(get(app, "/api/ads").await).await;
    assert_eq!(ads["canManage"], true);
    let campaign = ads["campaigns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "cmp-ads-1")
        .unwrap();
    assert_eq!(campaign["status"], "ACTIVE", "failed write must not mutate");
}

#[tokio::test]
async fn ads_writes_validate_input_before_touching_the_api() {
    let app = app(new_state());
    send(
        app.clone(),
        "POST",
        "/api/ads/manage",
        serde_json::json!({ "enabled": true }),
    )
    .await;

    let res = send(
        app.clone(),
        "POST",
        "/api/ads/campaigns/nope/status",
        serde_json::json!({ "status": "PAUSED" }),
    )
    .await;
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("not found"));

    let res = send(
        app.clone(),
        "POST",
        "/api/ads/campaigns/cmp-ads-1/status",
        serde_json::json!({ "status": "NOPE" }),
    )
    .await;
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("ACTIVE"));

    let res = send(
        app.clone(),
        "POST",
        "/api/ads/campaigns/cmp-ads-1/budget",
        serde_json::json!({}),
    )
    .await;
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("dailyBudget"));

    let res = send(
        app.clone(),
        "POST",
        "/api/ads/boost",
        serde_json::json!({
            "name": "Boost", "objective": "OUTCOME_TRAFFIC",
            "dailyBudget": 1000, "days": 3, "countries": [], "storyId": "p_1"
        }),
    )
    .await;
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("country"));
}

#[tokio::test]
async fn ads_sync_requires_a_connected_meta_and_a_token() {
    let app = app(new_state());

    // The seed has a connected Meta slot but no provider token.
    let res = send(app.clone(), "POST", "/api/ads/sync", serde_json::json!({})).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("reconnect"));

    send(
        app.clone(),
        "POST",
        "/api/platforms/meta/disconnect",
        serde_json::json!({}),
    )
    .await;
    let res = send(app, "POST", "/api/ads/sync", serde_json::json!({})).await;
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("connect Meta"));
}

#[tokio::test]
async fn sync_meta_without_a_stored_token_asks_for_a_reconnect() {
    // The demo seed has a connected Meta slot but no provider token (tokens are
    // only obtained through OAuth), so sync must explain what to do.
    let app = app(new_state());
    let res = send(
        app,
        "POST",
        "/api/platforms/meta/sync",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body_json(res).await;
    assert!(
        body["error"].as_str().unwrap().contains("reconnect"),
        "{body}"
    );
}

#[tokio::test]
async fn sync_still_requires_a_connected_platform() {
    let app = app(new_state());
    let res = send(
        app,
        "POST",
        "/api/platforms/youtube/sync",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("not connected"));
}

/// Base64 PNG with the given pixel dimensions (header only — parsers read IHDR).
fn png_with_size(width: u32, height: u32) -> String {
    let mut bytes = vec![0x89u8, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}

#[tokio::test]
async fn posts_can_be_planned_and_removed() {
    let app = app(new_state());

    // A new month starts empty in production stores; a planner row can be added.
    let created = send(
        app.clone(),
        "POST",
        "/api/posts",
        serde_json::json!({
            "month": 7,
            "topic": "Songkran teaser",
            "pillar": "Promo",
            "format": "Reel",
            "goal": "Views",
            "date": "2026-07-01",
            "time": "18:00",
            "status": "Start",
            "hook": "", "caption": "", "cta": "", "hashtagGroup": "",
            "hashtags": [], "imageUrl": "", "note": "", "done": false,
            "platforms": ["Facebook"]
        }),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let id = body_json(created).await["id"].as_str().unwrap().to_string();

    let listed = body_json(get(app.clone(), "/api/posts?month=7").await).await;
    assert!(listed
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == id && p["topic"] == "Songkran teaser"));

    // Editing the core plan fields works.
    let res = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/api/posts/{id}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "topic": "Songkran launch", "date": "2026-07-03" }).to_string(),
        ))
        .unwrap();
    let patched = body_json(app.clone().oneshot(res).await.unwrap()).await;
    assert_eq!(patched["topic"], "Songkran launch");
    assert_eq!(patched["date"], "2026-07-03");

    // Deleting removes it from the month.
    let deleted = send(
        app.clone(),
        "DELETE",
        &format!("/api/posts/{id}"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::OK);
    let listed = body_json(get(app.clone(), "/api/posts?month=7").await).await;
    assert!(!listed.as_array().unwrap().iter().any(|p| p["id"] == id));

    // Deleting again is a 404.
    let again = send(
        app,
        "DELETE",
        &format!("/api/posts/{id}"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(again.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn brand_style_tokens_validate_and_persist() {
    let app = app(new_state());
    let res = send(
        app.clone(),
        "PATCH",
        "/api/brand",
        serde_json::json!({ "radius": 4, "fillOpacity": 50, "strokeWidth": 0, "shadow": "strong" }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let brand = body_json(res).await;
    assert_eq!(brand["radius"], 4);
    assert_eq!(brand["fillOpacity"], 50);
    assert_eq!(brand["strokeWidth"], 0);
    assert_eq!(brand["shadow"], "strong");

    for (patch, expected) in [
        (serde_json::json!({ "radius": 25 }), "0–24"),
        (serde_json::json!({ "fillOpacity": 4 }), "5–100"),
        (serde_json::json!({ "strokeWidth": 4 }), "0–3"),
        (
            serde_json::json!({ "shadow": "blurry" }),
            "none, soft or strong",
        ),
    ] {
        let res = send(app.clone(), "PATCH", "/api/brand", patch).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(body_json(res).await["error"]
            .as_str()
            .unwrap()
            .contains(expected));
    }
}

#[tokio::test]
async fn brand_images_upload_serve_and_delete() {
    let state = new_state();
    let dir = std::env::temp_dir().join(format!(
        "cp-brand-images-{}",
        content_planner_server::store::random_hex(6)
    ));
    {
        let mut store = state.write().await;
        store.images_dir = Some(dir.clone());
    }
    let app = app(state);

    let res = send(
        app.clone(),
        "POST",
        "/api/brand/images",
        serde_json::json!({
            "name": "My Logo.png", "data": png_with_size(500, 500), "kind": "logo"
        }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let uploaded = body_json(res).await;
    let url = uploaded["url"].as_str().unwrap().to_string();
    assert_eq!(
        url, "/api/brand/images/my-logo.png",
        "slugged, served by name"
    );
    assert_eq!(uploaded["width"], 500);
    assert_eq!(uploaded["height"], 500);

    let served = get(app.clone(), &url).await;
    assert_eq!(served.status(), StatusCode::OK);
    assert_eq!(served.headers()["content-type"], "image/png");

    // The brand kit accepts the uploaded URL (and rejects random paths).
    let patched = send(
        app.clone(),
        "PATCH",
        "/api/brand",
        serde_json::json!({ "logos": ["", url.clone(), "https://cdn.example/logo.svg"] }),
    )
    .await;
    assert_eq!(patched.status(), StatusCode::OK);
    assert_eq!(body_json(patched).await["logos"][1], url);

    let bad = send(
        app.clone(),
        "PATCH",
        "/api/brand",
        serde_json::json!({ "logos": ["javascript:alert(1)"] }),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);

    // A .png that is not a PNG is refused.
    let fake = send(
        app.clone(),
        "POST",
        "/api/brand/images",
        serde_json::json!({ "name": "fake.png", "data": "bm90IGEgcG5n", "kind": "logo" }),
    )
    .await;
    assert_eq!(fake.status(), StatusCode::BAD_REQUEST);

    // Every brand image is 32x32 - 500x500 px.
    for (name, w, h, kind, expected) in [
        ("small.png", 16, 16, "logo", "at least 32×32"),
        ("big.png", 501, 501, "logo", "at most 500×500"),
        ("ref-small.png", 16, 16, "moodboard", "at least 32×32"),
        ("huge.png", 900, 200, "moodboard", "at most 500×500"),
    ] {
        let res = send(
            app.clone(),
            "POST",
            "/api/brand/images",
            serde_json::json!({ "name": name, "data": png_with_size(w, h), "kind": kind }),
        )
        .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "{name}");
        let error = body_json(res).await["error"].as_str().unwrap().to_string();
        assert!(error.contains(expected), "{name}: {error}");
    }

    // An unknown slot is refused outright.
    let unknown = send(
        app.clone(),
        "POST",
        "/api/brand/images",
        serde_json::json!({ "name": "x.png", "data": png_with_size(500, 500), "kind": "banner" }),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(unknown).await["error"]
        .as_str()
        .unwrap()
        .contains("logo or moodboard"));

    let deleted = send(app.clone(), "DELETE", &url, serde_json::json!({})).await;
    assert_eq!(deleted.status(), StatusCode::OK);
    assert_eq!(get(app, &url).await.status(), StatusCode::NOT_FOUND);
    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn txn_requires_positive_amount() {
    let app = app(new_state());
    let bad = send(
        app.clone(),
        "POST",
        "/api/txns",
        serde_json::json!({
            "date": "2026-03-01", "amount": 0, "kind": "OUT", "category": "Gear", "sub": "Lens"
        }),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);

    let good = send(
        app,
        "POST",
        "/api/txns",
        serde_json::json!({
            "date": "2026-03-01", "amount": 900, "kind": "OUT", "category": "Gear", "sub": "Lens"
        }),
    )
    .await;
    assert_eq!(good.status(), StatusCode::OK);
    assert!(body_json(good).await["id"]
        .as_str()
        .unwrap()
        .starts_with("t-"));
}

#[tokio::test]
async fn root_says_server_is_on() {
    let res = get(app(new_state()), "/").await;
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("Server on"));
}

#[tokio::test]
async fn auth_login_me_logout_flow() {
    let app = app(new_state());
    let res = send(
        app.clone(),
        "POST",
        "/api/auth/login",
        serde_json::json!({
            "email": "owner@studio.local", "password": "demo1234"
        }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let login = body_json(res).await;
    let token = login["token"].as_str().unwrap().to_string();
    assert_eq!(token.len(), 64);
    assert!(token
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    assert_eq!(login["user"]["name"], "Studio Owner");
    assert_eq!(login["user"]["role"], "Owner");

    let me = send_auth(
        app.clone(),
        "GET",
        "/api/auth/me",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(me.status(), StatusCode::OK);
    assert_eq!(body_json(me).await["user"]["role"], "Owner");

    let out = send_auth(
        app.clone(),
        "POST",
        "/api/auth/logout",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(out.status(), StatusCode::OK);
    assert_eq!(body_json(out).await["ok"], true);

    // token is gone after logout
    let after = send_auth(
        app,
        "GET",
        "/api/auth/me",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(after.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_requires_correct_password() {
    let app = app(new_state());

    let unknown = send(
        app.clone(),
        "POST",
        "/api/auth/login",
        serde_json::json!({
            "email": "nobody@studio.local", "password": "demo1234"
        }),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::UNAUTHORIZED);
    let unknown = body_json(unknown).await;

    let wrong = send(
        app.clone(),
        "POST",
        "/api/auth/login",
        serde_json::json!({
            "email": "owner@studio.local", "password": "wrong-password"
        }),
    )
    .await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    let wrong = body_json(wrong).await;

    assert_eq!(unknown["error"], wrong["error"]);
    assert_eq!(unknown["error"], "invalid email or password");

    let ok = send(
        app,
        "POST",
        "/api/auth/login",
        serde_json::json!({
            "email": "OWNER@STUDIO.LOCAL", "password": "demo1234"
        }),
    )
    .await;
    assert_eq!(ok.status(), StatusCode::OK);
    assert_eq!(body_json(ok).await["user"]["role"], "Owner");
}

#[tokio::test]
async fn register_validates_and_creates_an_account() {
    let app = app(new_state());

    let short = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Newbie", "email": "new@test.local", "password": "short"
        }),
    )
    .await;
    assert_eq!(short.status(), StatusCode::BAD_REQUEST);

    let bad_email = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Newbie", "email": "no-at-sign", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(bad_email.status(), StatusCode::BAD_REQUEST);

    let empty_name = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "  ", "email": "new@test.local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(empty_name.status(), StatusCode::BAD_REQUEST);

    let created = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Newbie", "email": "New@Test.Local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let created = body_json(created).await;
    let token = created["token"].as_str().unwrap().to_string();
    assert_eq!(token.len(), 64);
    assert_eq!(created["user"]["name"], "Newbie");

    // auto-login: the returned token is live
    let me = send_auth(
        app.clone(),
        "GET",
        "/api/auth/me",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(me.status(), StatusCode::OK);
    assert_eq!(body_json(me).await["user"]["name"], "Newbie");

    // same email (any case) → 409
    let dup = send(
        app,
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Other", "email": "new@test.local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(dup.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn register_uses_directory_role_or_defaults_to_viewer() {
    let app = app(new_state());

    // role mapping only applies when the name is not already an account: add a
    // fresh directory entry, then register an account with that exact name.
    let listed = send(
        app.clone(),
        "POST",
        "/api/setup/users",
        serde_json::json!({
            "name": "Dir Editor", "role": "Editor"
        }),
    )
    .await;
    assert_eq!(listed.status(), StatusCode::OK);

    let editor = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Dir Editor", "email": "dir@test.local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(editor.status(), StatusCode::OK);
    assert_eq!(body_json(editor).await["user"]["role"], "Editor");

    let fresh = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Fresh Face", "email": "fresh@test.local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(fresh.status(), StatusCode::OK);
    let fresh_body = body_json(fresh).await;
    // self-serve signup is least-privilege: never Owner/Editor
    assert_eq!(fresh_body["user"]["role"], "Viewer");
    assert_eq!(fresh_body["user"]["plan"], "free");

    let setup = body_json(get(app, "/api/setup").await).await;
    assert!(setup["users"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["name"] == "Fresh Face" && u["role"] == "Viewer"));
}

#[tokio::test]
async fn reads_require_auth_and_setup_is_redacted_when_auth_is_on() {
    let state = new_state();
    {
        let mut store = state.write().await;
        store.setup.auth_required = true;
    }
    let app = app(state);

    // Authoring, finance, marketing and directory reads are private.
    for uri in [
        "/api/content",
        "/api/posts",
        "/api/ideas",
        "/api/tags",
        "/api/metrics",
        "/api/txns",
        "/api/campaigns",
        "/api/platforms",
        "/api/auth/accounts",
    ] {
        let res = get(app.clone(), uri).await;
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "{uri} must require a session"
        );
    }

    // Liveness, the public delivery API and the public brand CI stay open.
    for uri in [
        "/api/health",
        "/api/ready",
        "/api/public/content",
        "/api/brand",
        "/api/brand/fonts",
    ] {
        let res = get(app.clone(), uri).await;
        assert_eq!(res.status(), StatusCode::OK, "{uri} must stay public");
    }

    // Pre-login setup carries the config the sign-in screen needs, but no
    // staff directory or role matrix.
    let setup = body_json(get(app.clone(), "/api/setup").await).await;
    assert_eq!(setup["authRequired"], true);
    assert!(setup["users"].as_array().unwrap().is_empty());
    assert!(setup["roles"].as_array().unwrap().is_empty());

    // A live session restores both the reads and the full directory.
    let token = login(app.clone(), "owner@studio.local", "demo1234").await;
    let content = send_auth(
        app.clone(),
        "GET",
        "/api/content",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(content.status(), StatusCode::OK);

    let setup = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/setup",
            Some(&token),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    assert!(!setup["users"].as_array().unwrap().is_empty());
    assert!(!setup["roles"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn plan_is_selectable_per_account() {
    let app = app(new_state());
    let registered = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Plan Tester", "email": "plan@test.local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(registered.status(), StatusCode::OK);
    let body = body_json(registered).await;
    assert_eq!(body["user"]["plan"], "free");
    let token = body["token"].as_str().unwrap().to_string();

    let bad = send_auth(
        app.clone(),
        "POST",
        "/api/auth/plan",
        Some(&token),
        serde_json::json!({ "plan": "platinum" }),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);

    let pro = send_auth(
        app.clone(),
        "POST",
        "/api/auth/plan",
        Some(&token),
        serde_json::json!({ "plan": "pro" }),
    )
    .await;
    assert_eq!(pro.status(), StatusCode::OK);
    assert_eq!(body_json(pro).await["user"]["plan"], "pro");

    let me = send_auth(
        app,
        "GET",
        "/api/auth/me",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(body_json(me).await["user"]["plan"], "pro");
}

#[tokio::test]
async fn session_tokens_are_random_and_expirable() {
    let state = new_state();
    let app = app(state.clone());

    let first = login(app.clone(), "owner@studio.local", "demo1234").await;
    let second = login(app.clone(), "owner@studio.local", "demo1234").await;
    assert_ne!(first, second);
    assert_eq!(first.len(), 64);
    assert_eq!(second.len(), 64);

    let expired = "expired-token".to_string();
    {
        let mut store = state.write().await;
        store.sessions.insert(
            content_planner_server::store::session_key(&expired),
            content_planner_server::store::Session {
                email: "owner@studio.local".into(),
                expires: chrono::Utc::now() - chrono::Duration::seconds(1),
            },
        );
    }
    let me = send_auth(
        app,
        "GET",
        "/api/auth/me",
        Some(&expired),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(me.status(), StatusCode::UNAUTHORIZED);
    assert!(state.read().await.session(&expired).is_none());
}

#[tokio::test]
async fn change_password_flow() {
    let app = app(new_state());
    let token = login(app.clone(), "owner@studio.local", "demo1234").await;
    let other = login(app.clone(), "owner@studio.local", "demo1234").await;

    let wrong_old = send_auth(
        app.clone(),
        "POST",
        "/api/auth/change-password",
        Some(&token),
        serde_json::json!({
            "oldPassword": "not-it", "newPassword": "newpassword1"
        }),
    )
    .await;
    assert_eq!(wrong_old.status(), StatusCode::UNAUTHORIZED);

    let short = send_auth(
        app.clone(),
        "POST",
        "/api/auth/change-password",
        Some(&token),
        serde_json::json!({
            "oldPassword": "demo1234", "newPassword": "short"
        }),
    )
    .await;
    assert_eq!(short.status(), StatusCode::BAD_REQUEST);

    let changed = send_auth(
        app.clone(),
        "POST",
        "/api/auth/change-password",
        Some(&token),
        serde_json::json!({
            "oldPassword": "demo1234", "newPassword": "newpassword1"
        }),
    )
    .await;
    assert_eq!(changed.status(), StatusCode::OK);
    assert_eq!(body_json(changed).await["ok"], true);

    // the old password no longer logs in, the new one does
    let old_login = send(
        app.clone(),
        "POST",
        "/api/auth/login",
        serde_json::json!({
            "email": "owner@studio.local", "password": "demo1234"
        }),
    )
    .await;
    assert_eq!(old_login.status(), StatusCode::UNAUTHORIZED);
    let fresh = login(app.clone(), "owner@studio.local", "newpassword1").await;

    // the other pre-change session is invalidated, the caller's token survives
    let other_me = send_auth(
        app.clone(),
        "GET",
        "/api/auth/me",
        Some(&other),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(other_me.status(), StatusCode::UNAUTHORIZED);
    let caller_me = send_auth(
        app.clone(),
        "GET",
        "/api/auth/me",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(caller_me.status(), StatusCode::OK);
    let fresh_me = send_auth(
        app,
        "GET",
        "/api/auth/me",
        Some(&fresh),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(fresh_me.status(), StatusCode::OK);
}

#[tokio::test]
async fn me_without_token_is_unauthorized() {
    let res = get(app(new_state()), "/api/auth/me").await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn users_can_be_added_and_removed() {
    let app = app(new_state());
    let res = send(
        app.clone(),
        "POST",
        "/api/setup/users",
        serde_json::json!({
            "name": "Temp User", "role": "Editor"
        }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let setup = body_json(res).await;
    assert!(setup["users"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["name"] == "Temp User"));

    // duplicate name → 400
    let dup = send(
        app.clone(),
        "POST",
        "/api/setup/users",
        serde_json::json!({
            "name": "Temp User"
        }),
    )
    .await;
    assert_eq!(dup.status(), StatusCode::BAD_REQUEST);

    // remove → gone
    let removed = send(
        app.clone(),
        "DELETE",
        "/api/setup/users/Temp%20User",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(removed.status(), StatusCode::OK);
    let setup = body_json(removed).await;
    assert!(!setup["users"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["name"] == "Temp User"));

    // empty role defaults to Editor
    let roleless = send(
        app.clone(),
        "POST",
        "/api/setup/users",
        serde_json::json!({
            "name": "No Role"
        }),
    )
    .await;
    assert_eq!(body_json(roleless).await["users"][2]["role"], "Editor");

    // shrink to one user, then the last one cannot be removed
    let one_left = send(
        app.clone(),
        "DELETE",
        "/api/setup/users/Studio%20Owner",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(one_left.status(), StatusCode::OK);
    let two_left = send(
        app.clone(),
        "DELETE",
        "/api/setup/users/Editor%20Earn",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(two_left.status(), StatusCode::OK);
    let last = send(
        app,
        "DELETE",
        "/api/setup/users/No%20Role",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(last.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_lock_blocks_other_users() {
    let app = app(new_state());
    let locked = send(
        app.clone(),
        "POST",
        "/api/posts/p-desk/lock",
        serde_json::json!({
            "user": "A"
        }),
    )
    .await;
    assert_eq!(locked.status(), StatusCode::OK);
    assert_eq!(body_json(locked).await["lockedBy"], "A");

    // locking again as the same user is idempotent
    let again = send(
        app.clone(),
        "POST",
        "/api/posts/p-desk/lock",
        serde_json::json!({ "user": "A" }),
    )
    .await;
    assert_eq!(again.status(), StatusCode::OK);
    assert_eq!(body_json(again).await["lockedBy"], "A");

    // another user cannot patch a locked post
    let denied = send(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({
            "user": "B", "status": "Done"
        }),
    )
    .await;
    assert_eq!(denied.status(), StatusCode::CONFLICT);
    assert!(body_json(denied).await["error"]
        .as_str()
        .unwrap()
        .contains("A"));

    // the holder can
    let allowed = send(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({
            "user": "A", "status": "Done"
        }),
    )
    .await;
    assert_eq!(allowed.status(), StatusCode::OK);

    // another user cannot unlock
    let wrong = send(
        app.clone(),
        "POST",
        "/api/posts/p-desk/unlock",
        serde_json::json!({ "user": "B" }),
    )
    .await;
    assert_eq!(wrong.status(), StatusCode::CONFLICT);

    // the holder can, and the post is free again
    let unlocked = send(
        app.clone(),
        "POST",
        "/api/posts/p-desk/unlock",
        serde_json::json!({ "user": "A" }),
    )
    .await;
    assert_eq!(unlocked.status(), StatusCode::OK);
    assert!(body_json(unlocked).await["lockedBy"].is_null());

    let free = send(
        app,
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({
            "user": "B", "status": "Done"
        }),
    )
    .await;
    assert_eq!(free.status(), StatusCode::OK);
}

#[tokio::test]
async fn metrics_import_is_deterministic() {
    let app = app(new_state());
    let first = send(
        app.clone(),
        "POST",
        "/api/metrics/import",
        serde_json::json!({
            "platform": "Instagram", "month": 2
        }),
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);
    let first = body_json(first).await;
    let imported = first["imported"].as_u64().unwrap();
    assert!(imported >= 1);
    let rows = first["metrics"].as_array().unwrap();
    assert_eq!(rows.len() as u64, imported);
    for m in rows {
        assert_eq!(m["platform"], "Instagram");
        assert!(m["views"].as_u64().unwrap() >= 1000);
    }

    // importing again returns exactly the same rows
    let second = send(
        app.clone(),
        "POST",
        "/api/metrics/import",
        serde_json::json!({
            "platform": "Instagram", "month": 2
        }),
    )
    .await;
    assert_eq!(body_json(second).await, first);

    // empty platform → 400
    let empty = send(
        app,
        "POST",
        "/api/metrics/import",
        serde_json::json!({
            "platform": "", "month": 2
        }),
    )
    .await;
    assert_eq!(empty.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn anonymous_writes_are_open_when_auth_not_required() {
    let app = app(new_state());
    // default seed: authRequired is off, roles are Owner/Editor/Viewer
    let setup = body_json(get(app.clone(), "/api/setup").await).await;
    assert_eq!(setup["authRequired"], false);
    let roles = setup["roles"].as_array().unwrap();
    assert!(roles
        .iter()
        .any(|r| r["name"] == "Owner" && r["permissions"].as_array().unwrap().len() == 15));
    assert!(roles
        .iter()
        .any(|r| r["name"] == "Viewer" && r["permissions"].as_array().unwrap().is_empty()));

    // no token needed while auth is off
    let res = send(
        app,
        "PATCH",
        "/api/setup",
        serde_json::json!({ "workspaceName": "Anonymous Write" }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body_json(res).await["workspaceName"], "Anonymous Write");
}

#[tokio::test]
async fn auth_required_blocks_anonymous_writes() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({
            "authRequired": true
        }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);
    assert_eq!(body_json(enabled).await["authRequired"], true);

    // anonymous write → 401, anonymous read stays 200
    let blocked = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({ "workspaceName": "Nope" }),
    )
    .await;
    assert_eq!(blocked.status(), StatusCode::UNAUTHORIZED);
    assert!(body_json(blocked).await["error"].is_string());

    let read = get(app, "/api/setup").await;
    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(body_json(read).await["authRequired"], true);
}

#[tokio::test]
async fn role_permissions_are_enforced() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({
            "authRequired": true
        }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);

    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;
    let added = send_auth(
        app.clone(),
        "POST",
        "/api/setup/users",
        Some(&owner),
        serde_json::json!({
            "name": "Viewer Val", "role": "Viewer"
        }),
    )
    .await;
    assert_eq!(added.status(), StatusCode::OK);
    // registering an account for that directory user keeps the Viewer role
    let registered = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Viewer Val", "email": "viewer@test.local", "password": "viewerpass123"
        }),
    )
    .await;
    assert_eq!(registered.status(), StatusCode::OK);
    assert_eq!(body_json(registered).await["user"]["role"], "Viewer");

    // Editor has posts.write → 200, but not setup.write → 403
    let editor = login(app.clone(), "editor@studio.local", "demo1234").await;
    let edited = send_auth(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        Some(&editor),
        serde_json::json!({
            "user": "Editor Earn", "status": "Done"
        }),
    )
    .await;
    assert_eq!(edited.status(), StatusCode::OK);

    let denied_setup = send_auth(
        app.clone(),
        "PATCH",
        "/api/setup",
        Some(&editor),
        serde_json::json!({
            "owner": "Editor Earn"
        }),
    )
    .await;
    assert_eq!(denied_setup.status(), StatusCode::FORBIDDEN);
    assert!(body_json(denied_setup).await["error"]
        .as_str()
        .unwrap()
        .contains("setup.write"));

    // Viewer has no permissions → 403
    let viewer = login(app.clone(), "viewer@test.local", "viewerpass123").await;
    let denied_post = send_auth(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        Some(&viewer),
        serde_json::json!({
            "user": "Viewer Val", "status": "Done"
        }),
    )
    .await;
    assert_eq!(denied_post.status(), StatusCode::FORBIDDEN);

    // no token at all → 401
    let anon = send(
        app,
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({ "status": "Done" }),
    )
    .await;
    assert_eq!(anon.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn roles_can_be_edited_and_guarded() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({
            "authRequired": true
        }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);
    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;
    let editor = login(app.clone(), "editor@studio.local", "demo1234").await;

    // duplicate or empty role names → 400
    let dup = send_auth(
        app.clone(),
        "POST",
        "/api/setup/roles",
        Some(&owner),
        serde_json::json!({
            "name": "Owner"
        }),
    )
    .await;
    assert_eq!(dup.status(), StatusCode::BAD_REQUEST);
    let empty = send_auth(
        app.clone(),
        "POST",
        "/api/setup/roles",
        Some(&owner),
        serde_json::json!({
            "name": "  ", "permissions": []
        }),
    )
    .await;
    assert_eq!(empty.status(), StatusCode::BAD_REQUEST);

    // unknown permission → 400, both on POST and PATCH
    let bad_perm = send_auth(
        app.clone(),
        "POST",
        "/api/setup/roles",
        Some(&owner),
        serde_json::json!({
            "name": "Intern", "permissions": ["posts.publish"]
        }),
    )
    .await;
    assert_eq!(bad_perm.status(), StatusCode::BAD_REQUEST);

    let bad_patch = send_auth(
        app.clone(),
        "PATCH",
        "/api/setup",
        Some(&owner),
        serde_json::json!({
            "roles": [{ "name": "Editor", "permissions": ["posts.publish"] }]
        }),
    )
    .await;
    assert_eq!(bad_patch.status(), StatusCode::BAD_REQUEST);

    // temporarily strip posts.write from Editor (full replacement)
    let stripped = send_auth(app.clone(), "PATCH", "/api/setup", Some(&owner), serde_json::json!({
        "roles": [
            { "name": "Owner", "permissions": [
                "setup.write", "users.manage", "posts.write", "posts.lock", "metrics.import",
                "finance.write", "brand.write", "ideas.write", "hashtags.write", "platforms.manage"
            ] },
            { "name": "Editor", "permissions": ["posts.lock"] },
            { "name": "Viewer", "permissions": [] }
        ]
    }))
    .await;
    assert_eq!(stripped.status(), StatusCode::OK);

    let now_denied = send_auth(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        Some(&editor),
        serde_json::json!({
            "user": "Editor Earn", "status": "Done"
        }),
    )
    .await;
    assert_eq!(now_denied.status(), StatusCode::FORBIDDEN);

    // deleting a role that is in use → 400
    let in_use = send_auth(
        app.clone(),
        "DELETE",
        "/api/setup/roles/Editor",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(in_use.status(), StatusCode::BAD_REQUEST);

    // an added, unused role can be deleted again
    let created = send_auth(
        app.clone(),
        "POST",
        "/api/setup/roles",
        Some(&owner),
        serde_json::json!({
            "name": "Intern"
        }),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    assert!(body_json(created).await["roles"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["name"] == "Intern"));

    let deleted = send_auth(
        app,
        "DELETE",
        "/api/setup/roles/Intern",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::OK);
    assert!(!body_json(deleted).await["roles"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["name"] == "Intern"));
}

#[tokio::test]
async fn register_rejects_duplicate_display_name() {
    let app = app(new_state());

    let first = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Shared Name", "email": "shared1@test.local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);

    let second = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Shared Name", "email": "shared2@test.local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
    assert_eq!(body_json(second).await["error"], "name already taken");

    let setup = body_json(get(app, "/api/setup").await).await;
    assert_eq!(
        setup["users"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|u| u["name"] == "Shared Name")
            .count(),
        1
    );
}

#[tokio::test]
async fn accounts_can_be_listed_and_revoked() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({
            "authRequired": true
        }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);
    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;
    let editor = login(app.clone(), "editor@studio.local", "demo1234").await;

    // public sign-up, then the new account has one live session
    let registered = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Revoke Me", "email": "revoke@test.local", "password": "revokeme12345"
        }),
    )
    .await;
    assert_eq!(registered.status(), StatusCode::OK);
    let viewer = body_json(registered).await["token"]
        .as_str()
        .unwrap()
        .to_string();

    // Editor lacks users.manage
    let denied = send_auth(
        app.clone(),
        "GET",
        "/api/auth/accounts",
        Some(&editor),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let listed = send_auth(
        app.clone(),
        "GET",
        "/api/auth/accounts",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(listed.status(), StatusCode::OK);
    let rows = body_json(listed).await;
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["email"] == "revoke@test.local")
        .cloned()
        .unwrap();
    assert_eq!(row["name"], "Revoke Me");
    assert_eq!(row["sessions"], 1);
    assert!(row.get("passwordHash").is_none());

    // logout-all revokes the account's sessions but nobody else's
    let out = send_auth(
        app.clone(),
        "POST",
        "/api/auth/logout-all",
        Some(&viewer),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(out.status(), StatusCode::OK);
    let gone = send_auth(
        app.clone(),
        "GET",
        "/api/auth/me",
        Some(&viewer),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(gone.status(), StatusCode::UNAUTHORIZED);
    let owner_me = send_auth(
        app.clone(),
        "GET",
        "/api/auth/me",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(owner_me.status(), StatusCode::OK);

    // deleting the account revokes credentials and sessions
    let fresh = login(app.clone(), "revoke@test.local", "revokeme12345").await;
    let deleted = send_auth(
        app.clone(),
        "DELETE",
        "/api/auth/accounts/REVOKE%40TEST.LOCAL",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::OK);
    let after = send_auth(
        app.clone(),
        "GET",
        "/api/auth/me",
        Some(&fresh),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(after.status(), StatusCode::UNAUTHORIZED);
    let relogin = send(
        app.clone(),
        "POST",
        "/api/auth/login",
        serde_json::json!({
            "email": "revoke@test.local", "password": "revokeme12345"
        }),
    )
    .await;
    assert_eq!(relogin.status(), StatusCode::UNAUTHORIZED);
    let missing = send_auth(
        app,
        "DELETE",
        "/api/auth/accounts/ghost%40test.local",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn session_derived_locks_ignore_client_names() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({
            "authRequired": true
        }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);
    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;
    let editor = login(app.clone(), "editor@studio.local", "demo1234").await;

    // the holder is the signed-in account, not the body name
    let locked = send_auth(
        app.clone(),
        "POST",
        "/api/posts/p-desk/lock",
        Some(&owner),
        serde_json::json!({
            "user": "Impostor"
        }),
    )
    .await;
    assert_eq!(locked.status(), StatusCode::OK);
    assert_eq!(body_json(locked).await["lockedBy"], "Studio Owner");

    // Editor cannot impersonate the holder through patch.user
    let denied = send_auth(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        Some(&editor),
        serde_json::json!({
            "user": "Studio Owner", "status": "Done"
        }),
    )
    .await;
    assert_eq!(denied.status(), StatusCode::CONFLICT);

    let wrong_unlock = send_auth(
        app.clone(),
        "POST",
        "/api/posts/p-desk/unlock",
        Some(&editor),
        serde_json::json!({
            "user": "Studio Owner"
        }),
    )
    .await;
    assert_eq!(wrong_unlock.status(), StatusCode::CONFLICT);

    // the real holder can
    let unlocked = send_auth(
        app.clone(),
        "POST",
        "/api/posts/p-desk/unlock",
        Some(&owner),
        serde_json::json!({
            "user": "Anyone"
        }),
    )
    .await;
    assert_eq!(unlocked.status(), StatusCode::OK);
    assert!(body_json(unlocked).await["lockedBy"].is_null());

    // client-supplied lockedBy on create is ignored too
    let created = send_auth(
        app,
        "POST",
        "/api/posts",
        Some(&owner),
        serde_json::json!({
            "month": 5, "topic": "Sneaky Lock", "pillar": "P", "format": "Reel", "goal": "G",
            "date": null, "time": "09:00", "status": "Start", "hook": "", "caption": "", "cta": "",
            "hashtagGroup": "", "hashtags": [], "imageUrl": "", "note": "", "done": false,
            "platforms": [], "lockedBy": "Impostor"
        }),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    assert!(body_json(created).await["lockedBy"].is_null());
}

#[tokio::test]
async fn role_patch_is_guarded() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({
            "authRequired": true
        }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);
    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;

    // empty replacement
    let empty = send_auth(
        app.clone(),
        "PATCH",
        "/api/setup",
        Some(&owner),
        serde_json::json!({
            "roles": []
        }),
    )
    .await;
    assert_eq!(empty.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(empty).await["error"]
        .as_str()
        .unwrap()
        .contains("at least one role"));

    // duplicate names
    let twins = send_auth(
        app.clone(),
        "PATCH",
        "/api/setup",
        Some(&owner),
        serde_json::json!({
            "roles": [
                { "name": "Owner", "permissions": ["users.manage"] },
                { "name": "Owner", "permissions": [] },
                { "name": "Editor", "permissions": [] },
                { "name": "Viewer", "permissions": [] }
            ]
        }),
    )
    .await;
    assert_eq!(twins.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(twins).await["error"]
        .as_str()
        .unwrap()
        .contains("duplicate role name"));

    // dropping a role that a directory user still holds
    let in_use = send_auth(
        app.clone(),
        "PATCH",
        "/api/setup",
        Some(&owner),
        serde_json::json!({
            "roles": [
                { "name": "Owner", "permissions": ["users.manage"] },
                { "name": "Viewer", "permissions": [] }
            ]
        }),
    )
    .await;
    assert_eq!(in_use.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(in_use).await["error"]
        .as_str()
        .unwrap()
        .contains("role is in use: Editor"));

    // after unassigning Editor, the same shrink is allowed
    let removed = send_auth(
        app.clone(),
        "DELETE",
        "/api/setup/users/Editor%20Earn",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(removed.status(), StatusCode::OK);
    let shrunk = send_auth(
        app,
        "PATCH",
        "/api/setup",
        Some(&owner),
        serde_json::json!({
            "roles": [
                { "name": "Owner", "permissions": ["users.manage"] },
                { "name": "Viewer", "permissions": [] }
            ]
        }),
    )
    .await;
    assert_eq!(shrunk.status(), StatusCode::OK);
    assert_eq!(
        body_json(shrunk).await["roles"].as_array().unwrap().len(),
        2
    );
}

#[tokio::test]
async fn profile_update_requires_session_and_unique_name() {
    let app = app(new_state());

    // no session → 401 even in demo mode (profile editing is per-account)
    let anon = send(
        app.clone(),
        "PATCH",
        "/api/auth/profile",
        serde_json::json!({ "name": "Nope" }),
    )
    .await;
    assert_eq!(anon.status(), StatusCode::UNAUTHORIZED);

    let token = login(app.clone(), "editor@studio.local", "demo1234").await;

    let empty = send_auth(
        app.clone(),
        "PATCH",
        "/api/auth/profile",
        Some(&token),
        serde_json::json!({ "name": "  " }),
    )
    .await;
    assert_eq!(empty.status(), StatusCode::BAD_REQUEST);

    let taken = send_auth(
        app.clone(),
        "PATCH",
        "/api/auth/profile",
        Some(&token),
        serde_json::json!({ "name": "Studio Owner" }),
    )
    .await;
    assert_eq!(taken.status(), StatusCode::CONFLICT);
    assert_eq!(body_json(taken).await["error"], "name already taken");

    let renamed = send_auth(
        app.clone(),
        "PATCH",
        "/api/auth/profile",
        Some(&token),
        serde_json::json!({ "name": "Earn Renamed" }),
    )
    .await;
    assert_eq!(renamed.status(), StatusCode::OK);
    let renamed = body_json(renamed).await;
    assert_eq!(renamed["user"]["name"], "Earn Renamed");
    assert_eq!(renamed["user"]["role"], "Editor");

    // the directory entry follows the rename, so the role survives
    let setup = body_json(get(app.clone(), "/api/setup").await).await;
    assert!(setup["users"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["name"] == "Earn Renamed" && u["role"] == "Editor"));
    assert!(!setup["users"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["name"] == "Editor Earn"));

    let me = send_auth(
        app,
        "GET",
        "/api/auth/me",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(body_json(me).await["user"]["name"], "Earn Renamed");
}

#[tokio::test]
async fn last_role_cannot_be_deleted() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({
            "authRequired": true
        }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);
    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;

    // unassign Editor first so the shrink is not blocked by the in-use guard
    let removed = send_auth(
        app.clone(),
        "DELETE",
        "/api/setup/users/Editor%20Earn",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(removed.status(), StatusCode::OK);

    let shrunk = send_auth(
        app.clone(),
        "PATCH",
        "/api/setup",
        Some(&owner),
        serde_json::json!({
            "roles": [{ "name": "Owner", "permissions": ["users.manage"] }]
        }),
    )
    .await;
    assert_eq!(shrunk.status(), StatusCode::OK);
    assert_eq!(
        body_json(shrunk).await["roles"].as_array().unwrap().len(),
        1
    );

    let last = send_auth(
        app,
        "DELETE",
        "/api/setup/roles/Owner",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(last.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(last).await["error"]
        .as_str()
        .unwrap()
        .contains("last role"));
}

#[tokio::test]
async fn oauth_runs_in_mock_mode_without_credentials() {
    let app = app(new_state());
    let status = get(app.clone(), "/api/oauth/meta").await;
    assert_eq!(status.status(), StatusCode::OK);
    let body = body_json(status).await;
    assert_eq!(body["mode"], "mock");
    assert_eq!(body["configured"], false);

    let start = get(app.clone(), "/api/oauth/meta/start").await;
    assert_eq!(start.status(), StatusCode::OK);
    assert_eq!(body_json(start).await["mode"], "mock");

    let unknown = get(app, "/api/oauth/myspace").await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn oauth_callback_requires_valid_state() {
    let app = app(new_state());
    let missing = get(app.clone(), "/api/oauth/meta/callback?code=demo").await;
    assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
    let bogus = get(app, "/api/oauth/meta/callback?code=demo&state=nope").await;
    assert_eq!(bogus.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn oauth_mock_callback_connects_and_redirects() {
    let state = new_state();
    {
        let mut store = state.write().await;
        store.oauth_states.insert(
            "st-test".into(),
            content_planner_server::oauth::OAuthState::new(
                "tiktok",
                content_planner_server::store::DEFAULT_WORKSPACE_ID,
            ),
        );
    }
    let app = app(state);

    let res = get(
        app.clone(),
        "/api/oauth/tiktok/callback?code=demo&state=st-test",
    )
    .await;
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    let location = res
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(location.contains("oauth=tiktok"), "{location}");
    assert!(location.contains("status=ok"), "{location}");

    // state is single-use
    let again = get(
        app.clone(),
        "/api/oauth/tiktok/callback?code=demo&state=st-test",
    )
    .await;
    assert_eq!(again.status(), StatusCode::BAD_REQUEST);

    let platforms = body_json(get(app, "/api/platforms").await).await;
    let tiktok = platforms
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "tiktok")
        .unwrap();
    assert_eq!(tiktok["status"], "connected");
}

#[tokio::test]
async fn oauth_redirect_mode_builds_authorize_url_when_configured() {
    let state = new_state();
    {
        let mut store = state.write().await;
        store
            .oauth
            .set_credentials("meta", "app-id-123", "secret-xyz");
    }
    let app = app(state);

    let status = body_json(get(app.clone(), "/api/oauth/meta").await).await;
    assert_eq!(status["mode"], "redirect");
    assert_eq!(status["configured"], true);
    assert_eq!(status["authorizeHost"], "www.facebook.com");

    let start = body_json(get(app, "/api/oauth/meta/start").await).await;
    assert_eq!(start["mode"], "redirect");
    let url = start["url"].as_str().unwrap();
    assert!(
        url.starts_with("https://www.facebook.com/v26.0/dialog/oauth?"),
        "{url}"
    );
    assert!(url.contains("client_id=app-id-123"), "{url}");
    assert!(
        url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8787%2Fapi%2Foauth%2Fmeta%2Fcallback"),
        "{url}"
    );
    assert!(url.contains("state=st-"), "{url}");
    assert!(url.contains("scope=pages_show_list"), "{url}");
    assert!(
        url.contains("auth_type=rerequest"),
        "Meta retries must re-show the consent dialog: {url}"
    );
    assert!(
        url.contains("business_management"),
        "Business-portfolio Pages are invisible without business_management: {url}"
    );
    assert!(
        !url.contains("secret-xyz"),
        "client secret must never leave the server"
    );
}

fn seed_pick(
    store: &mut content_planner_server::store::Store,
    pick: &str,
    created: chrono::DateTime<chrono::Utc>,
) {
    use content_planner_server::oauth::{Account, PendingPick};
    let account = |handle: &str, id: &str| Account {
        handle: handle.into(),
        external_id: id.into(),
        scopes: vec![],
        token_type: "long-lived".into(),
        expires_at: None,
    };
    store.oauth_picks.insert(
        pick.into(),
        PendingPick {
            platform: "meta".into(),
            token: "tok-test-secret".into(),
            accounts: vec![account("Page One", "111"), account("Page Two", "222")],
            created,
            workspace_id: content_planner_server::store::DEFAULT_WORKSPACE_ID.into(),
        },
    );
}

#[tokio::test]
async fn oauth_pick_lists_candidates_and_finalizes_choice() {
    let state = new_state();
    {
        let mut store = state.write().await;
        store.setup.auth_required = true;
        seed_pick(&mut store, "pick-test", chrono::Utc::now());
    }
    let app = app(state);

    // anonymous callers learn nothing, even with the pick id
    let anon = get(app.clone(), "/api/oauth/meta/pending?pick=pick-test").await;
    assert_eq!(anon.status(), StatusCode::UNAUTHORIZED);

    let token = login(app.clone(), "owner@studio.local", "demo1234").await;

    // unknown pick → 404
    let missing = send_auth(
        app.clone(),
        "GET",
        "/api/oauth/meta/pending?pick=nope",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);

    // the candidate list shows handles but never the stored user token
    let listed = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/oauth/meta/pending?pick=pick-test",
            Some(&token),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    let accounts = listed["accounts"].as_array().unwrap();
    assert_eq!(accounts.len(), 2);
    assert_eq!(listed.to_string().matches("tok-test-secret").count(), 0);
    assert!(accounts.iter().any(|a| a["handle"] == "Page One"));

    // choosing a page that is not part of the pick → 404, pick survives
    let bad = send_auth(
        app.clone(),
        "POST",
        "/api/oauth/meta/choose",
        Some(&token),
        serde_json::json!({ "pick": "pick-test", "external_id": "999" }),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::NOT_FOUND);

    // choosing a page finalizes the connection and consumes the pick
    let ok = send_auth(
        app.clone(),
        "POST",
        "/api/oauth/meta/choose",
        Some(&token),
        serde_json::json!({ "pick": "pick-test", "external_id": "222" }),
    )
    .await;
    assert_eq!(ok.status(), StatusCode::OK);
    assert_eq!(body_json(ok).await["connection"]["handle"], "Page Two");

    let gone = send_auth(
        app.clone(),
        "GET",
        "/api/oauth/meta/pending?pick=pick-test",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(gone.status(), StatusCode::NOT_FOUND);

    let platforms = body_json(
        send_auth(
            app,
            "GET",
            "/api/platforms",
            Some(&token),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    let fb = platforms
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "meta")
        .unwrap();
    assert_eq!(fb["status"], "connected");
    assert_eq!(fb["handle"], "Page Two");
}

#[tokio::test]
async fn oauth_pick_expiry_is_enforced() {
    let state = new_state();
    {
        let mut store = state.write().await;
        store.setup.auth_required = true;
        seed_pick(
            &mut store,
            "pick-old",
            chrono::Utc::now() - chrono::Duration::minutes(30),
        );
    }
    let app = app(state);
    let token = login(app.clone(), "owner@studio.local", "demo1234").await;

    let listed = send_auth(
        app.clone(),
        "GET",
        "/api/oauth/meta/pending?pick=pick-old",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(listed.status(), StatusCode::BAD_REQUEST);

    let chosen = send_auth(
        app,
        "POST",
        "/api/oauth/meta/choose",
        Some(&token),
        serde_json::json!({ "pick": "pick-old", "external_id": "111" }),
    )
    .await;
    assert_eq!(chosen.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn oauth_start_respects_permissions_when_auth_required() {
    let state = new_state();
    {
        let mut store = state.write().await;
        store
            .oauth
            .set_credentials("meta", "app-id-123", "secret-xyz");
    }
    let app = app(state);
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({
            "authRequired": true
        }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);

    let anon = get(app.clone(), "/api/oauth/meta/start").await;
    assert_eq!(anon.status(), StatusCode::UNAUTHORIZED);

    let editor = login(app.clone(), "editor@studio.local", "demo1234").await;
    let editor_start = send_auth(
        app.clone(),
        "GET",
        "/api/oauth/meta/start",
        Some(&editor),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(editor_start.status(), StatusCode::FORBIDDEN);

    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;
    let owner_start = send_auth(
        app,
        "GET",
        "/api/oauth/meta/start",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(owner_start.status(), StatusCode::OK);
    assert_eq!(body_json(owner_start).await["mode"], "redirect");
}

// ---- production hardening regressions ----

#[tokio::test]
async fn new_ids_do_not_collide_with_seed_rows() {
    let app = app(new_state());
    let created = send(
        app.clone(),
        "POST",
        "/api/ideas",
        serde_json::json!({
            "topic": "Fresh Idea", "format": "Short-video", "idea": "x", "link": "", "done": false
        }),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let idea = body_json(created).await;
    let id = idea["id"].as_str().unwrap().to_string();
    assert!(id.starts_with("i-"));
    assert!(
        !["i-1", "i-2", "i-3"].contains(&id.as_str()),
        "collided with seed id {id}"
    );

    // toggling the new idea must not touch a seed row
    let toggled = send(
        app.clone(),
        "POST",
        &format!("/api/ideas/{id}/toggle"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(body_json(toggled).await["done"], true);
    let ideas = body_json(get(app, "/api/ideas").await).await;
    let seed = ideas
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == "i-1")
        .unwrap();
    assert_eq!(seed["done"], false);
}

#[tokio::test]
async fn setup_patch_is_atomic() {
    let app = app(new_state());
    let rejected = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({
            "authRequired": true,
            "roles": []
        }),
    )
    .await;
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);

    // the invalid request must not have flipped authRequired
    let setup = body_json(get(app, "/api/setup").await).await;
    assert_eq!(setup["authRequired"], false);
}

#[tokio::test]
async fn unknown_role_is_rejected_for_directory_users() {
    let app = app(new_state());
    let bad = send(
        app.clone(),
        "POST",
        "/api/setup/users",
        serde_json::json!({
            "name": "Ghost Role", "role": "Ghost"
        }),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);

    let padded = send(
        app.clone(),
        "POST",
        "/api/setup/users",
        serde_json::json!({
            "name": "  Trimmed User  ", "role": " Editor "
        }),
    )
    .await;
    assert_eq!(padded.status(), StatusCode::OK);
    let setup = body_json(padded).await;
    assert!(setup["users"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["name"] == "Trimmed User" && u["role"] == "Editor"));
}

#[tokio::test]
async fn post_date_can_be_cleared_and_invalid_values_rejected() {
    let app = app(new_state());
    let cleared = send(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({ "date": null }),
    )
    .await;
    assert_eq!(cleared.status(), StatusCode::OK);
    assert!(body_json(cleared).await["date"].is_null());

    let invalid = send(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({ "date": "07/02/2026" }),
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);

    let bad_month = send(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({ "month": 13 }),
    )
    .await;
    assert_eq!(bad_month.status(), StatusCode::BAD_REQUEST);

    // the row is intact after rejected patches
    let row = body_json(get(app.clone(), "/api/posts?month=2").await).await;
    let post = row
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "p-desk")
        .unwrap();
    assert!(post["date"].is_null());
    assert_eq!(post["month"], 2);
}

#[tokio::test]
async fn unknown_status_is_rejected_on_post_write() {
    let app = app(new_state());
    let bad = send(
        app.clone(),
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({ "status": "Imaginary" }),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
    let good = send(
        app,
        "PATCH",
        "/api/posts/p-desk",
        serde_json::json!({ "status": "Done" }),
    )
    .await;
    assert_eq!(good.status(), StatusCode::OK);
}

#[tokio::test]
async fn import_metrics_validates_platform_and_month() {
    let app = app(new_state());
    let unknown = send(
        app.clone(),
        "POST",
        "/api/metrics/import",
        serde_json::json!({
            "platform": "Myspace", "month": 2
        }),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);

    let bad_month = send(
        app.clone(),
        "POST",
        "/api/metrics/import",
        serde_json::json!({
            "platform": "Instagram", "month": 42
        }),
    )
    .await;
    assert_eq!(bad_month.status(), StatusCode::BAD_REQUEST);

    let padded = send(
        app,
        "POST",
        "/api/metrics/import",
        serde_json::json!({
            "platform": "  Instagram  ", "month": 2
        }),
    )
    .await;
    assert_eq!(padded.status(), StatusCode::OK);
    assert_eq!(
        body_json(padded).await["metrics"][0]["platform"],
        "Instagram"
    );
}

#[tokio::test]
async fn malformed_json_returns_the_json_error_envelope() {
    let app = app(new_state());
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from("{not json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(res).await["error"].is_string());
}

#[tokio::test]
async fn unknown_routes_and_ready_endpoint() {
    let app = app(new_state());
    let missing = get(app.clone(), "/api/nope").await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert!(body_json(missing).await["error"].is_string());

    let ready = get(app, "/api/ready").await;
    assert_eq!(ready.status(), StatusCode::OK);
    let body = body_json(ready).await;
    assert_eq!(body["status"], "ready");
    assert_eq!(body["demoMode"], true);
}

#[tokio::test]
async fn posts_feed_returns_all_months_when_month_is_omitted() {
    let app = app(new_state());
    let all = body_json(get(app.clone(), "/api/posts").await).await;
    assert_eq!(all.as_array().unwrap().len(), 6);
    let only_two = body_json(get(app, "/api/posts?month=2").await).await;
    assert_eq!(only_two.as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn registration_can_be_disabled() {
    let state = new_state();
    state.write().await.allow_registration = false;
    let app = app(state);
    let res = send(
        app,
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Nope", "email": "nope@test.local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn login_is_rate_limited_after_repeated_failures() {
    let app = app(new_state());
    for _ in 0..10 {
        let res = send(
            app.clone(),
            "POST",
            "/api/auth/login",
            serde_json::json!({
                "email": "owner@studio.local", "password": "definitely-wrong"
            }),
        )
        .await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
    let locked = send(
        app,
        "POST",
        "/api/auth/login",
        serde_json::json!({
            "email": "owner@studio.local", "password": "demo1234"
        }),
    )
    .await;
    assert_eq!(locked.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(body_json(locked).await["error"]
        .as_str()
        .unwrap()
        .contains("too many"));
}

#[tokio::test]
async fn sync_requires_a_connected_platform() {
    let app = app(new_state());
    let res = send(
        app,
        "POST",
        "/api/platforms/youtube/sync",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn connect_is_blocked_when_the_provider_uses_oauth() {
    let state = new_state();
    state
        .write()
        .await
        .oauth
        .set_credentials("meta", "app-id", "secret");
    let app = app(state);
    let res = send(
        app,
        "POST",
        "/api/platforms/meta/connect",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn profile_rename_rewrites_post_locks() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({ "authRequired": true }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);
    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;

    let locked = send_auth(
        app.clone(),
        "POST",
        "/api/posts/p-desk/lock",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(body_json(locked).await["lockedBy"], "Studio Owner");

    let renamed = send_auth(
        app.clone(),
        "PATCH",
        "/api/auth/profile",
        Some(&owner),
        serde_json::json!({
            "name": "Studio Boss"
        }),
    )
    .await;
    assert_eq!(renamed.status(), StatusCode::OK);

    // reads need the session too now that authRequired is on
    let posts = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/posts?month=2",
            Some(&owner),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    let post = posts
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "p-desk")
        .unwrap();
    assert_eq!(
        post["lockedBy"], "Studio Boss",
        "lock must follow the rename"
    );

    // the renamed account can still release its own lock
    let unlocked = send_auth(
        app,
        "POST",
        "/api/posts/p-desk/unlock",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(unlocked.status(), StatusCode::OK);
    assert!(body_json(unlocked).await["lockedBy"].is_null());
}

#[tokio::test]
async fn last_administrator_cannot_be_removed_or_deleted() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({ "authRequired": true }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);
    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;

    let remove_user = send_auth(
        app.clone(),
        "DELETE",
        "/api/setup/users/Studio%20Owner",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(remove_user.status(), StatusCode::BAD_REQUEST);

    let delete_self = send_auth(
        app.clone(),
        "DELETE",
        "/api/auth/accounts/owner%40studio.local",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(delete_self.status(), StatusCode::BAD_REQUEST);

    // a non-admin account can be deleted by the owner
    let editor_login = login(app.clone(), "editor@studio.local", "demo1234").await;
    let delete_editor = send_auth(
        app.clone(),
        "DELETE",
        "/api/auth/accounts/editor%40studio.local",
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(delete_editor.status(), StatusCode::OK);
    let _ = editor_login;

    // The directory entry goes with the account (no phantom users).
    let setup = body_json(get(app, "/api/setup").await).await;
    assert!(!setup["users"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["name"] == "Editor Earn"));
}

#[tokio::test]
async fn admin_recovery_token_bypasses_role_checks() {
    let state = new_state();
    state.write().await.admin_token = Some("recovery-token-123456".into());
    let app = app(state);
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({ "authRequired": true }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);

    // anonymous write is blocked…
    let blocked = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({ "workspaceName": "Nope" }),
    )
    .await;
    assert_eq!(blocked.status(), StatusCode::UNAUTHORIZED);

    // …unless the operator recovery token is presented
    let res = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/setup")
                .header("content-type", "application/json")
                .header("x-admin-token", "recovery-token-123456")
                .body(Body::from(
                    serde_json::json!({ "workspaceName": "Recovered" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body_json(res).await["workspaceName"], "Recovered");
}

#[tokio::test]
async fn store_snapshot_round_trips() {
    let mut store = content_planner_server::store::Store::seed();
    store.posts.push(content_planner_server::model::Post {
        id: "p-snapshot".into(),
        month: 4,
        topic: "Persisted".into(),
        pillar: "Pillar I".into(),
        format: "Short-video".into(),
        goal: "10K subscribe".into(),
        date: Some("2026-04-01".into()),
        time: "09:00".into(),
        status: "Start".into(),
        hook: String::new(),
        caption: String::new(),
        cta: String::new(),
        hashtag_group: String::new(),
        hashtags: vec![],
        image_url: String::new(),
        note: String::new(),
        done: false,
        platforms: vec![],
        locked_by: None,
    });
    let token = store.open_session("owner@studio.local");

    let path = std::env::temp_dir().join(format!("cp-snapshot-{}.json", std::process::id()));
    store.save_to(&path).unwrap();
    let raw = std::fs::read_to_string(&path).unwrap();
    let loaded = content_planner_server::store::Store::load_or_seed(&path).unwrap();
    std::fs::remove_file(&path).ok();

    assert!(loaded
        .posts
        .iter()
        .any(|p| p.id == "p-snapshot" && p.topic == "Persisted"));
    assert!(loaded.accounts.contains_key("owner@studio.local"));
    // Sessions persist as hashes only: a leaked snapshot must not contain a
    // usable bearer token, but a restart keeps the user signed in.
    assert_eq!(loaded.sessions.len(), 1);
    assert!(!raw.contains(&token), "token leaked into the snapshot");
    assert!(
        raw.contains(&content_planner_server::store::session_key(&token)),
        "only the hash is persisted"
    );
}

// ---- CMS: content studio ----

async fn create_content(
    app: axum::Router,
    token: Option<&str>,
    body: serde_json::Value,
) -> serde_json::Value {
    let res = send_auth(app, "POST", "/api/content", token, body).await;
    assert_eq!(res.status(), StatusCode::OK);
    body_json(res).await
}

#[tokio::test]
async fn content_studio_lifecycle() {
    let app = app(new_state());

    // seeded rows: 3 summaries, sorted by updatedAt desc
    let listed = body_json(get(app.clone(), "/api/content").await).await;
    let rows = listed.as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert!(rows
        .iter()
        .any(|c| c["slug"] == "why-slow-mornings-changed-our-year" && c["status"] == "published"));

    // full item strips revision bodies
    let welcome = body_json(get(app.clone(), "/api/content/c-welcome").await).await;
    assert!(welcome["body"].as_str().unwrap().contains("slow mornings"));
    assert_eq!(welcome["revisions"][0]["body"], "");
    assert!(welcome["revisions"][0]["revision"].as_u64().is_some());

    // create draft (demo mode: no token, author falls back to the workspace owner)
    let created = create_content(
        app.clone(),
        None,
        serde_json::json!({
            "title": "New Studio Piece", "slug": "new-studio-piece", "kind": "article",
            "status": "draft", "body": "Hello **world**.", "excerpt": "First words.",
            "heroImageUrl": "", "tags": ["test"], "seoTitle": "", "seoDescription": "",
            "author": "", "createdAt": "", "updatedAt": "", "publishedAt": null,
            "scheduledFor": null, "version": 0, "revisions": []
        }),
    )
    .await;
    assert!(created["id"].as_str().unwrap().starts_with("c-"));
    assert_eq!(created["version"], 1);
    assert_eq!(created["author"], "Studio Owner");

    // edit bumps the version and stores a pre-edit revision
    let id = created["id"].as_str().unwrap().to_string();
    let updated = send(
        app.clone(),
        "PATCH",
        &format!("/api/content/{id}"),
        serde_json::json!({ "version": 1, "title": "New Studio Piece v2", "note": "Retitle" }),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::OK);
    let updated = body_json(updated).await;
    assert_eq!(updated["version"], 2);
    assert_eq!(updated["title"], "New Studio Piece v2");
    assert_eq!(updated["revisions"].as_array().unwrap().len(), 1);
    assert_eq!(updated["revisions"][0]["title"], "New Studio Piece");

    // stale version → 409
    let stale = send(
        app.clone(),
        "PATCH",
        &format!("/api/content/{id}"),
        serde_json::json!({ "version": 1, "title": "Conflicting" }),
    )
    .await;
    assert_eq!(stale.status(), StatusCode::CONFLICT);
    assert!(body_json(stale).await["error"]
        .as_str()
        .unwrap()
        .contains("version"));

    // publish → public delivery exposes it
    let published = send(
        app.clone(),
        "POST",
        &format!("/api/content/{id}/publish"),
        serde_json::json!({ "version": 2 }),
    )
    .await;
    assert_eq!(published.status(), StatusCode::OK);
    let published = body_json(published).await;
    assert_eq!(published["status"], "published");
    assert!(published["publishedAt"].is_string());
    assert_eq!(published["version"], 3);

    let public_list = body_json(get(app.clone(), "/api/public/content").await).await;
    assert!(public_list
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["id"] == id));
    let public_item =
        body_json(get(app.clone(), "/api/public/content/new-studio-piece").await).await;
    assert_eq!(public_item["title"], "New Studio Piece v2");
    assert_eq!(public_item["body"], "Hello **world**.");

    // unpublish removes it from public delivery
    let unpublished = send(
        app.clone(),
        "POST",
        &format!("/api/content/{id}/unpublish"),
        serde_json::json!({ "version": 3 }),
    )
    .await;
    assert_eq!(unpublished.status(), StatusCode::OK);
    assert_eq!(body_json(unpublished).await["status"], "draft");
    let gone = get(app, "/api/public/content/new-studio-piece").await;
    assert_eq!(gone.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn content_revisions_and_restore() {
    let app = app(new_state());
    let created = create_content(
        app.clone(),
        None,
        serde_json::json!({
            "title": "Revision Test", "slug": "", "kind": "note", "status": "draft",
            "body": "one", "excerpt": "", "tags": []
        }),
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();
    assert!(!created["slug"].as_str().unwrap().is_empty());

    for (version, body) in [(1, "two"), (2, "three")] {
        let res = send(
            app.clone(),
            "PATCH",
            &format!("/api/content/{id}"),
            serde_json::json!({ "version": version, "body": body }),
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    let revisions =
        body_json(get(app.clone(), &format!("/api/content/{id}/revisions")).await).await;
    let revisions = revisions.as_array().unwrap();
    assert_eq!(revisions.len(), 2);
    assert_eq!(revisions[0]["revision"], 2); // newest first
    assert_eq!(revisions[1]["body"], "one");

    // restore revision 1 (body "one") at the current version 3
    let restored = send(
        app.clone(),
        "POST",
        &format!("/api/content/{id}/revisions/1/restore"),
        serde_json::json!({ "version": 3 }),
    )
    .await;
    assert_eq!(restored.status(), StatusCode::OK);
    let restored = body_json(restored).await;
    assert_eq!(restored["body"], "one");
    assert_eq!(restored["status"], "draft");
    assert_eq!(restored["version"], 4);

    // restoring a missing revision → 404
    let missing = send(
        app,
        "POST",
        &format!("/api/content/{id}/revisions/99/restore"),
        serde_json::json!({ "version": 4 }),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn content_schedule_and_validation() {
    let app = app(new_state());
    let created = create_content(
        app.clone(),
        None,
        serde_json::json!({ "title": "Scheduled Piece", "slug": "scheduled-piece", "kind": "article", "status": "draft", "body": "x" }),
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();

    // past schedule rejected
    let past = send(
        app.clone(),
        "POST",
        &format!("/api/content/{id}/schedule"),
        serde_json::json!({ "version": 1, "scheduledFor": "2020-01-01T00:00:00Z" }),
    )
    .await;
    assert_eq!(past.status(), StatusCode::BAD_REQUEST);

    // future schedule accepted and not public yet
    let future = chrono::Utc::now() + chrono::Duration::days(3);
    let scheduled = send(
        app.clone(),
        "POST",
        &format!("/api/content/{id}/schedule"),
        serde_json::json!({ "version": 1, "scheduledFor": future.to_rfc3339() }),
    )
    .await;
    assert_eq!(scheduled.status(), StatusCode::OK);
    assert_eq!(body_json(scheduled).await["status"], "scheduled");
    let public_now = body_json(get(app.clone(), "/api/public/content").await).await;
    assert!(!public_now.as_array().unwrap().iter().any(|c| c["id"] == id));

    // validation
    let no_title = send(
        app.clone(),
        "POST",
        "/api/content",
        serde_json::json!({ "title": "  ", "slug": "x", "kind": "article", "status": "draft", "body": "" }),
    )
    .await;
    assert_eq!(no_title.status(), StatusCode::BAD_REQUEST);
    let bad_slug = send(
        app.clone(),
        "POST",
        "/api/content",
        serde_json::json!({ "title": "Bad Slug", "slug": "Not A Slug", "kind": "article", "status": "draft", "body": "" }),
    )
    .await;
    assert_eq!(bad_slug.status(), StatusCode::BAD_REQUEST);
    let bad_kind = send(
        app.clone(),
        "POST",
        "/api/content",
        serde_json::json!({ "title": "Bad Kind", "slug": "bad-kind", "kind": "tweet", "status": "draft", "body": "" }),
    )
    .await;
    assert_eq!(bad_kind.status(), StatusCode::BAD_REQUEST);
    let too_many_tags = send(
        app.clone(),
        "POST",
        "/api/content",
        serde_json::json!({
            "title": "Many Tags", "slug": "many-tags", "kind": "article", "status": "draft", "body": "",
            "tags": (0..21).map(|i| format!("tag{i}")).collect::<Vec<_>>()
        }),
    )
    .await;
    assert_eq!(too_many_tags.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn duplicate_content_gets_a_unique_slug_and_draft_status() {
    let app = app(new_state());
    let copy = send(
        app,
        "POST",
        "/api/content/c-welcome/duplicate",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(copy.status(), StatusCode::OK);
    let copy = body_json(copy).await;
    assert_eq!(copy["status"], "draft");
    assert!(copy["title"].as_str().unwrap().contains("(copy)"));
    assert_ne!(copy["slug"], "why-slow-mornings-changed-our-year");
    assert!(copy["publishedAt"].is_null());
    assert_eq!(copy["revisions"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn client_role_gets_a_content_studio_path() {
    let app = app(new_state());
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({ "authRequired": true }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);
    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;

    // admin onboards a client without a password → one-time temp password
    let created = send_auth(
        app.clone(),
        "POST",
        "/api/auth/accounts",
        Some(&owner),
        serde_json::json!({ "name": "Client Cam", "email": "cam@test.local", "role": "Client" }),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let created = body_json(created).await;
    let temp = created["temporaryPassword"].as_str().unwrap().to_string();
    assert!(temp.len() >= 8);
    assert_eq!(created["account"]["role"], "Client");

    // the client can log in and use the studio…
    let client = login(app.clone(), "cam@test.local", &temp).await;
    let written = send_auth(
        app.clone(),
        "POST",
        "/api/content",
        Some(&client),
        serde_json::json!({ "title": "Client Draft", "slug": "client-draft", "kind": "article", "status": "draft", "body": "hi" }),
    )
    .await;
    assert_eq!(written.status(), StatusCode::OK);
    let written = body_json(written).await;
    assert_eq!(written["author"], "Client Cam");
    let published = send_auth(
        app.clone(),
        "POST",
        &format!("/api/content/{}/publish", written["id"].as_str().unwrap()),
        Some(&client),
        serde_json::json!({ "version": 1 }),
    )
    .await;
    assert_eq!(published.status(), StatusCode::OK);

    // …but not the workspace settings or deletion
    let denied_setup = send_auth(
        app.clone(),
        "PATCH",
        "/api/setup",
        Some(&client),
        serde_json::json!({ "owner": "Cam" }),
    )
    .await;
    assert_eq!(denied_setup.status(), StatusCode::FORBIDDEN);
    let denied_delete = send_auth(
        app.clone(),
        "DELETE",
        &format!("/api/content/{}", written["id"].as_str().unwrap()),
        Some(&client),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(denied_delete.status(), StatusCode::FORBIDDEN);

    // the owner can delete
    let owner_delete = send_auth(
        app.clone(),
        "DELETE",
        &format!("/api/content/{}", written["id"].as_str().unwrap()),
        Some(&owner),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(owner_delete.status(), StatusCode::OK);

    // account creation guards
    let editor = login(app.clone(), "editor@studio.local", "demo1234").await;
    let denied = send_auth(
        app.clone(),
        "POST",
        "/api/auth/accounts",
        Some(&editor),
        serde_json::json!({ "name": "Nope", "email": "nope@test.local", "role": "Viewer" }),
    )
    .await;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    let dup = send_auth(
        app.clone(),
        "POST",
        "/api/auth/accounts",
        Some(&owner),
        serde_json::json!({ "name": "Other Cam", "email": "cam@test.local", "role": "Viewer" }),
    )
    .await;
    assert_eq!(dup.status(), StatusCode::CONFLICT);
    let bad_role = send_auth(
        app,
        "POST",
        "/api/auth/accounts",
        Some(&owner),
        serde_json::json!({ "name": "Roleless", "email": "roleless@test.local", "role": "Ghost" }),
    )
    .await;
    assert_eq!(bad_role.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn scheduled_content_publishes_when_due() {
    let state = new_state();
    let app = app(state.clone());

    // force the seeded scheduled row into the past
    {
        let mut store = state.write().await;
        let item = store
            .content
            .iter_mut()
            .find(|c| c.status == "scheduled")
            .unwrap();
        item.scheduled_for = Some((chrono::Utc::now() - chrono::Duration::minutes(1)).to_rfc3339());
    }

    let public = body_json(get(app.clone(), "/api/public/content").await).await;
    assert!(public
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["slug"] == "march-launch-announcement"));

    let item = body_json(get(app, "/api/content/c-march").await).await;
    assert_eq!(item["status"], "published");
    assert!(item["publishedAt"].is_string());
    assert!(item["scheduledFor"].is_null());
}

#[tokio::test]
async fn content_without_version_is_rejected() {
    let app = app(new_state());
    let res = send(
        app,
        "PATCH",
        "/api/content/c-welcome",
        serde_json::json!({ "title": "No version" }),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(res).await["error"]
        .as_str()
        .unwrap()
        .contains("version"));
}

#[tokio::test]
async fn demo_mode_content_is_attributed_to_the_session_account() {
    let app = app(new_state());
    // authRequired is off, but a valid bearer token still identifies the author
    let editor = login(app.clone(), "editor@studio.local", "demo1234").await;
    let created = send_auth(
        app,
        "POST",
        "/api/content",
        Some(&editor),
        serde_json::json!({ "title": "Attributed", "kind": "note", "status": "draft", "body": "hi" }),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    assert_eq!(body_json(created).await["author"], "Editor Earn");
}

#[tokio::test]
async fn spa_static_serving_api_precedence_and_traversal_guard() {
    use content_planner_server::app_with_static;

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("cp-static-{unique}"));
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(
        dir.join("index.html"),
        "<!doctype html><div id=\"app\"></div>",
    )
    .unwrap();
    std::fs::write(dir.join("assets/app.js"), "console.log(1)").unwrap();
    std::fs::write(dir.join("secret.txt"), "top secret").unwrap();

    let app = app_with_static(new_state(), Some(dir.clone()));

    // index shell at /
    let res = get(app.clone(), "/").await;
    assert_eq!(res.status(), StatusCode::OK);
    let content_type = res.headers()["content-type"].to_str().unwrap().to_string();
    assert!(content_type.starts_with("text/html"));
    let csp = res.headers()["content-security-policy"]
        .to_str()
        .unwrap()
        .to_string();
    assert!(csp.contains("default-src 'self'"));
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&bytes).contains("id=\"app\""));

    // hashed assets: correct MIME + immutable cache
    let res = get(app.clone(), "/assets/app.js").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()["content-type"],
        "text/javascript; charset=utf-8"
    );
    assert!(res.headers()["cache-control"]
        .to_str()
        .unwrap()
        .contains("immutable"));

    // extension-less client route falls back to the shell
    let res = get(app.clone(), "/some/deep/route").await;
    assert_eq!(res.status(), StatusCode::OK);

    // a missing asset is a JSON 404, not the shell
    let res = get(app.clone(), "/assets/missing.js").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert!(body_json(res).await["error"].is_string());

    // unknown API routes stay JSON under the static fallback
    let res = get(app.clone(), "/api/nope").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert!(body_json(res).await["error"].is_string());

    // API CSP stays locked down while the app gets its own
    let res = get(app.clone(), "/api/setup").await;
    assert!(res.headers()["content-security-policy"]
        .to_str()
        .unwrap()
        .contains("default-src 'none'"));

    // path traversal is rejected
    let res = get(app.clone(), "/..%2Fsecret.txt").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // non-GET on static paths is 405
    let res = send(app, "POST", "/unknown", serde_json::json!({})).await;
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn fresh_store_has_no_demo_data() {
    let fresh = content_planner_server::store::Store::fresh();
    assert!(fresh.posts.is_empty());
    assert!(fresh.content.is_empty());
    assert!(fresh.ideas.is_empty());
    assert!(fresh.hashtags.is_empty());
    assert!(fresh.metrics.is_empty());
    assert!(fresh.txns.is_empty());
    // Platform stubs stay, but disconnected and without demo handles.
    assert_eq!(fresh.workspaces.len(), 1);
    let fresh_conns = &fresh.workspaces[0].connections;
    assert_eq!(fresh_conns.len(), 3);
    assert!(fresh_conns
        .iter()
        .all(|c| c.status == "disconnected" && c.handle.is_empty()));
    assert!(fresh.accounts.is_empty());
    assert!(fresh.sessions.is_empty());
    assert!(fresh.setup.users.is_empty());
    assert!(fresh.setup.owner.is_empty());
    assert!(fresh.workspaces[0].brand.channel.is_empty());
    assert!(!fresh.demo_mode);
    assert!(!fresh.allow_registration);

    // Config defaults the app needs to work stay in place.
    assert!(!fresh.setup.platforms.is_empty());
    assert!(!fresh.setup.statuses.is_empty());
    assert!(!fresh.setup.roles.is_empty());

    // The demo seed is still available for DEMO_MODE=1.
    let seeded = content_planner_server::store::Store::seed();
    assert!(!seeded.posts.is_empty());
    assert!(!seeded.content.is_empty());
    assert!(!seeded.accounts.is_empty());
}

#[tokio::test]
async fn workspace_name_is_editable_and_validated() {
    let app = app(new_state());

    let patched = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({ "workspaceName": "  Acme Studio  " }),
    )
    .await;
    assert_eq!(patched.status(), StatusCode::OK);
    assert_eq!(body_json(patched).await["workspaceName"], "Acme Studio");

    let read = body_json(get(app.clone(), "/api/setup").await).await;
    assert_eq!(read["workspaceName"], "Acme Studio");

    let empty = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({ "workspaceName": "   " }),
    )
    .await;
    assert_eq!(empty.status(), StatusCode::BAD_REQUEST);

    let too_long = send(
        app,
        "PATCH",
        "/api/setup",
        serde_json::json!({ "workspaceName": "x".repeat(81) }),
    )
    .await;
    assert_eq!(too_long.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn brand_fonts_upload_list_and_delete() {
    let dir = std::env::temp_dir().join(format!("cp-fonts-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let state = new_state();
    state.write().await.fonts_dir = Some(dir.clone());
    let app = app(state);

    // base64("wOF2fake") — a minimal valid-looking woff2 payload.
    let uploaded = send(
        app.clone(),
        "POST",
        "/api/brand/fonts",
        serde_json::json!({ "name": "Test Font.woff2", "data": "d09GMmZha2U=" }),
    )
    .await;
    assert_eq!(uploaded.status(), StatusCode::OK);
    let list = body_json(uploaded).await;
    assert_eq!(list[0]["family"], "Test Font");
    assert_eq!(list[0]["name"], "test-font.woff2");
    assert_eq!(list[0]["url"], "/api/brand/fonts/test-font.woff2/file");

    let file = get(app.clone(), "/api/brand/fonts/test-font.woff2/file").await;
    assert_eq!(file.status(), StatusCode::OK);
    assert_eq!(file.headers()["content-type"], "font/woff2");
    assert!(file.headers()["cache-control"]
        .to_str()
        .unwrap()
        .contains("immutable"));

    let listed = body_json(get(app.clone(), "/api/brand/fonts").await).await;
    assert_eq!(listed.as_array().unwrap().len(), 1);

    // Wrong magic bytes must be rejected.
    let bad = send(
        app.clone(),
        "POST",
        "/api/brand/fonts",
        serde_json::json!({ "name": "evil.woff2", "data": "bm9wZQ==" }),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);

    let deleted = send(
        app.clone(),
        "DELETE",
        "/api/brand/fonts/test-font.woff2",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::OK);
    assert!(body_json(deleted).await.as_array().unwrap().is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn boot_migration_grants_new_permissions_to_owner() {
    // Simulate a snapshot written by an older version: 12 permissions only.
    let state = new_state();
    let token = {
        let mut store = state.write().await;
        let owner = store
            .setup
            .roles
            .iter_mut()
            .find(|r| r.name == "Owner")
            .unwrap();
        owner.permissions.truncate(12);
        store.apply_env();
        // `apply_env` turns authRequired on and purges the demo accounts (no
        // DEMO_MODE), so create a real account + session for the assertions.
        use content_planner_server::store::{hash_password, Account};
        store.accounts.insert(
            "owner@example.com".into(),
            Account {
                name: "Studio Owner".into(),
                email: "owner@example.com".into(),
                password_hash: hash_password("longenough123"),
                plan: "free".into(),
                last_active: None,
            },
        );
        store.open_session("owner@example.com")
    };
    let app = app(state);
    // `apply_env` turns authRequired on, so the directory is only visible to a
    // signed-in caller.
    let setup = body_json(
        send_auth(
            app,
            "GET",
            "/api/setup",
            Some(&token),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    let owner = setup["roles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "Owner")
        .unwrap();
    let perms: Vec<&str> = owner["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    assert!(perms.contains(&"campaigns.write"));
    assert!(perms.contains(&"campaigns.delete"));
    assert_eq!(perms.len(), 15);
}

#[tokio::test]
async fn campaign_setup_schedule_and_cleanup() {
    let app = app(new_state());

    // The seed ships a campaign linked to a content item.
    let listed = body_json(get(app.clone(), "/api/campaigns").await).await;
    assert_eq!(listed.as_array().unwrap().len(), 2);
    assert_eq!(listed[0]["name"], "Spring Launch");
    assert_eq!(listed[0]["contentIds"][0], "c-march");

    // Create with brief + schedule window.
    let created = send(
        app.clone(),
        "POST",
        "/api/campaigns",
        serde_json::json!({
            "name": "Summer Push",
            "objective": "Push the summer series",
            "status": "active",
            "startDate": "2026-07-01",
            "endDate": "2026-07-31",
            "platforms": ["Instagram", "Instagram", ""],
            "hashtags": ["summer"],
            "budget": 5000,
            "goalMetric": "views",
            "goalTarget": 100000,
            "contentIds": ["c-welcome"]
        }),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let campaign = body_json(created).await;
    let id = campaign["id"].as_str().unwrap().to_string();
    assert_eq!(campaign["platforms"].as_array().unwrap().len(), 1); // deduped
    assert_eq!(campaign["owner"], "Studio Owner"); // from the demo session
    assert_eq!(campaign["contentIds"][0], "c-welcome");

    // Schedule window validation.
    let bad_range = send(
        app.clone(),
        "PATCH",
        &format!("/api/campaigns/{id}"),
        serde_json::json!({ "endDate": "2026-06-01" }),
    )
    .await;
    assert_eq!(bad_range.status(), StatusCode::BAD_REQUEST);

    let bad_status = send(
        app.clone(),
        "PATCH",
        &format!("/api/campaigns/{id}"),
        serde_json::json!({ "status": "archived" }),
    )
    .await;
    assert_eq!(bad_status.status(), StatusCode::BAD_REQUEST);

    let unknown_content = send(
        app.clone(),
        "PATCH",
        &format!("/api/campaigns/{id}"),
        serde_json::json!({ "contentIds": ["c-nope"] }),
    )
    .await;
    assert_eq!(unknown_content.status(), StatusCode::BAD_REQUEST);

    let moved = send(
        app.clone(),
        "PATCH",
        &format!("/api/campaigns/{id}"),
        serde_json::json!({ "endDate": "2026-08-15", "budget": 7500 }),
    )
    .await;
    assert_eq!(moved.status(), StatusCode::OK);
    let moved_body = body_json(moved).await;
    assert_eq!(moved_body["endDate"], "2026-08-15");
    assert_eq!(moved_body["budget"], 7500.0);

    let deleted = send(
        app,
        "DELETE",
        &format!("/api/campaigns/{id}"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::OK);
}

#[tokio::test]
async fn campaign_writes_need_permission_when_auth_is_on() {
    let app = app(new_state());
    // Turn auth on, then try to create anonymously: 401.
    let enabled = send(
        app.clone(),
        "PATCH",
        "/api/setup",
        serde_json::json!({ "authRequired": true }),
    )
    .await;
    assert_eq!(enabled.status(), StatusCode::OK);

    let anonymous = send(
        app.clone(),
        "POST",
        "/api/campaigns",
        serde_json::json!({ "name": "Nope" }),
    )
    .await;
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);

    // A Viewer cannot create campaigns…
    let owner = login(app.clone(), "owner@studio.local", "demo1234").await;
    let added = send_auth(
        app.clone(),
        "POST",
        "/api/setup/users",
        Some(&owner),
        serde_json::json!({ "name": "Viewer V", "role": "Viewer" }),
    )
    .await;
    assert_eq!(added.status(), StatusCode::OK);
    let viewer = send(
        app.clone(),
        "POST",
        "/api/auth/register",
        serde_json::json!({
            "name": "Viewer V", "email": "viewer-v@test.local", "password": "longenough123"
        }),
    )
    .await;
    assert_eq!(viewer.status(), StatusCode::OK);
    let viewer_token = body_json(viewer).await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let forbidden = send_auth(
        app.clone(),
        "POST",
        "/api/campaigns",
        Some(&viewer_token),
        serde_json::json!({ "name": "Viewer Campaign" }),
    )
    .await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    // …but an Editor can.
    let editor = login(app.clone(), "editor@studio.local", "demo1234").await;
    let allowed = send_auth(
        app,
        "POST",
        "/api/campaigns",
        Some(&editor),
        serde_json::json!({ "name": "Editor Campaign" }),
    )
    .await;
    assert_eq!(allowed.status(), StatusCode::OK);
}

// ---- workspaces ----

async fn send_ws(
    app: axum::Router,
    method: &str,
    uri: &str,
    workspace: &str,
    body: serde_json::Value,
) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .header("x-workspace-id", workspace)
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

async fn get_ws(app: axum::Router, uri: &str, workspace: &str) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .uri(uri)
            .header("x-workspace-id", workspace)
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn workspaces_crud_scopes_brand_and_platforms() {
    let state = new_state();
    let app = app(state.clone());

    // Seeded deployment: exactly one workspace.
    let list = body_json(get(app.clone(), "/api/workspaces").await).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(
        list[0]["id"],
        content_planner_server::store::DEFAULT_WORKSPACE_ID
    );

    // Create a second workspace.
    let created = body_json(
        send(
            app.clone(),
            "POST",
            "/api/workspaces",
            serde_json::json!({ "name": "Client B" }),
        )
        .await,
    )
    .await;
    let ws_b = created["id"].as_str().unwrap().to_string();
    assert!(ws_b.starts_with("ws-"));
    assert_eq!(created["name"], "Client B");
    assert_eq!(created["connected"], 0);
    assert_eq!(created["total"], 3);

    // Rename it.
    let renamed = body_json(
        send(
            app.clone(),
            "PATCH",
            &format!("/api/workspaces/{ws_b}"),
            serde_json::json!({ "name": "Client B (renamed)" }),
        )
        .await,
    )
    .await;
    assert_eq!(renamed["name"], "Client B (renamed)");

    // Connecting a platform in workspace B never touches the default one.
    let connected = send_ws(
        app.clone(),
        "POST",
        "/api/platforms/tiktok/connect",
        &ws_b,
        serde_json::json!({ "handle": "@clientb" }),
    )
    .await;
    assert_eq!(connected.status(), StatusCode::OK);

    let b_platforms = body_json(get_ws(app.clone(), "/api/platforms", &ws_b).await).await;
    let b_tiktok = b_platforms
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "tiktok")
        .unwrap();
    assert_eq!(b_tiktok["status"], "connected");
    assert_eq!(b_tiktok["handle"], "@clientb");

    let default_platforms = body_json(get(app.clone(), "/api/platforms").await).await;
    let d_tiktok = default_platforms
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "tiktok")
        .unwrap();
    assert_eq!(d_tiktok["status"], "disconnected");

    // Brand identity is per workspace too.
    let saved = send_ws(
        app.clone(),
        "PATCH",
        "/api/brand",
        &ws_b,
        serde_json::json!({ "channel": "Client B Channel" }),
    )
    .await;
    assert_eq!(saved.status(), StatusCode::OK);
    let b_brand = body_json(get_ws(app.clone(), "/api/brand", &ws_b).await).await;
    assert_eq!(b_brand["channel"], "Client B Channel");
    let d_brand = body_json(get(app.clone(), "/api/brand").await).await;
    assert_eq!(d_brand["channel"], "Studio Channel");

    // Provider tokens are keyed per workspace: disconnecting in B must not
    // touch the default workspace's token for the same platform.
    {
        let mut store = state.write().await;
        store.platform_tokens.insert(
            content_planner_server::store::Store::token_key(&ws_b, "tiktok"),
            "tok-b".into(),
        );
        store.platform_tokens.insert(
            content_planner_server::store::Store::token_key(
                content_planner_server::store::DEFAULT_WORKSPACE_ID,
                "tiktok",
            ),
            "tok-default".into(),
        );
    }
    let disconnected = send_ws(
        app.clone(),
        "POST",
        "/api/platforms/tiktok/disconnect",
        &ws_b,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(disconnected.status(), StatusCode::OK);
    {
        let store = state.read().await;
        let key = |ws: &str| content_planner_server::store::Store::token_key(ws, "tiktok");
        assert!(!store.platform_tokens.contains_key(&key(&ws_b)));
        assert!(store
            .platform_tokens
            .contains_key(&key(content_planner_server::store::DEFAULT_WORKSPACE_ID)));
    }

    // The setup payload reports the active workspace's name.
    let setup_b = body_json(get_ws(app.clone(), "/api/setup", &ws_b).await).await;
    assert_eq!(setup_b["workspaceName"], "Client B (renamed)");

    // Delete workspace B: its token disappears with it.
    let deleted = send(
        app.clone(),
        "DELETE",
        &format!("/api/workspaces/{ws_b}"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::OK);
    let list = body_json(get(app.clone(), "/api/workspaces").await).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    let store = state.read().await;
    assert!(store.workspace(&ws_b).is_none());
    assert!(store
        .platform_tokens
        .contains_key(&content_planner_server::store::Store::token_key(
            content_planner_server::store::DEFAULT_WORKSPACE_ID,
            "tiktok"
        )));
}

#[tokio::test]
async fn last_workspace_cannot_be_deleted_and_rename_validates() {
    let state = new_state();
    let app = app(state);
    let default_id = content_planner_server::store::DEFAULT_WORKSPACE_ID;

    let refused = send(
        app.clone(),
        "DELETE",
        &format!("/api/workspaces/{default_id}"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(refused.status(), StatusCode::BAD_REQUEST);

    let empty = send(
        app.clone(),
        "POST",
        "/api/workspaces",
        serde_json::json!({ "name": "   " }),
    )
    .await;
    assert_eq!(empty.status(), StatusCode::BAD_REQUEST);

    let unknown = send(
        app,
        "PATCH",
        "/api/workspaces/ws-nope",
        serde_json::json!({ "name": "X" }),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn oauth_start_binds_the_requested_workspace() {
    let state = new_state();
    let app = app(state.clone());

    let created = body_json(
        send(
            app.clone(),
            "POST",
            "/api/workspaces",
            serde_json::json!({ "name": "OAuth Workspace" }),
        )
        .await,
    )
    .await;
    let ws = created["id"].as_str().unwrap().to_string();

    {
        let mut store = state.write().await;
        store
            .oauth
            .set_credentials("meta", "app-id-123", "secret-xyz");
    }

    let start = body_json(get_ws(app, "/api/oauth/meta/start", &ws).await).await;
    assert_eq!(start["mode"], "redirect");
    let url = start["url"].as_str().unwrap();
    let state_id = url
        .split("state=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .unwrap()
        .to_string();

    let store = state.read().await;
    let entry = store.oauth_states.get(&state_id).unwrap();
    assert_eq!(entry.workspace_id, ws);
}

#[tokio::test]
async fn v1_snapshot_migrates_into_default_workspace() {
    let seed = content_planner_server::store::Store::seed();
    let mut v1: serde_json::Value =
        serde_json::from_slice(&seed.snapshot_bytes().unwrap()).unwrap();
    v1["version"] = serde_json::json!(1);
    v1.as_object_mut().unwrap().remove("workspaces");
    v1["brand"] = serde_json::to_value(&seed.workspaces[0].brand).unwrap();
    v1["connections"] = serde_json::to_value(&seed.workspaces[0].connections).unwrap();

    let path = std::env::temp_dir().join(format!(
        "cp-v1-snapshot-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, serde_json::to_vec(&v1).unwrap()).unwrap();

    let store = content_planner_server::store::Store::load_or_seed(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(store.workspaces.len(), 1);
    let ws = &store.workspaces[0];
    assert_eq!(ws.id, content_planner_server::store::DEFAULT_WORKSPACE_ID);
    assert_eq!(ws.brand.channel, "Studio Channel");
    assert_eq!(ws.connections.len(), 3);
    assert!(ws
        .connections
        .iter()
        .any(|c| c.id == "meta" && c.status == "connected"));
}

#[tokio::test]
async fn separate_facebook_and_instagram_slots_migrate_to_one_meta() {
    let seed = content_planner_server::store::Store::seed();
    let mut snapshot: serde_json::Value =
        serde_json::from_slice(&seed.snapshot_bytes().unwrap()).unwrap();
    // Rewrite the workspace the way the pre-Meta app stored it: a connected
    // facebook slot (Page name as handle) plus a separate instagram slot.
    snapshot["workspaces"][0]["connections"] = serde_json::json!([
        {
            "id": "facebook", "status": "connected", "handle": "Meni",
            "externalId": "1207146735819378", "scopes": ["pages_show_list"],
            "tokenType": "long-lived", "expiresAt": "2026-10-18",
            "lastSync": "2026-03-01 06:00", "mediaCount": 7, "note": ""
        },
        {
            "id": "instagram", "status": "connected", "handle": "@meni",
            "externalId": "17841400000000001", "scopes": ["instagram_basic"],
            "tokenType": "long-lived", "expiresAt": "2026-11-01",
            "lastSync": "2026-03-01 06:00", "mediaCount": 3, "note": ""
        }
    ]);

    let path = std::env::temp_dir().join(format!(
        "cp-meta-migration-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, serde_json::to_vec(&snapshot).unwrap()).unwrap();

    let store = content_planner_server::store::Store::load_or_seed(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    let conns = &store.workspaces[0].connections;
    assert_eq!(
        conns.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["meta", "youtube", "tiktok"],
        "instagram is folded into the single Meta connection"
    );
    let meta = conns.iter().find(|c| c.id == "meta").unwrap();
    assert_eq!(meta.status, "connected");
    assert_eq!(
        meta.handle, "https://www.facebook.com/profile.php?id=1207146735819378",
        "the Page name is upgraded to the linkable profile URL"
    );
    assert_eq!(meta.external_id, "1207146735819378");
    assert_eq!(meta.media_count, 7);
}

async fn send_auth_ws(
    app: axum::Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    workspace: &str,
    body: serde_json::Value,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-workspace-id", workspace);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    app.oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn workspaces_are_isolated_per_account() {
    use content_planner_server::model::User;
    use content_planner_server::store::{hash_password, Account};

    let state = new_state();
    let (token_a, token_b) = {
        let mut store = state.write().await;
        store.apply_env(); // auth on, demo accounts purged
        for (name, email) in [
            ("Account A", "a@example.com"),
            ("Account B", "b@example.com"),
        ] {
            store.accounts.insert(
                email.into(),
                Account {
                    name: name.into(),
                    email: email.into(),
                    password_hash: hash_password("longenough123"),
                    plan: "free".into(),
                    last_active: None,
                },
            );
            store.setup.users.push(User {
                name: name.into(),
                role: "Owner".into(),
            });
        }
        (
            store.open_session("a@example.com"),
            store.open_session("b@example.com"),
        )
    };
    let app = app(state);

    // Account A creates a workspace.
    let created = body_json(
        send_auth(
            app.clone(),
            "POST",
            "/api/workspaces",
            Some(&token_a),
            serde_json::json!({ "name": "A space" }),
        )
        .await,
    )
    .await;
    let ws_a = created["id"].as_str().unwrap().to_string();

    // A sees it; B never does.
    let list_a = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/workspaces",
            Some(&token_a),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    assert!(list_a
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w["id"] == ws_a.as_str()));
    let list_b = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/workspaces",
            Some(&token_b),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    assert!(list_b
        .as_array()
        .unwrap()
        .iter()
        .all(|w| w["id"] != ws_a.as_str()));

    // B cannot rename or delete A's workspace (404 — no existence leak).
    let rename = send_auth(
        app.clone(),
        "PATCH",
        &format!("/api/workspaces/{ws_a}"),
        Some(&token_b),
        serde_json::json!({ "name": "hijack" }),
    )
    .await;
    assert_eq!(rename.status(), StatusCode::NOT_FOUND);
    let delete = send_auth(
        app.clone(),
        "DELETE",
        &format!("/api/workspaces/{ws_a}"),
        Some(&token_b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(delete.status(), StatusCode::NOT_FOUND);

    // B cannot target A's workspace with the header either: the connection
    // lands in B's own workspace, and A's stays untouched.
    let connect_b = send_auth_ws(
        app.clone(),
        "POST",
        "/api/platforms/tiktok/connect",
        Some(&token_b),
        &ws_a,
        serde_json::json!({ "handle": "@b-account" }),
    )
    .await;
    assert_eq!(connect_b.status(), StatusCode::OK);

    let platforms_a = body_json(
        send_auth_ws(
            app.clone(),
            "GET",
            "/api/platforms",
            Some(&token_a),
            &ws_a,
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    let a_tiktok = platforms_a
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "tiktok")
        .unwrap();
    assert_eq!(a_tiktok["status"], "disconnected");

    // The brand stays isolated the same way.
    let saved = send_auth_ws(
        app.clone(),
        "PATCH",
        "/api/brand",
        Some(&token_a),
        &ws_a,
        serde_json::json!({ "channel": "A Channel" }),
    )
    .await;
    assert_eq!(saved.status(), StatusCode::OK);
    let brand_b = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/brand",
            Some(&token_b),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    assert_ne!(brand_b["channel"], "A Channel");
}

#[tokio::test]
async fn resolve_workspace_id_respects_ownership() {
    use content_planner_server::store::{blank_workspace_for, Store};

    let mut store = Store::seed();
    store
        .workspaces
        .push(blank_workspace_for("ws-owned", "Owned", "b@example.com"));

    // Owned workspace: only the owner resolves it, others fall back.
    assert_eq!(
        store.resolve_workspace_id(Some("ws-owned"), Some("b@example.com")),
        Some("ws-owned".into())
    );
    assert_ne!(
        store.resolve_workspace_id(Some("ws-owned"), Some("a@example.com")),
        Some("ws-owned".into())
    );
    // Demo/operator scope (None) sees everything.
    assert_eq!(
        store.resolve_workspace_id(Some("ws-owned"), None),
        Some("ws-owned".into())
    );
    // Shared (ownerless) workspaces stay visible to every account.
    assert_eq!(
        store.resolve_workspace_id(
            Some(content_planner_server::store::DEFAULT_WORKSPACE_ID),
            Some("a@example.com")
        ),
        Some(content_planner_server::store::DEFAULT_WORKSPACE_ID.into())
    );
}

// ---- workspace members ----

/// Auth-on state with three accounts (owner, member, outsider) and a live
/// session token for each.
async fn member_test_state() -> (content_planner_server::AppState, String, String, String) {
    use content_planner_server::model::User;
    use content_planner_server::store::{hash_password, Account};

    let state = new_state();
    let (owner, member, other) = {
        let mut store = state.write().await;
        store.apply_env(); // auth on, demo accounts purged
        for (name, email, role) in [
            ("Account Owner", "owner@example.com", "Owner"),
            ("Member Mia", "member@example.com", "Editor"),
            ("Outsider Bob", "other@example.com", "Viewer"),
        ] {
            store.accounts.insert(
                email.into(),
                Account {
                    name: name.into(),
                    email: email.into(),
                    password_hash: hash_password("longenough123"),
                    plan: "free".into(),
                    last_active: None,
                },
            );
            store.setup.users.push(User {
                name: name.into(),
                role: role.into(),
            });
        }
        (
            store.open_session("owner@example.com"),
            store.open_session("member@example.com"),
            store.open_session("other@example.com"),
        )
    };
    (state, owner, member, other)
}

/// Creates a workspace owned by the owner token and returns its id.
async fn create_owned_workspace(app: axum::Router, owner_token: &str, name: &str) -> String {
    let created = body_json(
        send_auth(
            app,
            "POST",
            "/api/workspaces",
            Some(owner_token),
            serde_json::json!({ "name": name }),
        )
        .await,
    )
    .await;
    assert_eq!(created["isOwner"], true);
    created["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn workspace_members_share_access_and_flag_ownership() {
    let (state, owner_token, member_token, _) = member_test_state().await;
    let app = app(state);

    let ws = create_owned_workspace(app.clone(), &owner_token, "Shared space").await;

    // The owner adds the member by email; the response carries the directory
    // name and role.
    let added = send_auth(
        app.clone(),
        "POST",
        &format!("/api/workspaces/{ws}/members"),
        Some(&owner_token),
        serde_json::json!({ "email": "member@example.com" }),
    )
    .await;
    assert_eq!(added.status(), StatusCode::OK);
    let list = body_json(added).await;
    assert_eq!(list["owner"]["email"], "owner@example.com");
    assert_eq!(list["owner"]["name"], "Account Owner");
    assert!(list["owner"].get("role").is_none());
    assert_eq!(list["members"].as_array().unwrap().len(), 1);
    assert_eq!(list["members"][0]["email"], "member@example.com");
    assert_eq!(list["members"][0]["name"], "Member Mia");
    assert_eq!(list["members"][0]["role"], "Editor");

    // The member sees the workspace, flagged as not-owner (exact wire key).
    let list_member = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/workspaces",
            Some(&member_token),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    let entry = list_member
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["id"] == ws.as_str())
        .expect("the member sees the shared workspace");
    assert!(entry.get("isOwner").is_some());
    assert_eq!(entry["isOwner"], false);

    // The owner's own entry is flagged as owner.
    let list_owner = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/workspaces",
            Some(&owner_token),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    let entry = list_owner
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["id"] == ws.as_str())
        .unwrap();
    assert_eq!(entry["isOwner"], true);

    // Workspace-scoped data written by the owner is readable by the member.
    let saved = send_auth_ws(
        app.clone(),
        "PATCH",
        "/api/brand",
        Some(&owner_token),
        &ws,
        serde_json::json!({ "channel": "Shared Channel" }),
    )
    .await;
    assert_eq!(saved.status(), StatusCode::OK);
    let brand = body_json(
        send_auth_ws(
            app.clone(),
            "GET",
            "/api/brand",
            Some(&member_token),
            &ws,
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    assert_eq!(brand["channel"], "Shared Channel");
}

#[tokio::test]
async fn workspace_member_management_is_owner_only() {
    let (state, owner_token, member_token, other_token) = member_test_state().await;
    let app = app(state);

    let ws = create_owned_workspace(app.clone(), &owner_token, "Owner only").await;
    let added = send_auth(
        app.clone(),
        "POST",
        &format!("/api/workspaces/{ws}/members"),
        Some(&owner_token),
        serde_json::json!({ "email": "member@example.com" }),
    )
    .await;
    assert_eq!(added.status(), StatusCode::OK);

    // Neither a fellow member nor an unrelated account may read or change the
    // list.
    for token in [&member_token, &other_token] {
        let get_res = send_auth(
            app.clone(),
            "GET",
            &format!("/api/workspaces/{ws}/members"),
            Some(token),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(get_res.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            body_json(get_res).await["error"],
            "only the workspace owner can manage members"
        );
        let post_res = send_auth(
            app.clone(),
            "POST",
            &format!("/api/workspaces/{ws}/members"),
            Some(token),
            serde_json::json!({ "email": "other@example.com" }),
        )
        .await;
        assert_eq!(post_res.status(), StatusCode::FORBIDDEN);
        let delete_res = send_auth(
            app.clone(),
            "DELETE",
            &format!("/api/workspaces/{ws}/members/other@example.com"),
            Some(token),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(delete_res.status(), StatusCode::FORBIDDEN);
    }

    // The list is unchanged: the member is still on it.
    let list = body_json(
        send_auth(
            app,
            "GET",
            &format!("/api/workspaces/{ws}/members"),
            Some(&owner_token),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    assert_eq!(list["members"].as_array().unwrap().len(), 1);
    assert_eq!(list["members"][0]["email"], "member@example.com");
}

#[tokio::test]
async fn workspace_member_add_validates_email_and_duplicates() {
    let (state, owner_token, _, _) = member_test_state().await;
    let app = app(state);

    let ws = create_owned_workspace(app.clone(), &owner_token, "Validation").await;
    let members_uri = format!("/api/workspaces/{ws}/members");

    // Unknown account → 404.
    let unknown = send_auth(
        app.clone(),
        "POST",
        &members_uri,
        Some(&owner_token),
        serde_json::json!({ "email": "ghost@example.com" }),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);

    // Malformed emails → 400.
    for bad in ["nope", "member @example.com", "  "] {
        let res = send_auth(
            app.clone(),
            "POST",
            &members_uri,
            Some(&owner_token),
            serde_json::json!({ "email": bad }),
        )
        .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "email: {bad:?}");
    }

    // The owner's own email → 409.
    let owner_self = send_auth(
        app.clone(),
        "POST",
        &members_uri,
        Some(&owner_token),
        serde_json::json!({ "email": "owner@example.com" }),
    )
    .await;
    assert_eq!(owner_self.status(), StatusCode::CONFLICT);

    // Trim + lowercase normalization lets a mixed-case invite through.
    let added = send_auth(
        app.clone(),
        "POST",
        &members_uri,
        Some(&owner_token),
        serde_json::json!({ "email": " Member@Example.com " }),
    )
    .await;
    assert_eq!(added.status(), StatusCode::OK);
    let list = body_json(added).await;
    assert_eq!(list["members"][0]["email"], "member@example.com");

    // Adding the same account again → 409.
    let duplicate = send_auth(
        app,
        "POST",
        &members_uri,
        Some(&owner_token),
        serde_json::json!({ "email": "member@example.com" }),
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn workspace_member_removal_revokes_visibility() {
    let (state, owner_token, member_token, _) = member_test_state().await;
    let app = app(state);

    let ws = create_owned_workspace(app.clone(), &owner_token, "Shared then not").await;
    let added = send_auth(
        app.clone(),
        "POST",
        &format!("/api/workspaces/{ws}/members"),
        Some(&owner_token),
        serde_json::json!({ "email": "member@example.com" }),
    )
    .await;
    assert_eq!(added.status(), StatusCode::OK);
    let saved = send_auth_ws(
        app.clone(),
        "PATCH",
        "/api/brand",
        Some(&owner_token),
        &ws,
        serde_json::json!({ "channel": "Shared Channel" }),
    )
    .await;
    assert_eq!(saved.status(), StatusCode::OK);

    // While a member, the workspace is listed and its brand is readable.
    let list = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/workspaces",
            Some(&member_token),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    assert!(list
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w["id"] == ws.as_str()));
    let brand = body_json(
        send_auth_ws(
            app.clone(),
            "GET",
            "/api/brand",
            Some(&member_token),
            &ws,
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    assert_eq!(brand["channel"], "Shared Channel");

    // The owner removes them; the email segment arrives URL-encoded and axum
    // decodes it before the handler sees it.
    let removed = send_auth(
        app.clone(),
        "DELETE",
        &format!("/api/workspaces/{ws}/members/member%40example.com"),
        Some(&owner_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(removed.status(), StatusCode::OK);
    assert!(body_json(removed).await["members"]
        .as_array()
        .unwrap()
        .is_empty());

    // Visibility is gone: no longer listed…
    let list = body_json(
        send_auth(
            app.clone(),
            "GET",
            "/api/workspaces",
            Some(&member_token),
            serde_json::json!({}),
        )
        .await,
    )
    .await;
    assert!(list
        .as_array()
        .unwrap()
        .iter()
        .all(|w| w["id"] != ws.as_str()));

    // …and the workspace header falls back to their own workspace instead of
    // erroring, mirroring `workspaces_are_isolated_per_account`.
    let brand = send_auth_ws(
        app,
        "GET",
        "/api/brand",
        Some(&member_token),
        &ws,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(brand.status(), StatusCode::OK);
    assert_eq!(body_json(brand).await["channel"], "Studio Channel");
}

#[tokio::test]
async fn workspace_member_endpoints_404_unknown_workspace() {
    let (state, owner_token, _, _) = member_test_state().await;
    let app = app(state);

    let get_res = send_auth(
        app.clone(),
        "GET",
        "/api/workspaces/ws-nope/members",
        Some(&owner_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(get_res.status(), StatusCode::NOT_FOUND);
    let post_res = send_auth(
        app.clone(),
        "POST",
        "/api/workspaces/ws-nope/members",
        Some(&owner_token),
        serde_json::json!({ "email": "member@example.com" }),
    )
    .await;
    assert_eq!(post_res.status(), StatusCode::NOT_FOUND);
    let delete_res = send_auth(
        app.clone(),
        "DELETE",
        "/api/workspaces/ws-nope/members/member@example.com",
        Some(&owner_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(delete_res.status(), StatusCode::NOT_FOUND);

    // A legacy ownerless workspace stays manageable by a signed-in account.
    let legacy = send_auth(
        app,
        "GET",
        &format!(
            "/api/workspaces/{}/members",
            content_planner_server::store::DEFAULT_WORKSPACE_ID
        ),
        Some(&owner_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(legacy.status(), StatusCode::OK);
    let legacy = body_json(legacy).await;
    assert_eq!(legacy["owner"]["email"], "");
    assert!(legacy["members"].as_array().unwrap().is_empty());
}
