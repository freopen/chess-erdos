mod process_archive;

mod proto {
  tonic::include_proto!("chess_erdos");
  pbdb::pbdb_impls!();
}

#[tokio::main]
async fn main() {
  env_logger::init();
  let _db_guard = proto::open_db(std::path::Path::new("db")).unwrap();
  process_archive::process_new_archives_task().await.unwrap();
}
