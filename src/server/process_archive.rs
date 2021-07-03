use std::process::{Command, Stdio};
use std::{collections::HashMap, time::Duration};

use anyhow::{ensure, Context, Result};
use chrono::{TimeZone, Utc};
use futures::executor::block_on;
use log::info;
use mongodb::bson::{doc, to_bson};
use pgn_reader::{RawHeader, SanPlus, Skip, Visitor};
use reqwest::get;
use shakmaty::san::Suffix;
use tokio::{task::spawn_blocking, time::sleep};

use super::data::{Db, ErdosLink, Meta, PlayerInfo, TimeControlType, User, WinType};

const ERDOS_NUMBER_INF: i32 = i32::MAX - 1;
const ERDOS_ID: &str = "gorizav"; //"DrNykterstein";
const LICHESS_DB_LIST: &str = "https://database.lichess.org/standard/list.txt";
const DATETIME_FORMAT: &str = "%Y.%m.%d %H:%M:%S";

struct GameParser {
  user_id: String,
  erdos_link: ErdosLink,
  headers: HashMap<String, String>,
  skip: bool,
  users_cache: HashMap<String, i32>,
  db: Db,
}

impl GameParser {
  fn new(db: Db) -> Self {
    let mut users_cache = HashMap::new();
    users_cache.insert("?".to_string(), ERDOS_NUMBER_INF);
    users_cache.insert(ERDOS_ID.to_string(), 0);
    GameParser {
      user_id: String::new(),
      erdos_link: ErdosLink::default(),
      headers: HashMap::new(),
      skip: false,
      users_cache,
      db,
    }
  }
}

impl GameParser {
  fn get_latest_erdos_number(&mut self, id: &str) -> i32 {
    if let Some(erdos_number) = self.users_cache.get(id) {
      *erdos_number
    } else if let Some(user) = block_on(self.db.users.find_one(doc! {"_id": &id}, None)).unwrap() {
      self.users_cache.insert(id.to_string(), user.erdos_number);
      user.erdos_number
    } else {
      block_on(
        self.db.users.insert_one(
          User {
            _id: id.to_string(),
            erdos_number: ERDOS_NUMBER_INF,
            first_game_time: Utc
              .datetime_from_str(
                &format!(
                  "{} {}",
                  self.headers.get("UTCDate").unwrap(),
                  self.headers.get("UTCTime").unwrap()
                ),
                DATETIME_FORMAT,
              )
              .unwrap(),
            erdos_links: vec![],
          },
          None,
        ),
      )
      .unwrap();
      self.users_cache.insert(id.to_string(), ERDOS_NUMBER_INF);
      ERDOS_NUMBER_INF
    }
  }
  fn parse_user_data(&mut self, for_white: bool) -> Option<PlayerInfo> {
    Some(PlayerInfo {
      title: self.headers.remove(if for_white {
        "WhiteTitle"
      } else {
        "BlackTitle"
      }),
      rating: self
        .headers
        .remove(if for_white { "WhiteElo" } else { "BlackElo" })
        .unwrap()
        .parse()
        .unwrap(),
      rating_diff: self
        .headers
        .remove(if for_white {
          "WhiteRatingDiff"
        } else {
          "BlackRatingDiff"
        })? // Cheaters often has no diffs, skip PGNs without diffs.
        .parse()
        .unwrap(),
    })
  }

  fn headers_to_erdos_link(&mut self) -> Option<()> {
    let white = self.headers.remove("White").unwrap();
    let white_erdos_number = self.get_latest_erdos_number(&white);
    let black = self.headers.remove("Black").unwrap();
    let black_erdos_number = self.get_latest_erdos_number(&black);
    if white == "?" || black == "?" || (white_erdos_number - black_erdos_number).abs() < 2 {
      return None;
    }

    let event = self.headers.remove("Event").unwrap();
    let without_rated = event.strip_prefix("Rated ")?;
    if without_rated.starts_with("Blitz ") {
      self.erdos_link.time_control_type = TimeControlType::Blitz;
    } else if without_rated.starts_with("Rapid ") {
      self.erdos_link.time_control_type = TimeControlType::Rapid;
    } else if without_rated.starts_with("Classical ") {
      self.erdos_link.time_control_type = TimeControlType::Classical;
    } else {
      return None;
    }

    match self.headers.remove("Result").as_deref() {
      Some("1-0") => {
        if white_erdos_number <= black_erdos_number + 1 {
          return None;
        }
        self.erdos_link.winner_is_white = true;
        self.user_id = white;
        self.erdos_link.loser_id = black;
      }
      Some("0-1") => {
        if black_erdos_number <= white_erdos_number + 1 {
          return None;
        }
        self.erdos_link.winner_is_white = false;
        self.user_id = black;
        self.erdos_link.loser_id = white;
      }
      _ => {
        return None;
      }
    };

    self.erdos_link.winner_info = self.parse_user_data(self.erdos_link.winner_is_white)?;
    self.erdos_link.loser_info = self.parse_user_data(!self.erdos_link.winner_is_white)?;

    self.erdos_link.win_type = match self.headers.remove("Termination").unwrap().as_str() {
      "Normal" => WinType::Resign,
      "Time forfeit" => WinType::Timeout,
      _ => return None,
    };

    self.erdos_link.time = Utc
      .datetime_from_str(
        &format!(
          "{} {}",
          self.headers.remove("UTCDate").unwrap(),
          self.headers.remove("UTCTime").unwrap(),
        ),
        DATETIME_FORMAT,
      )
      .unwrap();

    self.erdos_link.game_id = self
      .headers
      .remove("Site")
      .unwrap()
      .strip_prefix("https://lichess.org/")
      .unwrap()
      .to_string();

    let time_control_str = self.headers.remove("TimeControl").unwrap();
    let (main_str, increment_str) = time_control_str.split_once('+').unwrap();
    self.erdos_link.time_control_main = main_str.parse().unwrap();
    self.erdos_link.time_control_increment = increment_str.parse().unwrap();

    self.erdos_link.moves_count = 0;

    Some(())
  }
}

