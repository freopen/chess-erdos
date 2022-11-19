use std::{ops::Deref, sync::Arc};

pub use rocksdb::Options;
use serde::{de::DeserializeOwned, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Collection is not registered in this DB")]
    CollectionNotRegistered,
    #[error("RocksDB error")]
    RocksDB(#[from] rocksdb::Error),
    #[error("RMP decode error")]
    RmpDecode(#[from] rmp_serde::decode::Error),
    #[error("RMP encode error")]
    RmpEncode(#[from] rmp_serde::encode::Error),
}

#[derive(Clone)]
pub struct Database(Arc<DatabaseInner>);

impl Deref for Database {
    type Target = DatabaseInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct DatabaseInner {
    rocksdb: rocksdb::DB,
    mutex: std::sync::Mutex<()>,
}

impl Database {
    pub fn build() -> DatabaseBuilder {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
        DatabaseBuilder {
            opts,
            cf_descriptors: vec![],
        }
    }
}

#[derive(Default)]
pub struct DatabaseBuilder {
    opts: Options,
    cf_descriptors: Vec<rocksdb::ColumnFamilyDescriptor>,
}

impl DatabaseBuilder {
    pub fn add_collection<T: Collection>(mut self) -> Self {
        self.cf_descriptors
            .push(rocksdb::ColumnFamilyDescriptor::new(
                T::CF_NAME.to_string(),
                Options::default(),
            ));
        self
    }
    pub fn add_collection_opt<T: Collection>(mut self, opts: Options) -> Self {
        self.cf_descriptors
            .push(rocksdb::ColumnFamilyDescriptor::new(
                T::CF_NAME.to_string(),
                opts,
            ));
        self
    }
    pub fn set_options(mut self, opts: Options) -> Self {
        self.opts = opts;
        self
    }
    pub fn open(self, path: &str) -> Result<Database, rocksdb::Error> {
        let db = rocksdb::DB::open_cf_descriptors(&self.opts, path, self.cf_descriptors)?;
        Ok(Database(Arc::new(DatabaseInner {
            rocksdb: db,
            mutex: std::sync::Mutex::new(()),
        })))
    }
}

pub trait Key {
    fn serialize(&self) -> &[u8];
}

impl Key for () {
    fn serialize(&self) -> &[u8] {
        &[]
    }
}

impl Key for str {
    fn serialize(&self) -> &[u8] {
        self.as_bytes()
    }
}

pub struct CaseInsensitiveString(String);

impl From<&str> for CaseInsensitiveString {
    fn from(s: &str) -> Self {
        Self(s.to_lowercase())
    }
}

impl From<&String> for CaseInsensitiveString {
    fn from(s: &String) -> Self {
        Self(s.to_lowercase())
    }
}

impl<'a> Key for CaseInsensitiveString {
    fn serialize(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

pub trait Collection: Serialize + DeserializeOwned + Sized {
    type KeyType: Key;
    const CF_NAME: &'static str;

    fn get<K: Into<Self::KeyType>>(key: K, db: &Database) -> Result<Option<Self>, Error> {
        let key = key.into();
        let cf = db
            .rocksdb
            .cf_handle(Self::CF_NAME)
            .ok_or(Error::CollectionNotRegistered)?;
        db.rocksdb.get_pinned_cf(cf, key.serialize())?.map_or(Ok(None), |value| {
            rmp_serde::decode::from_slice(&value).map_err(Error::RmpDecode)
        })
    }

    fn modify<K: Into<Self::KeyType>>(
        key: K,
        db: &Database,
        modifier: impl FnOnce(Option<Self>) -> Option<Self>,
    ) -> Result<(), Error> {
        let cf = db
            .rocksdb
            .cf_handle(Self::CF_NAME)
            .ok_or(Error::CollectionNotRegistered)?;
        let key: Self::KeyType = key.into();
        let serialized_key = key.serialize();
        let _guard = db.mutex.lock().unwrap();
        let old_value = db.rocksdb.get_pinned_cf(cf, serialized_key)?.map_or(Ok(None), |value| {
            rmp_serde::decode::from_slice(&value).map_err(Error::RmpDecode)
        })?;
        let value = modifier(old_value);
        if let Some(value) = value {
            db.rocksdb.put_cf(
                cf,
                serialized_key,
                rmp_serde::encode::to_vec(&value).map_err(Error::RmpEncode)?,
            )?;
        } else {
            db.rocksdb.delete_cf(cf, serialized_key)?;
        }
        Ok(())
    }
}
