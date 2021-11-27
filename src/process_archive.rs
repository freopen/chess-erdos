use std::process::{Command, Stdio};
use std::{collections::HashMap, time::Duration};

use anyhow::{ensure, Context, Result};
use chrono::{TimeZone, Utc};
use metrics::increment_counter;
use pbdb::{Collection, SingleRecord};
use pgn_reader::{RawHeader, SanPlus, Skip, Visitor};
use reqwest::get;
use shakmaty::san::Suffix;
use tokio::{task::spawn_blocking, time::sleep};
use tracing::info;

use crate::{
  proto::{self, ErdosLink, TimeControlType, User, WinType},
  util::{user_to_erdos_number, user_to_erdos_number_at, ERDOS_ID, ERDOS_NUMBER_INF},
};

const LICHESS_DB_LIST: &str = "https://database.lichess.org/standard/list.txt";
const DATETIME_FORMAT: &str = "%Y.%m.%d %H:%M:%S";

struct GameParser {
  user_id: String,
  erdos_link: ErdosLink,
  headers: HashMap<String, String>,
  skip: bool,
  users_cache: HashMap<String, i32>,
}

impl GameParser {
  fn new() -> Self {
    let mut users_cache = HashMap::new();
    users_cache.insert("?".to_string(), ERDOS_NUMBER_INF);
    users_cache.insert(ERDOS_ID.to_string(), 0);
    GameParser {
      user_id: String::new(),
      erdos_link: ErdosLink::default(),
      headers: HashMap::new(),
      skip: false,
      users_cache,
    }
  }
}

impl GameParser {
  fn get_latest_erdos_number(&mut self, id: &str) -> Result<i32> {
    if let Some(erdos_number) = self.users_cache.get(id) {
      Ok(*erdos_number)
    } else if let Some(user) = User::get(&id.to_string())? {
      let erdos_number = user_to_erdos_number(&user);
      self.users_cache.insert(id.to_string(), erdos_number);
      Ok(erdos_number)
    } else {
      User {
        id: id.to_string(),
        first_game: Utc
          .datetime_from_str(
            &format!(
              "{} {}",
              self.headers.get("UTCDate").unwrap(),
              self.headers.get("UTCTime").unwrap()
            ),
            DATETIME_FORMAT,
          )?
          .timestamp(),
        erdos_links: vec![],
      }
      .put()?;
      self.users_cache.insert(id.to_string(), ERDOS_NUMBER_INF);
      Ok(ERDOS_NUMBER_INF)
    }
  }

  fn headers_to_erdos_link(&mut self) -> Result<bool> {
    let white = self.headers.remove("White").unwrap();
    let white_erdos_number = self.get_latest_erdos_number(&white)?;
    let black = self.headers.remove("Black").unwrap();
    let black_erdos_number = self.get_latest_erdos_number(&black)?;
    if white == "?" || black == "?" || (white_erdos_number - black_erdos_number).abs() < 2 {
      increment_counter!("games_skipped", "reason" => "fast_erdos");
      return Ok(false);
    }

    let event = self.headers.remove("Event").context("No Event header")?;
    let without_rated = if let Some(without_rated) = event.strip_prefix("Rated ") {
      without_rated
    } else {
      increment_counter!("games_skipped", "reason" => "unrated");
      return Ok(false); // Skip unrated games.
    };
    if without_rated.starts_with("Blitz ") {
      self
        .erdos_link
        .set_time_control_type(TimeControlType::Blitz);
    } else if without_rated.starts_with("Rapid ") {
      self
        .erdos_link
        .set_time_control_type(TimeControlType::Rapid);
    } else if without_rated.starts_with("Classical ") {
      self
        .erdos_link
        .set_time_control_type(TimeControlType::Classical);
    } else {
      increment_counter!("games_skipped", "reason" => format!("timecontrol: {}", without_rated));
      return Ok(false); // Skip other time control types.
    }

    match self.headers.remove("Result").as_deref() {
      Some("1-0") => {
        if white_erdos_number <= black_erdos_number + 1 {
          increment_counter!("games_skipped", "reason" => "erdos");
          return Ok(false); // Erdos number is not improving.
        }
        self.erdos_link.winner_is_white = true;
        self.user_id = white;
        self.erdos_link.loser_id = black;
      }
      Some("0-1") => {
        if black_erdos_number <= white_erdos_number + 1 {
          increment_counter!("games_skipped", "reason" => "erdos");
          return Ok(false); // Erdos number is not improving.
        }
        self.erdos_link.winner_is_white = false;
        self.user_id = black;
        self.erdos_link.loser_id = white;
      }
      _ => {
        increment_counter!("games_skipped", "reason" => "draw");
        return Ok(false); // Skip draws.
      }
    };

    self.erdos_link.winner_title = self
      .headers
      .remove(if self.erdos_link.winner_is_white {
        "WhiteTitle"
      } else {
        "BlackTitle"
      })
      .unwrap_or_default();
    self.erdos_link.winner_rating = self
      .headers
      .remove(if self.erdos_link.winner_is_white {
        "WhiteElo"
      } else {
        "BlackElo"
      })
      .context("No Elo header")?
      .parse()?;
    self.erdos_link.winner_rating_diff = if let Some(rating_diff) =
      self.headers.remove(if self.erdos_link.winner_is_white {
        "WhiteRatingDiff"
      } else {
        "BlackRatingDiff"
      }) {
      rating_diff.parse()?
    } else {
      increment_counter!("games_skipped", "reason" => "cheater");
      return Ok(false); // Cheaters often have no diffs, skip PGNs without diffs.
    };

    self.erdos_link.loser_title = self
      .headers
      .remove(if self.erdos_link.winner_is_white {
        "BlackTitle"
      } else {
        "WhiteTitle"
      })
      .unwrap_or_default();
    self.erdos_link.loser_rating = self
      .headers
      .remove(if self.erdos_link.winner_is_white {
        "BlackElo"
      } else {
        "WhiteElo"
      })
      .context("No Elo header")?
      .parse()?;
    self.erdos_link.loser_rating_diff = if let Some(rating_diff) =
      self.headers.remove(if self.erdos_link.winner_is_white {
        "BlackRatingDiff"
      } else {
        "WhiteRatingDiff"
      }) {
      rating_diff.parse()?
    } else {
      increment_counter!("games_skipped", "reason" => "cheater");
      return Ok(false); // Cheaters often have no diffs, skip PGNs without diffs.
    };

    self.erdos_link.set_win_type(
      match self
        .headers
        .remove("Termination")
        .context("No Termination header")?
        .as_str()
      {
        "Normal" => WinType::Resign,
        "Time forfeit" => WinType::Timeout,
        termination => {
          increment_counter!("games_skipped", "reason" => format!("termination: {}", termination));
          return Ok(false);
        } // Unknown termination type, safer to skip.
      },
    );

    self.erdos_link.time = Utc
      .datetime_from_str(
        &format!(
          "{} {}",
          self.headers.remove("UTCDate").unwrap(),
          self.headers.remove("UTCTime").unwrap(),
        ),
        DATETIME_FORMAT,
      )?
      .timestamp();

    self.erdos_link.game_id = self
      .headers
      .remove("Site")
      .context("No Site header")?
      .strip_prefix("https://lichess.org/")
      .context("Unexpected Site header")?
      .to_string();

    let time_control_str = self
      .headers
      .remove("TimeControl")
      .context("No TimeControl header")?;
    let (main_str, increment_str) = time_control_str
      .split_once('+')
      .context("Unexpected TimeControl format")?;
    self.erdos_link.time_control_main = main_str.parse()?;
    self.erdos_link.time_control_increment = increment_str.parse()?;

    self.erdos_link.move_count = 0;

    Ok(true)
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
    self.skip = !self.headers_to_erdos_link().unwrap();
    Skip(self.skip)
  }

