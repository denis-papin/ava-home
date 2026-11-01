use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

tokio::task_local! {
    /// Tracking number of the HTTP call being processed by the current task.
    pub static TRACKING_ID: u64;
}

static NEXT_TRACKING_ID: OnceLock<AtomicU64> = OnceLock::new();

/// Give a new tracking number for every call.
/// The counter starts from a time-based seed so numbers from two runs of the service rarely overlap.
fn next_tracking_id() -> u64 {
    NEXT_TRACKING_ID
        .get_or_init(|| {
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            AtomicU64::new(10_000 + seed % 90_000)
        })
        .fetch_add(1, Ordering::Relaxed)
}

/// Tracking number of the current call, if the code runs inside one.
pub fn current_tracking_id() -> Option<u64> {
    TRACKING_ID.try_with(|id| *id).ok()
}

/// Axum middleware: run the whole request under its own tracking number.
pub async fn track_request(request: Request, next: Next) -> Response {
    TRACKING_ID.scope(next_tracking_id(), next.run(request)).await
}

/// Log format: [2026-10-31T07:15:02Z] [INFO] [radiator_api::api] [67890] message
/// Lines written outside an HTTP call (startup...) show [-] as tracking number.
pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let tracking_id = current_tracking_id()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string());
            writeln!(
                buf,
                "[{}] [{}] [{}] [{}] {}",
                buf.timestamp_seconds(),
                record.level(),
                record.target(),
                tracking_id,
                record.args()
            )
        })
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::routing::get;
    use axum::{middleware, Router};
    use tower::ServiceExt;

    async fn call(app: Router) -> String {
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn every_call_gets_its_own_tracking_id() {
        let app = Router::new()
            .route("/", get(|| async { format!("{:?}", current_tracking_id()) }))
            .layer(middleware::from_fn(track_request));

        let first = call(app.clone()).await;
        let second = call(app).await;

        assert!(first.starts_with("Some("));
        assert!(second.starts_with("Some("));
        assert_ne!(first, second);
        assert_eq!(current_tracking_id(), None);
    }
}
