use std::marker::PhantomData;

use anyhow::{Context, Result, ensure};
use lazy_static::lazy_static;
use prost::Message;
use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, MergeOperands, Options};

use crate::proto;

pub trait DBColumnDescriptor: 'static {
    type ValueType: Message + Default;
    type MergeType: Message + Default;
    const NAME: &'static str;
    fn fill_options(_cf_desc: &mut Options) {}
    fn cf() -> ColumnFamilyDescriptor {
        let mut opts = Options::default();
        opts.set_merge_operator("", Self::merge_operator, Self::partial_merge_operator);
        Self::fill_options(&mut opts);
        ColumnFamilyDescriptor::new(Self::NAME, opts)
    }
    fn apply_merge(
        _key: &[u8],
        _value: &mut Self::ValueType,
        _merge: Self::MergeType,
    ) -> Result<()> {
        unimplemented!();
    }
    fn merge_operator(
        new_key: &[u8],
        existing_val: Option<&[u8]>,
        operands: &mut MergeOperands,
    ) -> Option<Vec<u8>> {
        let mut value = Self::ValueType::decode(existing_val?).unwrap();
        for op in operands {
            let op = Self::MergeType::decode(op).unwrap();
            Self::apply_merge(new_key, &mut value, op).unwrap();
        }
        let mut bytes = vec![];
        value.encode(&mut bytes).unwrap();
        Some(bytes)
    }
    fn partial_merge_operator(
        _new_key: &[u8],
        _existing_val: Option<&[u8]>,
        _operands: &mut MergeOperands,
    ) -> Option<Vec<u8>> {
        None
    }
}

pub struct DBColumn<'a, T> {
    db: &'a rocksdb::DB,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: DBColumnDescriptor> DBColumn<'a, T> {
    pub fn new(db: &'a rocksdb::DB) -> Self {
        DBColumn {
            db,
            phantom: PhantomData,
        }
    }
    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<T::ValueType>> {
        match self.db.get_cf(self.db.cf_handle(T::NAME).unwrap(), key)? {
            Some(bytes) => Ok(Some(T::ValueType::decode(bytes.as_slice())?)),
            None => Ok(None),
        }
    }
    pub fn put<K: AsRef<[u8]>>(&self, key: K, value: &T::ValueType) -> Result<()> {
        let mut bytes = vec![];
        value.encode(&mut bytes)?;
        self.db
            .put_cf(self.db.cf_handle(T::NAME).unwrap(), key, bytes)?;
        Ok(())
    }
    pub fn merge<K: AsRef<[u8]>>(&self, key: K, merge: T::MergeType) -> Result<()> {
        let mut bytes = vec![];
        merge.encode(&mut bytes)?;
        self.db
            .merge_cf(self.db.cf_handle(T::NAME).unwrap(), key, bytes)?;
        Ok(())
    }
}

pub struct Users;

impl DBColumnDescriptor for Users {
    type ValueType = proto::User;
    type MergeType = proto::UserUpdate;
    const NAME: &'static str = "users";
    fn apply_merge(_key: &[u8], value: &mut Self::ValueType, merge: Self::MergeType) -> Result<()> {
        match merge.update.context("empty update")? {
            proto::user_update::Update::NewErdosLink(erdos_link) => {
                ensure!(value.erdos_number > erdos_link.erdos_number, "Update does not improve Erdos number");
                value.erdos_number = erdos_link.erdos_number;
                value.erdos_links.push(erdos_link);
            }
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref DB: rocksdb::DB = {
        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);
        rocksdb::DB::open_cf_descriptors(&db_opts, "db", vec![Users::cf()]).unwrap()
    };
    pub static ref DB_USERS: DBColumn<'static, Users> = DBColumn::new(&DB);
}
