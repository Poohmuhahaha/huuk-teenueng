use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use content_planner_server::db;
use content_planner_server::store::{demo_env, write_atomic, Store};
use content_planner_server::{app, AppState};
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let port: u16 = match std::env::var("PORT") {
        Ok(raw) => raw.trim().parse().unwrap_or_else(|_| {
            eprintln!("PORT must be a number, got '{raw}'");
            std::process::exit(2);
        }),
        Err(_) => 8787,
    };
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".into());
    let addr = format!("{bind}:{port}");

    let run_migrations_only = std::env::args().any(|arg| arg == "--run-migrations-only");
    let auto_migrate = std::env::var("AUTO_MIGRATE")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);

    // Optional PostgreSQL database connection (SOP §3.3 Hard Rule 3: migrations run only in one-off mode).
    let db_pool = match std::env::var("DATABASE_URL") {
        Ok(raw) if !raw.trim().is_empty() => {
            let pool = match db::init_pool(raw.trim()).await {
                Ok(pool) => pool,
                Err(error) => {
                    eprintln!("cannot connect to DATABASE_URL: {error}");
                    std::process::exit(5);
                }
            };
            if run_migrations_only {
                if let Err(error) = db::run_migrations(&pool).await {
                    eprintln!("database migration failed: {error}");
                    std::process::exit(6);
                }
                println!("Migrations executed successfully. Exiting one-off container.");
                return;
            }
            if auto_migrate {
                if let Err(error) = db::run_migrations(&pool).await {
                    eprintln!("database migration failed: {error}");
                    std::process::exit(6);
                }
            }
            Some(pool)
        }
        _ => {
            if run_migrations_only {
                eprintln!("DATABASE_URL must be set for --run-migrations-only");
                std::process::exit(5);
            }
            None
        }
    };

    // Persistent snapshot path. `DATA_FILE=` (empty) disables file persistence.
    let data_path: Option<PathBuf> = match std::env::var("DATA_FILE") {
        Ok(raw) if raw.trim().is_empty() => None,
        Ok(raw) => Some(PathBuf::from(raw.trim())),
        Err(_) => Some(PathBuf::from("data/content-planner.json")),
    };

    let demo = demo_env();
    let db_loaded = match &db_pool {
        Some(pool) => match db::load_store_from_db(pool).await {
            Ok(store) => store,
            Err(error) => {
                eprintln!("cannot load store from database: {error}");
                std::process::exit(4);
            }
        },
        None => None,
    };

    let mut store = match (db_loaded, &data_path) {
        (Some(store), _) => store,
        (None, Some(path)) if path.exists() => match Store::load_or_seed(path) {
            Ok(store) => store,
            Err(error) => {
                eprintln!("cannot load {}: {error}", path.display());
                eprintln!("fix or move the file, or start with a different DATA_FILE");
                std::process::exit(4);
            }
        },
        // A fresh production store starts empty; DEMO_MODE=1 seeds the
        // wireframe dataset instead (local development only).
        (None, _) if demo => Store::seed(),
        (None, _) => Store::fresh(),
    };
    store.db = db_pool.clone();
    store.apply_env();

    if !store.demo_mode && store.accounts.is_empty() {
        eprintln!(
            "warning: no accounts exist and demo mode is off — set ADMIN_EMAIL/ADMIN_PASSWORD \
             (or ALLOW_REGISTRATION=1, DEMO_MODE=1) before starting"
        );
    }

    // Uploaded brand fonts live beside the snapshot (same volume).
    store.fonts_dir = data_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|dir| dir.join("fonts"));
    store.images_dir = data_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|dir| dir.join("images"));

    let state: AppState = Arc::new(RwLock::new(store));
    let persist_task = if data_path.is_some() || db_pool.is_some() {
        Some(tokio::spawn(persist_loop(
            state.clone(),
            data_path.clone(),
            db_pool.clone(),
        )))
    } else {
        None
    };
    // Pulls the connected platforms' content on a schedule (LIVE_REFRESH_MIN).
    let live_task = content_planner_server::live::spawn_refresh_loop(state.clone());
    // Pulls the Meta Ads mirror on its own, slower schedule (ADS_REFRESH_MIN).
    let ads_task = content_planner_server::ads::spawn_ads_loop(state.clone());

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("cannot bind {addr}: {error}");
            std::process::exit(3);
        }
    };

    let providers: Vec<&str> = {
        let store = state.read().await;
        content_planner_server::oauth::PROVIDERS
            .iter()
            .filter(|p| store.oauth.configured(p))
            .map(|p| p.name)
            .collect()
    };

    tracing::info!(%addr, "content-planner-server listening");
    println!("Server on → http://localhost:{port}");
    if db_pool.is_some() {
        println!("Database : PostgreSQL connected (migrations applied)");
    }
    if let Some(path) = &data_path {
        println!("Data     : {}", path.display());
    } else {
        println!("Data     : in-memory only (DATA_FILE= empty)");
    }
    println!(
        "Mode     : {}{}",
        if state.read().await.demo_mode {
            "demo"
        } else {
            "production"
        },
        if state.read().await.setup.auth_required {
            " · auth required"
        } else {
            " · auth off"
        }
    );
    if providers.is_empty() {
        println!("OAuth    : mock mode (set META_APP_ID/META_APP_SECRET for real Meta login)");
    } else {
        println!("OAuth    : real mode for {}", providers.join(", "));
    }
    match content_planner_server::static_dir() {
        Some(dir) if dir.is_dir() => println!("Web      : serving {}", dir.display()),
        Some(dir) => eprintln!(
            "warning: STATIC_DIR={} does not exist — the SPA will not be served",
            dir.display()
        ),
        None => println!("Web      : API only (set STATIC_DIR to serve the frontend)"),
    }

    if let Err(error) = axum::serve(listener, app(state.clone()))
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!(%error, "server error");
        std::process::exit(1);
    }

    live_task.abort();
    ads_task.abort();
    if let Some(task) = persist_task {
        task.abort();
    }
    if data_path.is_some() || db_pool.is_some() {
        let bytes = match state.read().await.snapshot_bytes() {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::error!(%error, "final snapshot serialization failed");
                return;
            }
        };
        if let Some(pool) = db_pool {
            if let Err(error) = db::save_snapshot_bytes_to_db(&pool, &bytes).await {
                tracing::error!(%error, "final database snapshot failed");
            }
        }
        if let Some(path) = data_path {
            let result = tokio::task::spawn_blocking(move || write_atomic(&path, &bytes)).await;
            match result {
                Ok(Ok(())) => tracing::info!("final snapshot saved"),
                Ok(Err(error)) => tracing::error!(%error, "final snapshot failed"),
                Err(error) => tracing::error!(%error, "final snapshot task failed"),
            }
        }
    }
    println!("Server stopped");
}