impl Visitor for GameParser {
  type Result = ();

  fn begin_game(&mut self) {
    self.skip = false;
    self.headers.clear();
  }

  fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
    self.headers.insert(
      String::from_utf8(key.into()).unwrap(),
      value.decode_utf8().unwrap().to_string(),
    );
  }

  fn end_headers(&mut self) -> Skip {
    self.skip = self.headers_to_erdos_link().is_none();
    Skip(self.skip)
  }

  fn san(&mut self, san: SanPlus) {
    self.erdos_link.moves_count += 1;
    if san.suffix == Some(Suffix::Checkmate) {
      self.erdos_link.win_type = WinType::Mate;
    }
  }

  fn begin_variation(&mut self) -> Skip {
    Skip(true)
  }

  fn end_game(&mut self) -> Self::Result {
    if !self.skip && self.erdos_link.moves_count >= 20 {
      let winner = block_on(self.db.users.find_one(doc! {"_id": &self.user_id}, None))
        .unwrap()
        .unwrap();
      let loser = block_on(
        self
          .db
          .users
          .find_one(doc! {"_id": &self.erdos_link.loser_id}, None),
      )
      .unwrap()
      .unwrap();
      let loser_erdos_number = if loser.erdos_number == 0 {
        0
      } else {
        loser
          .erdos_links
          .into_iter()
          .find(|erdos_link| erdos_link.time < self.erdos_link.time)
          .map(|erdos_link| erdos_link.erdos_number)
          .unwrap_or(ERDOS_NUMBER_INF)
      };
      if winner.erdos_number > loser_erdos_number + 1 {
        self.erdos_link.erdos_number = loser_erdos_number + 1;
        block_on(self.db.users.update_one(
          doc! {"_id": &self.user_id},
          doc! {
            "$set": {"erdos_number": self.erdos_link.erdos_number},
            "$push": {"erdos_links": to_bson(&self.erdos_link).unwrap()}
          },
          None,
        ))
        .unwrap();
        *self.users_cache.get_mut(&self.user_id).unwrap() = self.erdos_link.erdos_number;
      }
    }
  }
}

fn process_archive(url: &str, db: Db) -> Result<()> {
  let mut curl_child = Command::new("curl")
    .arg(url)
    .stdout(Stdio::piped())
    .spawn()?;
  let curl_output = curl_child.stdout.take().context("No curl stdout")?;
  let mut pbzip_child = Command::new("pbunzip2")
    .stdin(curl_output)
    .stdout(Stdio::piped())
    .spawn()?;
  let pbzip_output = pbzip_child.stdout.take().context("No pbzip stdout")?;
  let mut pgn_read = pgn_reader::BufferedReader::new(pbzip_output);
  let mut game_parser = GameParser::new(db);
  pgn_read.read_all(&mut game_parser)?;
  ensure!(curl_child.wait()?.success(), "Curl failed");
  ensure!(pbzip_child.wait()?.success(), "Pbzip failed");
  Ok(())
}

pub async fn process_new_archives_task(db: Db) -> Result<()> {
  if db.meta.find_one(None, None).await?.is_none() {
    db.meta
      .insert_one(
        Meta {
          last_processed_archive:
            "https://database.lichess.org/standard/lichess_db_standard_rated_2019-05.pgn.bz2"
              .to_string(),
        },
        None,
      )
      .await?;
  }
  if db
    .users
    .find_one(doc! {"_id": ERDOS_ID}, None)
    .await?
    .is_none()
  {
    let user = User {
      _id: ERDOS_ID.to_string(),
      erdos_number: 0,
      ..Default::default()
    };
    dbg!(&user);
    db.users.insert_one(user, None).await?;
  }
  loop {
    info!("Processing new archives");
    let last_archive = db
      .meta
      .find_one(doc! {}, None)
      .await?
      .context("No meta record found")?
      .last_processed_archive;
    let lichess_archives: Vec<String> = get(LICHESS_DB_LIST)
      .await?
      .text()
      .await?
      .split_ascii_whitespace()
      .rev()
      .map(String::from)
      .skip_while(|archive| archive <= &last_archive)
      .collect();
    info!("New archives found: {}", lichess_archives.len());
    for archive in lichess_archives {
      info!("Processing archive url: {}", &archive);
      let archive_clone = archive.clone();
      let db_clone = db.clone();
      spawn_blocking(move || process_archive(&archive_clone, db_clone)).await??;
      db.meta
        .update_one(
          doc! {},
          doc! {"$set": {"last_processed_archive": &archive}},
          None,
        )
        .await?;
      info!("Archive url processed: {}", &archive);
    }
    sleep(Duration::from_secs(60 * 60)).await;
  }
}
