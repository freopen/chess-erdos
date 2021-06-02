#![feature(destructuring_assignment)]
mod server;

use tokio::try_join;

use crate::server::{db, grpc::tonic_server_task, process_archive::process_new_archives_task};

#[tokio::main]
async fn main() {
    env_logger::init();
    db::init_db();
    try_join!(process_new_archives_task(), tonic_server_task()).unwrap();
}
