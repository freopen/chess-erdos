use std::{borrow::Cow, marker::PhantomData};

use heed::{
    types::{Str, Unit},
    BytesDecode, BytesEncode, Database, Env, EnvOpenOptions,
};
use lazy_static::{initialize, lazy_static};
use prost::Message;

use crate::proto::{Meta, User};

pub struct Proto<T: Message>(PhantomData<T>);

impl<'a, T: Message + 'a> BytesEncode<'a> for Proto<T> {
    type EItem = T;

    fn bytes_encode(
        item: &'a Self::EItem,
    ) -> core::result::Result<Cow<'a, [u8]>, Box<dyn std::error::Error>> {
        let mut bytes = Vec::with_capacity(item.encoded_len());
        item.encode(&mut bytes)?;
        Ok(bytes.into())
    }
}

impl<'a, T: Message + Default + 'a> BytesDecode<'a> for Proto<T> {
    type DItem = T;

    fn bytes_decode(
        bytes: &'a [u8],
    ) -> core::result::Result<Self::DItem, Box<dyn std::error::Error>> {
        Ok(T::decode(bytes)?)
    }
}

lazy_static! {
    pub static ref ENV: Env = EnvOpenOptions::new()
        .max_dbs(2)
        .map_size(10 * 1024 * 1024 * 1024)
        .open("db")
        .unwrap();
    pub static ref META: Database<Unit, Proto<Meta>> = ENV.create_database(Some("meta")).unwrap();
    pub static ref USERS: Database<Str, Proto<User>> = ENV.create_database(Some("users")).unwrap();
}

pub fn init_db() {
    initialize(&ENV);
    initialize(&META);
    initialize(&USERS);
}
