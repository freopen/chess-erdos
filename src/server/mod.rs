pub(crate) mod db;
pub(crate) mod grpc;
pub(crate) mod process_archive;
pub(crate) mod proto {
    tonic::include_proto!("chess_erdos");
}
