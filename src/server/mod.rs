use anyhow::Result;
use bonsaidb::local::{
    config::{Builder, StorageConfiguration},
    Database,
};
use opentelemetry::KeyValue;
use tracing_subscriber::prelude::*;

use crate::data::DbSchema;

mod http;
mod process_archive;

fn register_metrics() {
    use metrics::register_counter;
    register_counter!("games_processed");
    register_counter!("games_skipped");
    register_counter!("erdos_updated");
}

pub async fn serve() -> Result<()> {
    let otlp_exporter = opentelemetry_otlp::new_exporter().tonic();
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
            opentelemetry::sdk::Resource::new(vec![
                KeyValue::new("service.name", "chess_erdos"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ]),
        ))
        .install_batch(opentelemetry::runtime::Tokio)?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("DEBUG"))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;
    metrics_exporter_prometheus::PrometheusBuilder::new().install()?;
    // let debug_metrics = metrics_util::debugging::DebuggingRecorder::new();
    // let snapshotter = debug_metrics.snapshotter();
    // tokio::spawn(async move {
    //     let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
    //     loop {
    //         interval.tick().await;
    //         dbg!(snapshotter.snapshot().into_hashmap());
    //     }
    // });
    // debug_metrics.install()?;

    register_metrics();

    let db = Database::open::<DbSchema>(StorageConfiguration::new("schema.bonsaidb"))?;

    let result = tokio::select! {
      v = http::serve(&db) => v,
    //   v = process_archive::process_new_archives_task(&db) => v,
    };

    opentelemetry::global::shutdown_tracer_provider();

    result
}
