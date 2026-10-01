#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Huuk desktop: starts the content-planner REST server as a sidecar before
// the window loads, so the app works with no browser and no network.
// The sidecar inherits our environment: localhost-only, on-disk snapshot,
// single-user demo auth. Tauri terminates sidecar children on app exit.
use std::path::PathBuf;
use tauri::Manager;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

const SIDECAR_PORT: u16 = 8787;

fn data_file(app: &tauri::AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().expect("desktop app data dir");
    std::fs::create_dir_all(&dir).ok();
    dir.join("content-planner.json")
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let snapshot = data_file(app.handle());
            std::env::set_var("PORT", SIDECAR_PORT.to_string());
            std::env::set_var("BIND_ADDR", "127.0.0.1");
            std::env::set_var("DEMO_MODE", "1");
            std::env::set_var("DATA_FILE", &snapshot);
            eprintln!("[huuk] sidecar snapshot: {}", snapshot.display());

            let (mut rx, child) = app
                .handle()
                .shell()
                .sidecar("content-planner-server")
                .expect("sidecar binary missing — run npm run desktop:sidecar first")
                .spawn()
                .expect("sidecar failed to start");
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stdout(bytes) => {
                            eprint!("[server] {}", String::from_utf8_lossy(&bytes))
                        }
                        CommandEvent::Stderr(bytes) => {
                            eprint!("[server:err] {}", String::from_utf8_lossy(&bytes))
                        }
                        CommandEvent::Error(err) => {
                            eprintln!("[server] process error: {err}")
                        }
                        CommandEvent::Terminated(status) => {
                            eprintln!("[server] exited: {status:?}")
                        }
                        _ => {}
                    }
                }
            });
            // Keep the handle alive without ever using it: Tauri kills
            // sidecar children when the app exits.
            std::mem::forget(child);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Huuk desktop");
}
