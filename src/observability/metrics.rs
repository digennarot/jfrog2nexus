use axum::{extract::State, routing::get, Router};
use metrics_exporter_prometheus::PrometheusHandle;
use std::net::SocketAddr;
use tracing::info;

/// Install the global Prometheus metrics recorder and return a handle for rendering.
///
/// Must be called once before any `metrics::counter!` / `metrics::gauge!` macros are used.
///
/// # Returns
///
/// A [`PrometheusHandle`] that can be passed to [`start_metrics_server`] to expose
/// the `/metrics` endpoint.
///
/// # Panics
///
/// Panics if the Prometheus recorder has already been installed (i.e. called twice).
pub fn setup_metrics_recorder() -> PrometheusHandle {
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder")
}

async fn metrics_handler(State(handle): State<PrometheusHandle>) -> String {
    handle.render()
}

/// Start an Axum HTTP server that serves Prometheus metrics on `GET /metrics`.
///
/// This function runs indefinitely and is intended to be spawned with `tokio::spawn`.
///
/// # Arguments
///
/// * `handle` - The Prometheus recorder handle returned by [`setup_metrics_recorder`].
/// * `addr` - The socket address to bind the metrics server to (e.g. `127.0.0.1:9090`).
///
/// # Panics
///
/// Panics if the TCP listener cannot bind to `addr` or if the server encounters a fatal error.
pub async fn start_metrics_server(handle: PrometheusHandle, addr: SocketAddr) {
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(handle);

    info!("Starting metrics server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind metrics server port");

    axum::serve(listener, app)
        .await
        .expect("failed to serve metrics");
}
