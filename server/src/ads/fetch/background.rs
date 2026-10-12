//! Background refresh: every `ADS_REFRESH_MIN` minutes (default 60, `0`
//! disables), for workspaces with a usable Meta token. Staggered 45 s behind
//! the live loop so both never hit Graph together.

use std::time::Duration;

use crate::ads::meta_user_token;
use crate::AppState;

use super::refresh::refresh_workspace_ads;

pub fn spawn_ads_loop(state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let minutes = std::env::var("ADS_REFRESH_MIN")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(60);
        if minutes == 0 {
            tracing::info!("ads refresh loop disabled (ADS_REFRESH_MIN=0)");
            return;
        }
        // Stagger behind the live loop so both do not hit Graph together.
        tokio::time::sleep(Duration::from_secs(45)).await;
        let mut ticker = tokio::time::interval(Duration::from_secs(minutes * 60));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            let workspace_ids: Vec<String> = {
                let mut store = state.write().await;
                store
                    .workspaces
                    .iter()
                    .filter(|w| {
                        w.connections
                            .iter()
                            .any(|c| c.id == "meta" && c.status == "connected")
                    })
                    .map(|w| w.id.clone())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .filter(|id| meta_user_token(&mut store, id).is_some())
                    .collect()
            };
            for id in workspace_ids {
                let _ = refresh_workspace_ads(&state, &id).await;
            }
        }
    })
}
