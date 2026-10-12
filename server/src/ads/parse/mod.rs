//! Tolerant decoders for the Marketing API (it mixes `string`/`number` for
//! the same field) plus human summaries shown in the UI.
//!
//! Layout (one Graph shape per file):
//! - [`primitives`] — mixed-type readers shared by every decoder.
//! - [`account`] — ad account rows.
//! - [`campaign`] — campaign rows.
//! - [`adset`] — ad set rows + targeting summary.
//! - [`ad`] — ad rows (creative flattened).
//! - [`insight`] — last-30-days performance rows.
//! - [`permission`] — permission errors → reconnect guidance.

mod account;
mod ad;
mod adset;
mod campaign;
mod insight;
mod permission;
mod primitives;

pub use account::parse_ad_account;
pub use ad::parse_ad;
pub use adset::{parse_adset, summarize_targeting};
pub use campaign::parse_campaign;
pub use insight::parse_insight;
pub use permission::permission_hint;

pub(crate) use primitives::{rows, str_of};
