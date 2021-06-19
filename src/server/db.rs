use chrono::{DateTime, TimeZone, Utc};
use lazy_static::lazy_static;
use mongodb::{Client, Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Meta {
  pub last_processed_archive: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
  pub _id: String,
  pub first_game_time: DateTime<Utc>,
  pub erdos_number: i32,
  pub erdos_links: Vec<ErdosLink>,
}

impl Default for User {
  fn default() -> Self {
    User {
      _id: String::new(),
      first_game_time: Utc.timestamp(0, 0),
      erdos_number: 0,
      erdos_links: vec![],
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErdosLink {
  pub erdos_number: i32,
  pub time: DateTime<Utc>,
  pub loser_id: String,
  pub game_info: GameInfo,
  pub winner_info: PlayerInfo,
  pub loser_info: PlayerInfo,
}

impl Default for ErdosLink {
  fn default() -> Self {
    ErdosLink {
      erdos_number: 0,
      time: Utc.timestamp(0, 0),
      loser_id: String::new(),
      game_info: GameInfo::default(),
      winner_info: PlayerInfo::default(),
      loser_info: PlayerInfo::default(),
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GameInfo {
  pub id: String,
  pub time_control: TimeControl,
  pub win_type: WinType,
  pub moves_count: i32,
  pub winner_is_white: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PlayerInfo {
  pub title: Option<String>,
  pub rating: i32,
  pub rating_diff: i32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TimeControl {
  pub category: TimeControlCategory,
  pub main: i32,
  pub increment: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TimeControlCategory {
  Blitz,
  Rapid,
  Classical,
}

impl Default for TimeControlCategory {
  fn default() -> Self {
    TimeControlCategory::Blitz
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WinType {
  Mate,
  Timeout,
  Resign,
}

impl Default for WinType {
  fn default() -> Self {
    WinType::Mate
  }
}

lazy_static! {
  pub static ref MONGO: Client = Client::with_options(Default::default()).unwrap();
  pub static ref DB: Database = MONGO.database("chess-erdos");
  pub static ref USERS: Collection<User> = DB.collection("users");
  pub static ref META: Collection<Meta> = DB.collection("meta");
}
