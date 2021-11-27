use std::net::SocketAddr;

use anyhow::Result;
use tracing_subscriber::prelude::*;

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
  opentelemetry::global::set_text_map_propagator(
    opentelemetry::sdk::propagation::TraceContextPropagator::new(),
  );
  let tracer = opentelemetry_jaeger::new_pipeline()
    .with_service_name("chess-erdos")
    .install_batch(opentelemetry::runtime::Tokio)?;
  tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new("INFO"))
    .with(tracing_opentelemetry::layer().with_tracer(tracer))
    .with(tracing_subscriber::fmt::layer().json())
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