  fn san(&mut self, san: SanPlus) {
    self.erdos_link.move_count += 1;
    if san.suffix == Some(Suffix::Checkmate) {
      self.erdos_link.set_win_type(WinType::Checkmate);
    }
  }

  fn begin_variation(&mut self) -> Skip {
    Skip(true)
  }

  fn end_game(&mut self) -> Self::Result {
    increment_counter!("games_processed");
    if !self.skip {
      if self.erdos_link.move_count < 20 {
        increment_counter!("games_skipped", "reason" => "short");
        return; // Skip games with less than 20 moves.
      }
      let mut winner = User::get(&self.user_id)
        .unwrap()
        .expect("User should be in DB at this point");
      let loser_erdos_number = if self.erdos_link.loser_id == ERDOS_ID {
        0
      } else {
        user_to_erdos_number_at(
          &User::get(&self.erdos_link.loser_id)
            .unwrap()
            .expect("User should be in DB at this point"),
          self.erdos_link.time,
        )
      };
      let winner_erdos_number = user_to_erdos_number(&winner);
      if winner_erdos_number > loser_erdos_number + 1 {
        increment_counter!(
          "erdos_updated",
          "new" => format!("{}", loser_erdos_number + 1),
          "old" => format!("{}", winner_erdos_number)
        );
        self.erdos_link.erdos_number = loser_erdos_number + 1;
        winner.erdos_links.push(self.erdos_link.clone());
        winner.put().unwrap();
        *self.users_cache.get_mut(&self.user_id).unwrap() = self.erdos_link.erdos_number;
      } else {
        increment_counter!("games_skipped", "reason" => "db_erdos");
      }
    }
  }
}

fn process_archive(url: &str) -> Result<()> {
  let mut curl_child = Command::new("curl")
    .arg(url)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()?;
  let curl_output = curl_child.stdout.take().context("No curl stdout")?;
  let mut pbzip_child = Command::new("pbunzip2")
    .stdin(curl_output)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()?;
  let pbzip_output = pbzip_child.stdout.take().context("No pbzip stdout")?;
  let mut pgn_read = pgn_reader::BufferedReader::new(pbzip_output);
  let mut game_parser = GameParser::new();
  pgn_read.read_all(&mut game_parser)?;
  ensure!(curl_child.wait()?.success(), "Curl failed");
  ensure!(pbzip_child.wait()?.success(), "Pbzip failed");
  Ok(())
}

pub async fn process_new_archives_task() -> Result<()> {
  loop {
    let last_archive = proto::Metadata::get()?.last_processed_archive;
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
      info!(%archive, "Processing archive");
      let archive_clone = archive.clone();
      spawn_blocking(move || process_archive(&archive_clone)).await??;
      info!(%archive, "Archive processed");
      let mut metadata = proto::Metadata::get()?;
      metadata.last_processed_archive = archive;
      metadata.put()?;
    }
    sleep(Duration::from_secs(60 * 60)).await;
  }
}
