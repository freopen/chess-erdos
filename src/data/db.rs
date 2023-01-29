use rocksdb_ext::{Collection, DatasetConfig};

use crate::data::*;

#[derive(Clone)]
pub struct DB {
    pub db: rocksdb_ext::DB,
    pub users: Collection<String, User>,
    pub erdos_links: Collection<(String, u32, u32), ErdosLink>,
    pub last_processed_archive: Collection<(), String>,
    pub game_checkpoint: Collection<(), String>,
}

impl DB {
    pub fn new() -> Result<Self> {
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
        Ok(DB {
            db,
            users,
            erdos_links,
            last_processed_archive,
            game_checkpoint,
        })
    }
}
