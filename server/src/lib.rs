//! Huuk by teenueng REST backend.
//!
//! Mirrors the frontend's mock API surface (`wireframe/src/mock/api.ts`) with an
//! in-memory store plus optional JSON snapshots. Swap `store::Store` for a
//! database repository later; the handlers and the HTTP contract stay the same.
pub mod ads;
pub mod error;
pub mod handlers;
pub mod live;
pub mod model;
pub mod oauth;
pub mod secrets;
pub mod store;
pub mod workspaces;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::{header::CONTENT_TYPE, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Json, Router,
};

use error::ApiError;
use serde_json::json;
use tokio::sync::RwLock;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

pub type AppState = Arc<RwLock<store::Store>>;

/// Largest accepted request body (JSON payloads are small).
pub const MAX_BODY_BYTES: usize = 256 * 1024;

/// Request deadline — keeps a stuck handler from pinning a connection forever.
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Content Security Policy for JSON API responses (nothing may load).
const API_CSP: &str = "default-src 'none'; frame-ancestors 'none'";

/// Content Security Policy for the served SPA (its own bundle, images over
/// https, API calls to itself or a configured origin).
const APP_CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
img-src 'self' data: blob: https:; font-src 'self' data:; connect-src 'self' http: https:; \
frame-ancestors 'none'; base-uri 'self'; form-action 'self'";

pub fn new_state() -> AppState {
    Arc::new(RwLock::new(store::Store::seed()))
}

/// Directory of built frontend assets (`STATIC_DIR`), when the binary should
/// also serve the SPA (the no-Docker deployment).
pub fn static_dir() -> Option<PathBuf> {
    std::env::var("STATIC_DIR")
        .ok()
        .map(|value| PathBuf::from(value.trim()))
        .filter(|path| !path.as_os_str().is_empty())
}

pub fn app(state: AppState) -> Router {
    app_with_static(state, static_dir())
}

