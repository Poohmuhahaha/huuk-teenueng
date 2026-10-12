//! Workspace refresh entry point. The store lock is never held across network
//! I/O: the token is read under one short lock, the fetch runs unlocked, and
//! the result is written back under a second lock. A failed refresh records
//! `ads.error` and leaves the previous mirror in place — including its audit
//! trail, which a refresh must never lose.

use crate::ads::meta_user_token;
use crate::model::AdsData;
use crate::AppState;

use super::mirror::fetch_meta_ads;

pub async fn refresh_workspace_ads(
    state: &AppState,
    workspace_id: &str,
) -> Result<AdsData, String> {
    let (oauth, token) = {
        let mut store = state.write().await;
        if store.workspace(workspace_id).is_none() {
            return Err("workspace not found".into());
        }
        let token = meta_user_token(&mut store, workspace_id);
        (store.oauth.clone(), token)
    };
    let Some(token) = token else {
        return Err("reconnect Meta to grant access — the stored token is missing".into());
    };

    match fetch_meta_ads(&oauth, &token).await {
        Ok(mut ads) => {
            let mut store = state.write().await;
            if let Some(workspace) = store.workspace_mut(workspace_id) {
                // Preserve what the mirror must never lose on a refresh.
                ads.audit = std::mem::take(&mut workspace.ads.audit);
                workspace.ads = ads.clone();
            }
            tracing::info!(
                workspace = %workspace_id,
                accounts = ads.accounts.len(),
                campaigns = ads.campaigns.len(),
                "ads mirror refreshed"
            );
            Ok(ads)
        }
        Err(error) => {
            tracing::warn!(workspace = %workspace_id, %error, "ads refresh failed");
            let mut store = state.write().await;
            if let Some(workspace) = store.workspace_mut(workspace_id) {
                workspace.ads.error = error.clone();
            }
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ads::fetch::test_support::fake_user_token_graph;

    #[tokio::test]
    async fn uses_the_user_token_not_the_page_token() {
        let base = fake_user_token_graph().await;
        let state = crate::new_state();
        {
            let mut store = state.write().await;
            store.oauth.set_graph_base(&base);
            store.enable_token_vault("unit-test-admin-secret-0123456789");
            store.set_platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta", "page-token");
            store.set_platform_token(
                crate::store::DEFAULT_WORKSPACE_ID,
                "meta#user",
                "user-token",
            );
        }
        let ads = refresh_workspace_ads(&state, crate::store::DEFAULT_WORKSPACE_ID)
            .await
            .unwrap();
        assert_eq!(ads.accounts.len(), 1);
        assert_eq!(ads.accounts[0].id, "act_7");
    }

    #[tokio::test]
    async fn falls_back_to_a_single_legacy_token() {
        // Connections made before the Page/user split stored only the user
        // token under the plain key.
        let base = fake_user_token_graph().await;
        let state = crate::new_state();
        {
            let mut store = state.write().await;
            store.oauth.set_graph_base(&base);
            store.enable_token_vault("unit-test-admin-secret-0123456789");
            store.set_platform_token(crate::store::DEFAULT_WORKSPACE_ID, "meta", "user-token");
        }
        let ads = refresh_workspace_ads(&state, crate::store::DEFAULT_WORKSPACE_ID)
            .await
            .unwrap();
        assert_eq!(ads.accounts.len(), 1);
    }
}
