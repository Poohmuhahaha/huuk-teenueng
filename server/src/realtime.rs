//! Server-sent change notifications.
//!
//! The store is mutated by HTTP handlers (and background refresh tasks use the
//! same API surface). Every successful mutation bumps a process-wide revision.
//! Connected SPAs watch `/api/events` and invalidate their query cache when it
//! changes. This keeps clients eventually consistent without adding a socket
//! protocol to every feature.

use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use axum::{
    extract::Request,
    http::Method,
    middleware::Next,
    response::{
        sse::{Event, KeepAlive, Sse},
        Response,
    },
};
use futures_util::stream::{self, Stream};
use tokio::time::{interval, MissedTickBehavior};

/// Monotonic revision for the current server process. It is process-local on
/// purpose: after a restart clients receive a fresh value and refresh once.
static REVISION: AtomicU64 = AtomicU64::new(1);

pub fn bump() {
    REVISION.fetch_add(1, Ordering::Relaxed);
}

pub fn current() -> u64 {
    REVISION.load(Ordering::Relaxed)
}

/// Long-lived SSE stream. One event per second is intentionally cheap; the
/// browser compares the revision and only refetches when it actually changes.
pub async fn events() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut ticker = interval(Duration::from_secs(1));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let stream = stream::unfold(ticker, |mut ticker| async move {
        ticker.tick().await;
        let event = Event::default()
            .event("revision")
            .data(current().to_string());
        Some((Ok(event), ticker))
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/// Bump the revision after any successful mutating HTTP response. Keeping this
/// in middleware means new endpoints are covered automatically.
pub async fn bump_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let response = next.run(req).await;
    let is_read = matches!(method, Method::GET | Method::HEAD | Method::OPTIONS);
    if !is_read && response.status().is_success() {
        bump();
    }
    response
}
