use async_graphql::{
  extensions::{ApolloTracing, Logger},
  Context, EmptyMutation, EmptySubscription, Enum, Object, Schema, SimpleObject,
};
use chrono::{DateTime, TimeZone, Utc};
use derivative::Derivative;
use mongodb::{bson::doc, Client, Collection, Database};
use serde::{Deserialize, Serialize};

macro_rules! derive_struct {
  {$i:item} => {
    #[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, Derivative)]
    #[derivative(Default)]
    $i
  };
}

macro_rules! derive_enum {
  ($i:item) => {
    #[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Enum, Derivative)]
    #[derivative(Default)]
    $i
  };
}

derive_struct! {
pub struct Meta {
  pub last_processed_archive: String,
}
}

derive_struct! {
pub struct User {
  pub _id: String,
  #[derivative(Default(value="Utc.timestamp(0, 0)"))]
  pub first_game_time: DateTime<Utc>,
  pub erdos_number: i32,
  pub erdos_links: Vec<ErdosLink>,
}
}

derive_struct! {
pub struct ErdosLink {
  pub erdos_number: i32,
  #[derivative(Default(value="Utc.timestamp(0, 0)"))]
  pub time: DateTime<Utc>,
  pub loser_id: String,
  pub winner_info: PlayerInfo,
  pub loser_info: PlayerInfo,
  pub game_id: String,
  pub time_control_type: TimeControlType,
  pub time_control_main: i32,
  pub time_control_increment: i32,
  pub win_type: WinType,
  pub moves_count: i32,
  pub winner_is_white: bool,
}
}

derive_struct! {
pub struct PlayerInfo {
  pub title: Option<String>,
  pub rating: i32,
  pub rating_diff: i32,
}
}

derive_enum! {
pub enum TimeControlType {
  #[derivative(Default)]
  Blitz,
  Rapid,
  Classical,
}
}

derive_enum! {
pub enum WinType {
  #[derivative(Default)]
  Mate,
  Timeout,
  Resign,
}
}

pub struct Query;

#[Object]
impl Query {
  async fn user(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<User> {
    let db: &Db = ctx.data()?;
    if let Some(user) = db.users.find_one(doc! {"_id": id}, None).await? {
      Ok(user)
    } else {
      Err(async_graphql::Error::new("Not found"))
    }
  }
}

pub type SchemaType = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn build_schema(db: Db) -> SchemaType {
  Schema::build(Query, EmptyMutation, EmptySubscription)
    .data(db)
    .extension(ApolloTracing)
    .extension(Logger)
    .finish()
}

#[derive(Clone)]
pub struct Db {
  pub mongo: Client,
  pub db: Database,
  pub users: Collection<User>,
  pub meta: Collection<Meta>,
}

impl Db {
  pub fn new() -> anyhow::Result<Self> {
    let mongo = Client::with_options(Default::default())?;
    let db = mongo.database("chess-erdos");
    let users = db.collection("users");
    let meta = db.collection("meta");
    Ok(Db {
      mongo,
      db,
      users,
      meta,
    })
  }
}
