use actix_web::{get, middleware, App, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;

#[derive(Parser)]
struct Opts {
    #[clap(short, long, default_value = "config.yaml")]
    config: String,

    #[clap(short, long, default_value = "localhost:9090")]
    address: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opts = Opts::parse();

    // Default log level
    match std::env::var_os("RUST_LOG") {
        Some(_) => (),
        None => std::env::set_var("RUST_LOG", "info"),
    }
    env_logger::init();
    data_exporter::init_metrics();

    let metrics_mw = PrometheusMetricsBuilder::new("")
        .endpoint("/metrics")
        .registry(prometheus::default_registry().clone())
        .build()
        .unwrap();

    let dm = data_exporter::config::parse(opts.config).unwrap();
    metrics_mw.registry.register(Box::new(dm)).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(metrics_mw.clone())
            .wrap(middleware::Logger::default())
            .service(healthz)
    })
    .bind(opts.address)
    .unwrap()
    .run()
    .await
}

#[get("/healthz")]
fn healthz() -> HttpResponse {
    HttpResponse::Ok().finish()
}
