//! The read mirror: ad accounts → structure → last-30-days insights,
//! plus the workspace refresh entry point and the background loop.
//!
//! Layout (one stage per file):
//! - [`accounts`] — visible ad accounts (+ Business-portfolio fallback).
//! - [`structure`] — one account's campaigns/ad sets/ads/insights (5 calls).
//! - [`mirror`] — assembles the whole mirror for one login.
//! - [`refresh`] — workspace entry point (lock → fetch → write-back).
//! - [`background`] — the `ADS_REFRESH_MIN` loop.

mod accounts;
mod background;
mod mirror;
mod refresh;
mod structure;

#[cfg(test)]
mod test_support;

pub use accounts::fetch_ad_accounts;
pub use background::spawn_ads_loop;
pub use mirror::fetch_meta_ads;
pub use refresh::refresh_workspace_ads;

/// Structure kept per workspace (bounds the snapshot).
pub(super) const KEEP_ACCOUNTS: usize = 5;
pub(super) const KEEP_CAMPAIGNS: usize = 200;
pub(super) const KEEP_ADSETS: usize = 500;
pub(super) const KEEP_ADS: usize = 1000;