/// Builds the router. API routes always win; everything else either serves the
/// SPA from `static_dir` or returns the JSON 404 envelope.
pub fn app_with_static(state: AppState, static_dir: Option<PathBuf>) -> Router {
    let fallback_dir = static_dir.clone();
    let mut router = Router::new()
        .route("/api/health", get(handlers::health))
        .route("/api/ready", get(handlers::ready))
        .route(
            "/api/setup",
            get(handlers::get_setup).patch(handlers::patch_setup),
        )
        .route("/api/setup/options", post(handlers::add_option))
        .route("/api/setup/users", post(handlers::add_user))
        .route("/api/setup/users/{name}", delete(handlers::remove_user))
        .route("/api/setup/roles", post(handlers::add_role))
        .route("/api/setup/roles/{name}", delete(handlers::remove_role))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/me", get(handlers::me))
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/auth/logout-all", post(handlers::logout_all))
        .route("/api/auth/change-password", post(handlers::change_password))
        .route("/api/auth/plan", post(handlers::set_plan))
        .route(
            "/api/auth/profile",
            axum::routing::patch(handlers::update_profile),
        )
        .route(
            "/api/auth/accounts",
            get(handlers::list_accounts).post(handlers::create_account),
        )
        .route(
            "/api/auth/accounts/{email}",
            delete(handlers::delete_account),
        )
        .route(
            "/api/content",
            get(handlers::list_content).post(handlers::create_content),
        )
        .route(
            "/api/content/{id}",
            get(handlers::get_content)
                .patch(handlers::update_content)
                .delete(handlers::delete_content),
        )
        .route("/api/content/{id}/publish", post(handlers::publish_content))
        .route(
            "/api/content/{id}/unpublish",
            post(handlers::unpublish_content),
        )
        .route(
            "/api/content/{id}/schedule",
            post(handlers::schedule_content),
        )
        .route(
            "/api/content/{id}/duplicate",
            post(handlers::duplicate_content),
        )
        .route(
            "/api/content/{id}/revisions",
            get(handlers::list_content_revisions),
        )
        .route(
            "/api/content/{id}/revisions/{revision}",
            get(handlers::get_content_revision),
        )
        .route(
            "/api/content/{id}/revisions/{revision}/restore",
            post(handlers::restore_content_revision),
        )
        .route("/api/public/content", get(handlers::list_public_content))
        .route(
            "/api/public/content/{slug}",
            get(handlers::get_public_content),
        )
        .route(
            "/api/posts",
            get(handlers::list_posts).post(handlers::add_post),
        )
        .route(
            "/api/posts/{id}",
            patch(handlers::update_post).delete(handlers::delete_post),
        )
        .route("/api/posts/{id}/lock", post(handlers::lock_post))
        .route("/api/posts/{id}/unlock", post(handlers::unlock_post))
        .route(
            "/api/ideas",
            get(handlers::list_ideas).post(handlers::add_idea),
        )
        .route("/api/ideas/{id}/toggle", post(handlers::toggle_idea))
        .route("/api/ideas/{id}/promote", post(handlers::promote_idea))
        .route("/api/tags", get(handlers::list_tags))
        .route("/api/tags/{id}", post(handlers::add_tag))
        .route("/api/metrics", get(handlers::list_metrics))
        .route("/api/metrics/import", post(handlers::import_metrics))
        .route("/api/live", get(handlers::get_live))
        .route("/api/ads", get(handlers::get_ads))
        .route("/api/ads/sync", post(handlers::sync_ads))
        .route("/api/ads/manage", post(handlers::set_ads_manage))
        .route(
            "/api/ads/campaigns/{id}/status",
            post(handlers::set_ad_campaign_status),
        )
        .route(
            "/api/ads/campaigns/{id}/budget",
            post(handlers::set_ad_campaign_budget),
        )
        .route(
            "/api/ads/campaigns/{id}/duplicate",
            post(handlers::duplicate_ad_campaign),
        )
        .route("/api/ads/boost", post(handlers::create_ad_boost))
        .route(
            "/api/txns",
            get(handlers::list_txns).post(handlers::add_txn),
        )
        .route(
            "/api/brand",
            get(handlers::get_brand).patch(handlers::save_brand),
        )
        .route(
            "/api/campaigns",
            get(handlers::list_campaigns).post(handlers::create_campaign),
        )
        .route(
            "/api/campaigns/{id}",
            get(handlers::get_campaign)
                .patch(handlers::update_campaign)
                .delete(handlers::delete_campaign),
        )
        .route(
            "/api/brand/fonts",
            get(handlers::list_fonts)
                .post(handlers::upload_font)
                .layer(DefaultBodyLimit::max(handlers::FONT_MAX_BYTES * 2)),
        )
        .route("/api/brand/fonts/{name}", delete(handlers::delete_font))
        .route(
            "/api/brand/images",
            post(handlers::upload_brand_image)
                .layer(DefaultBodyLimit::max(handlers::BRAND_IMAGE_MAX_BYTES * 2)),
        )
        .route(
            "/api/brand/images/{name}",
            get(handlers::brand_image_file).delete(handlers::delete_brand_image),
        )
        .route("/api/brand/fonts/{name}/file", get(handlers::font_file))
        .route(
            "/api/workspaces",
            get(workspaces::list).post(workspaces::create),
        )
        .route(
            "/api/workspaces/{id}",
            patch(workspaces::rename).delete(workspaces::remove),
        )
        .route(
            "/api/workspaces/{id}/members",
            get(workspaces::list_members).post(workspaces::add_member),
        )
        .route(
            "/api/workspaces/{id}/members/{email}",
            delete(workspaces::remove_member),
        )
        .route("/api/platforms", get(handlers::list_platforms))
        .route(
            "/api/platforms/{id}/connect",
            post(handlers::connect_platform),
        )
        .route(
            "/api/platforms/{id}/disconnect",
            post(handlers::disconnect_platform),
        )
        .route("/api/platforms/{id}/sync", post(handlers::sync_platform))
        .route("/api/oauth/{platform}", get(oauth::status))
        .route("/api/oauth/{platform}/start", get(oauth::start))
        .route("/api/oauth/{platform}/callback", get(oauth::callback))
        .route("/api/oauth/{platform}/pending", get(oauth::pending_pick))
        .route("/api/oauth/{platform}/choose", post(oauth::choose_page));
    if docs_enabled() {
        router = router
            .route("/api/docs", get(handlers::api_docs))
            .route("/api/docs/swagger-ui.css", get(handlers::api_docs_css))
            .route("/api/docs/swagger-ui-bundle.js", get(handlers::api_docs_js))
            .route("/api/docs/swagger-init.js", get(handlers::api_docs_init))
            .route("/api/openapi.yaml", get(handlers::openapi_spec));
    }
    if static_dir.is_none() {
        // API-only mode: `/` answers with a friendly hello. With a static dir
        // the root falls through to the SPA shell.
        router = router.route("/", get(handlers::root));
    }
    let read_state = state.clone();
    router
        .fallback(move |req: Request| {
            let dir = fallback_dir.clone();
            async move { handlers::serve_fallback(dir, req).await }
        })
        .layer(middleware::from_fn_with_state(
            read_state,
            require_read_auth,
        ))
        .layer(middleware::from_fn(security_headers))
        .layer(middleware::from_fn(timeout_middleware))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Swagger UI / OpenAPI spec are on in debug builds and when `SWAGGER_UI=1`
/// (or true/yes/on); release production builds keep them off unless enabled.
fn docs_enabled() -> bool {
    if cfg!(debug_assertions) {
        return true;
    }
    std::env::var("SWAGGER_UI")
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// Origins allowed to call the API. `CORS_ORIGINS` (comma-separated) wins;
/// otherwise demo mode stays permissive and production only trusts the
/// configured frontend origin (plus the local Vite dev server).
fn cors_layer() -> CorsLayer {
    let configured = std::env::var("CORS_ORIGINS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|o| !o.is_empty())
                .filter_map(|o| o.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>()
        })
        .filter(|origins| !origins.is_empty());

    let base = CorsLayer::new()
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    if let Some(origins) = configured {
        return base.allow_origin(AllowOrigin::list(origins));
    }
    if std::env::var("DEMO_MODE")
        .map(|v| matches!(v.trim(), "1" | "true"))
        .unwrap_or(false)
    {
        return base.allow_origin(tower_http::cors::Any);
    }

    let mut origins = vec![
        HeaderValue::from_static("http://localhost:5173"),
        HeaderValue::from_static("http://127.0.0.1:5173"),
    ];
    if let Ok(front) = std::env::var("FRONTEND_URL") {
        if let Ok(value) = front.parse::<HeaderValue>() {
            origins.push(value);
        }
    }
    base.allow_origin(AllowOrigin::list(origins))
}

/// `/api` GETs that stay reachable without a session while auth is required:
/// liveness, the redacted setup payload, the public content delivery API,
/// OAuth handshakes/callbacks, the optional API docs, and the brand CI (the
/// whole site is themed from it, including the login and public reader pages).
/// Uploaded font files are fetched by CSS `@font-face`, which cannot carry an
/// `Authorization` header, so they stay open as assets.
fn is_public_read(path: &str) -> bool {
    path == "/api/health"
        || path == "/api/ready"
        || path == "/api/setup"
        || path.starts_with("/api/public/")
        || path.starts_with("/api/oauth/")
        || path == "/api/brand"
        || path == "/api/brand/fonts"
        || (path.starts_with("/api/brand/fonts/") && path.ends_with("/file"))
        || path.starts_with("/api/brand/images/")
        || path == "/api/openapi.yaml"
        || path.starts_with("/api/docs")
}

/// Blocks anonymous reads of everything else under `/api` when `authRequired`
/// is on (authoring data, finance, the account directory, …). Mutations are
/// already gated per-handler by `require_perm`.
async fn require_read_auth(State(s): State<AppState>, req: Request, next: Next) -> Response {
    let path = req.uri().path();
    let is_read = req.method() == Method::GET || req.method() == Method::HEAD;
    if !is_read || !path.starts_with("/api/") || is_public_read(path) {
        return next.run(req).await;
    }
    let allowed = {
        let store = s.read().await;
        handlers::has_read_access(&store, req.headers())
    };
    if allowed {
        return next.run(req).await;
    }
    ApiError::unauthorized("authentication required").into_response()
}

/// Conservative security headers. API responses are never cacheable and ship a
/// locked-down CSP; served SPA files get their own CSP and honor the cache
/// headers set by the static handler.
async fn security_headers(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let mut response = next.run(req).await;
    let is_api = path == "/api" || path.starts_with("/api/");
    let is_html = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    if is_api || is_html {
        headers.insert(
            "content-security-policy",
            if is_html {
                HeaderValue::from_static(APP_CSP)
            } else {
                HeaderValue::from_static(API_CSP)
            },
        );
    }
    if is_api && !headers.contains_key("cache-control") {
        // Handlers may opt into caching (e.g. uploaded brand fonts).
        headers.insert("cache-control", HeaderValue::from_static("no-store"));
    } else if !is_api && !headers.contains_key("cache-control") {
        headers.insert("cache-control", HeaderValue::from_static("no-cache"));
    }
    response
}

/// Returns a JSON 504 when a handler exceeds [`REQUEST_TIMEOUT_SECS`].
async fn timeout_middleware(req: Request, next: Next) -> Response {
    match tokio::time::timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS), next.run(req)).await {
        Ok(response) => response,
        Err(_) => {
            tracing::warn!("request timed out");
            (
                StatusCode::GATEWAY_TIMEOUT,
                Json(json!({ "error": "request timed out" })),
            )
                .into_response()
        }
    }
}
