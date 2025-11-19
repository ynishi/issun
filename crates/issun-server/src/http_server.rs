//! HTTP server for metrics endpoint

use crate::metrics::SharedMetrics;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use std::net::SocketAddr;
use tracing::{error, info};

/// HTTP server state
#[derive(Clone)]
struct AppState {
    metrics: SharedMetrics,
}

/// Start HTTP server for metrics endpoint
pub async fn start_http_server(
    bind_addr: SocketAddr,
    metrics: SharedMetrics,
) -> Result<(), anyhow::Error> {
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .with_state(AppState { metrics });

    info!("Starting HTTP metrics server on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handler for /metrics endpoint
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    match state.metrics.gather() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(e) => {
            error!("Failed to gather metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to gather metrics: {}", e),
            )
        }
    }
}

/// Handler for /health endpoint
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_health_endpoint() {
        let response = health_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let metrics = Arc::new(crate::metrics::Metrics::new().unwrap());
        let state = AppState { metrics };

        let response = metrics_handler(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
