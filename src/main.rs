#![feature(destructuring_assignment)]
mod db;
mod grpc;
mod process_archive;

use tokio::try_join;

use crate::{grpc::tonic_server_task, process_archive::process_new_archives_task};

mod proto {
    tonic::include_proto!("chess_erdos");
}

#[tokio::main]
async fn main() {
    env_logger::init();
    db::init_db();
    try_join!(process_new_archives_task(), tonic_server_task()).unwrap();
}
