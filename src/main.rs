mod db;
mod grpc;
mod process_archive;

use tokio::try_join;

use crate::{
    db::DB_USERS,
    grpc::tonic_server_task,
    process_archive::{process_new_archives_task, ERDOS_ID},
};

mod proto {
    tonic::include_proto!("chess_erdos");
}

#[tokio::main]
async fn main() {
    env_logger::init();
    DB_USERS
        .put(
            ERDOS_ID.to_ascii_lowercase(),
            &proto::User {
                id: ERDOS_ID.into(),
                erdos_number: 0,
                erdos_links: vec![],
            },
        )
        .unwrap();
    try_join!(process_new_archives_task(), tonic_server_task()).unwrap();
}
