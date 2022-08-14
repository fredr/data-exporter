use axum::{routing::get, Extension, Router};
use clap::Parser;
use data_exporter::log_filter::LogFilter;
use data_exporter::DataMetrics;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tower_http::trace::TraceLayer;
use tracing::{dispatcher, Dispatch, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

#[derive(Parser)]
struct Opts {
    #[clap(short, long, default_value = "config.yaml")]
    config: String,

    #[clap(short, long, default_value = "localhost:9090")]
    address: String,

    #[clap(short = 'L', long, default_value = "info")]
    log_level: Level,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let subscriber = Registry::default()
        .with(tracing_logfmt::layer().with_filter(LogFilter::new(opts.log_level)));

    dispatcher::set_global_default(Dispatch::new(subscriber))
        .expect("failed setting up global dispatcher");

    let builder = PrometheusBuilder::new();
    let prometheus_handler = builder
        .install_recorder()
        .expect("failed to install recorder");

    let metrics = data_exporter::config::parse(opts.config).unwrap();
    data_exporter::init_metrics(&metrics);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/metrics", get(collect_metrics))
        .layer(Extension(metrics))
        .layer(Extension(prometheus_handler))
        .layer(TraceLayer::new_for_http());

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
