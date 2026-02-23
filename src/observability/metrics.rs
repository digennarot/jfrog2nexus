use axum::{extract::State, routing::get, Router};
use metrics_exporter_prometheus::PrometheusHandle;
use std::net::SocketAddr;
use tracing::info;

pub fn setup_metrics_recorder() -> PrometheusHandle {
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder")
}

async fn metrics_handler(State(handle): State<PrometheusHandle>) -> String {
    handle.render()
}

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
