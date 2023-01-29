use anyhow::Result;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

// mod http;
mod process_archive;

fn register_metrics() {
    use metrics::register_counter;
    register_counter!("games_processed");
    register_counter!("games_skipped");
    register_counter!("erdos_updated");
}

pub async fn serve() -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317/"),
        )
        .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
            opentelemetry::sdk::Resource::new(vec![
                KeyValue::new("service.name", "chess_erdos"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ]),
        ))
        .install_batch(opentelemetry::runtime::Tokio)?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("INFO"))
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::FULL)
                .compact(),
        )
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(([127, 0, 0, 1], 4001))
        .add_global_label("service.name", "chess_erdos")
        .add_global_label("service.version", env!("CARGO_PKG_VERSION"))
        .install()?;
    register_metrics();

    let db = crate::data::db::DB::new()?;

    let result = tokio::select! {
      // v = http::serve(&users, &erdos_links, &last_processed_archive) => v,
      v = process_archive::process_new_archives_task(&db) => v,
    };

    opentelemetry::global::shutdown_tracer_provider();

    result
}
