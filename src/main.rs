use proto::chess_erdos_service_server::ChessErdosServiceServer;

mod grpc;
mod process_archive;
mod util;

#[allow(clippy::enum_variant_names)]
mod proto {
  tonic::include_proto!("chess_erdos");
  pbdb::pbdb_impls!();
}

#[tokio::main]
async fn main() {
  env_logger::init();
  let _db_guard = proto::open_db(std::path::Path::new("db")).unwrap();

  let chess_erdos_service = grpc::ChessErdosServiceImpl::default();
  let grpc_web_service = tonic_web::config()
    .allow_origins(vec!["127.0.0.1"])
    .enable(ChessErdosServiceServer::new(chess_erdos_service));
  let grpc_server = tonic::transport::Server::builder()
    .accept_http1(true)
    .add_service(grpc_web_service)
    .serve("127.0.0.1:3000".parse().unwrap());
  let process_archives_task = process_archive::process_new_archives_task();

  tokio::select! {
    v = grpc_server => v.unwrap(),
    v = process_archives_task => v.unwrap(),
  }
}
