use actix_web::{get, App, HttpResponse, HttpServer};
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

    let prometheus = PrometheusMetricsBuilder::new("data_exporter")
        .registry(prometheus::default_registry().clone())
        .endpoint("/metrics")
        .build()
        .unwrap();

    let dm = data_exporter::config::parse(opts.config).unwrap();

    prometheus.registry.register(Box::new(dm)).unwrap();

    HttpServer::new(move || App::new().wrap(prometheus.clone()).service(healthz))
        .bind(opts.address)
        .unwrap()
        .run()
        .await
}

#[get("/healthz")]
fn healthz() -> HttpResponse {
    HttpResponse::Ok().finish()
}
