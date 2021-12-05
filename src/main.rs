use std::net::SocketAddr;

use anyhow::Result;
use opentelemetry::KeyValue;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

mod grpc;
mod process_archive;
mod util;

#[allow(clippy::enum_variant_names)]
mod proto {
  tonic::include_proto!("chess_erdos");
  pbdb::pbdb_impls!();
}

fn register_metrics() {
  use metrics::register_counter;
  register_counter!("games_processed", "Number of games processed");
  register_counter!("games_skipped", "Number of games skipped");
  register_counter!("erdos_updated", "Number of erdos number updates");
}

#[tokio::main]
async fn main() -> Result<()> {
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
    .with(tracing_subscriber::EnvFilter::new("INFO"))
    .with(tracing_opentelemetry::layer().with_tracer(tracer))
    .try_init()?;
  metrics_exporter_prometheus::PrometheusBuilder::new()
    .listen_address("127.0.0.1:40000".parse::<SocketAddr>()?)
    .install()?;
  register_metrics();

  let _db_guard = proto::open_db(std::path::Path::new("db"))?;

  tokio::select! {
    v = grpc::serve() => v?,
    v = process_archive::process_new_archives_task() => v?,
  }

  opentelemetry::global::shutdown_tracer_provider();

  Ok(())
}
