mod db;
mod process_archive;

use crate::{
    db::DB_USERS,
    process_archive::{process_new_archives, ERDOS_ID},
};

mod proto {
    tonic::include_proto!("chess_erdos");
}

fn main() {
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
    process_new_archives().unwrap();
}
