//! Meta Ads mirror: read the ad accounts, campaigns, ad sets, ads and their
//! settings + last-30-days performance into the workspace, and (only after an
//! explicit opt-in) change campaigns through the same login.
//!
//! Rules (same as `crate::live`):
//! - The store lock is never held across network I/O.
//! - A failed refresh never wipes the previous mirror: it records
//!   [`AdsData::error`] and leaves the data in place.
//! - Everything is best-effort: a metric the API does not report stays `0`.
//! - Reads are batched per ad account (5 calls) because the Marketing API's
//!   development tier allows only 60 points per ad account per 300 s window.
//! - Writes are 3 points each and always appended to [`AdsData::audit`].
//!
//! Layout (one Graph shape / one stage per file, each < 170 lines):
//! - [`graph`] — `GET`/`POST {GRAPH}{path}` helpers with Graph error mapping.
//! - [`parse`] — tolerant decoders (`parse/primitives.rs` readers,
//!   `parse/account.rs`, `parse/campaign.rs`, `parse/adset.rs`, `parse/ad.rs`,
//!   `parse/insight.rs`, `parse/permission.rs`).
//! - [`fetch`] — the read mirror (`fetch/accounts.rs`, `fetch/structure.rs`,
//!   `fetch/mirror.rs`, `fetch/refresh.rs`, `fetch/background.rs`).
//! - [`manage`] — the opt-in writes (`manage/context.rs` guard,
//!   `manage/campaign.rs` status/budget/duplicate, `manage/boost.rs` flow).

mod fetch;
mod graph;
mod manage;
mod parse;

pub use fetch::{fetch_ad_accounts, fetch_meta_ads, refresh_workspace_ads, spawn_ads_loop};
pub use manage::{create_boost, duplicate_campaign, set_campaign_budget, set_campaign_status};
pub use parse::{
    parse_ad, parse_ad_account, parse_adset, parse_campaign, parse_insight, permission_hint,
    summarize_targeting,
};

/// Audit entries kept (newest first).
pub const AUDIT_LIMIT: usize = 50;

/// Meta Ads endpoints need the *user* token: the Page token that mirrors Page
/// content cannot list ad accounts. Older connections stored only one token
/// (the user token) under the plain key, so fall back to it.
pub(crate) fn meta_user_token(
    store: &mut crate::store::Store,
    workspace_id: &str,
) -> Option<String> {
    store
        .platform_token(workspace_id, "meta#user")
        .or_else(|| store.platform_token(workspace_id, "meta"))
}
