use axum::{routing::get, Extension, Router};
use clap::Parser;
use data_exporter::DataMetrics;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

#[derive(Parser)]
struct Opts {
    #[clap(short, long, default_value = "config.yaml")]
    config: String,

    #[clap(short, long, default_value = "localhost:9090")]
    address: String,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    // Default log level
    match std::env::var_os("RUST_LOG") {
        Some(_) => (),
        None => std::env::set_var("RUST_LOG", "info"),
    }
    env_logger::init();

    let builder = PrometheusBuilder::new();
    let prometheus_handler = builder
        .install_recorder()
        .expect("failed to install recorder");

    data_exporter::init_metrics();

    let metrics = data_exporter::config::parse(opts.config).unwrap();

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/metrics", get(collect_metrics))
        .layer(Extension(metrics))
        .layer(Extension(prometheus_handler));

    // TODO(fredr): add middleware to log all requests

    let addr = opts.address.parse().expect("could not parse address");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("web server terminated");
}

async fn healthz() -> &'static str {
    "OK"
}

async fn collect_metrics(
    metrics: Extension<DataMetrics>,
    prometheus_handler: Extension<PrometheusHandle>,
) -> String {
    metrics.collect().await;
    prometheus_handler.render()
}
