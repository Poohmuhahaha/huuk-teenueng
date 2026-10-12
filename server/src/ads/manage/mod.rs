//! The opt-in writes: pause/resume budgets, deep-copy duplicates, and the
//! "boost a planned post" flow — every call is audited (newest first).
//!
//! Layout:
//! - [`context`] — opt-in/token guard + audit trail shared by all writes.
//! - [`campaign`] — status / budget / duplicate.
//! - [`boost`] — the 3-step paused boost flow.

mod boost;
mod campaign;
mod context;

pub use boost::create_boost;
pub use campaign::{duplicate_campaign, set_campaign_budget, set_campaign_status};