/// Periodically persists the store when its content changed, and prunes
/// expired sessions / pending OAuth states / login counters.
async fn persist_loop(state: AppState, path: Option<PathBuf>, db_pool: Option<sqlx::PgPool>) {
    let mut ticker = tokio::time::interval(Duration::from_secs(5));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut last: Option<u64> = None;
    loop {
        ticker.tick().await;
        let bytes = {
            let mut store = state.write().await;
            store.prune_ephemeral();
            match store.snapshot_bytes() {
                Ok(bytes) => bytes,
                Err(error) => {
                    tracing::error!(%error, "cannot serialize store");
                    continue;
                }
            }
        };
        let hash = hash_bytes(&bytes);
        if last == Some(hash) {
            continue;
        }
        let mut ok = true;
        if let Some(pool) = &db_pool {
            if let Err(error) = db::save_snapshot_bytes_to_db(pool, &bytes).await {
                tracing::error!(%error, "database snapshot write failed");
                ok = false;
            }
        }
        if let Some(target_path) = &path {
            let target = target_path.clone();
            let snapshot_bytes = bytes.clone();
            match tokio::task::spawn_blocking(move || write_atomic(&target, &snapshot_bytes)).await
            {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    tracing::error!(%error, path = %target_path.display(), "snapshot write failed");
                    ok = false;
                }
                Err(error) => {
                    tracing::error!(%error, "snapshot task failed");
                    ok = false;
                }
            }
        }
        if ok {
            last = Some(hash);
        }
    }
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => {
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received");
}
