use anyhow::Result;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use rocksdb_ext::DatasetConfig;
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

    let mut db_config = rocksdb_ext::DBConfig::default();
    db_config.opts.create_if_missing(true);
    db_config.opts.create_missing_column_families(true);
    let users_config = rocksdb_ext::CollectionConfig::new("users");
    let erdos_links_config = rocksdb_ext::CollectionConfig::new("erdos_links");
    let last_processed_archive_config =
        rocksdb_ext::CollectionConfig::new("last_processed_archive");
    let game_checkpoint_config = rocksdb_ext::CollectionConfig::new("game_checkpoint");
    users_config.update_db_config(&mut db_config);
    erdos_links_config.update_db_config(&mut db_config);
    last_processed_archive_config.update_db_config(&mut db_config);
    game_checkpoint_config.update_db_config(&mut db_config);
    let db = db_config.open(std::path::Path::new("db"))?;
    let users = users_config.open(&db);
    let erdos_links = erdos_links_config.open(&db);
    let last_processed_archive = last_processed_archive_config.open(&db);
    let game_checkpoint = game_checkpoint_config.open(&db);

    let result = tokio::select! {
      // v = http::serve(&users, &erdos_links, &last_processed_archive) => v,
      v = process_archive::process_new_archives_task(&users, &erdos_links, &last_processed_archive, &game_checkpoint) => v,
    };

    opentelemetry::global::shutdown_tracer_provider();

    result
}
