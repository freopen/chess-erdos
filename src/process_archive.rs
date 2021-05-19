use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use anyhow::{bail, ensure, Context, Result};
use bzip2::read::MultiBzDecoder;
use chrono::NaiveDateTime;
use log::{debug, info};
use pgn_reader::{RawHeader, SanPlus, Skip, Visitor};
use reqwest::blocking::get;
use shakmaty::san::Suffix;
use tokio::{task::spawn_blocking, time::sleep};

use crate::{
    db::{ENV, META, USERS},
    proto::{ErdosLink, Meta, User, WinType},
};

pub const ERDOS_NUMBER_INF: u32 = u32::MAX - 1;
pub const ERDOS_ID: &str = "Samsung777"; //"DrNykterstein";
pub const LICHESS_DB_LIST: &str = "https://database.lichess.org/standard/list.txt";

#[derive(Default)]
struct GameParser {
    user_id: String,
    erdos_link: ErdosLink,
    headers: HashMap<String, String>,
    skip: bool,
    users_cache: HashSet<String>,
}

fn get_erdos_number_at(id: &String, timestamp: i64) -> u32 {
    if id == ERDOS_ID {
        return 0;
    }
    let user = USERS
        .get(&mut ENV.read_txn().unwrap(), &id.to_ascii_lowercase())
        .unwrap()
        .unwrap();
    user.erdos_links
        .into_iter()
        .filter_map(|erdos_link| {
            if erdos_link.time <= timestamp {
                Some(erdos_link.erdos_number)
            } else {
                None
            }
        })
        .last()
        .unwrap_or(ERDOS_NUMBER_INF)
}

fn add_erdos_link(user_id: &str, erdos_link: ErdosLink) {
    let mut txn = ENV.write_txn().unwrap();
    let mut user = USERS
        .get(&mut txn, &user_id.to_ascii_lowercase())
        .unwrap()
        .unwrap();
    if let Some(last_erdos_link) = user.erdos_links.last() {
        assert!(
            last_erdos_link.erdos_number > erdos_link.erdos_number
                && last_erdos_link.time < erdos_link.time
        );
    }
    user.erdos_links.push(erdos_link);
    USERS
        .put(&mut txn, &user_id.to_ascii_lowercase(), &user)
        .unwrap();
    txn.commit().unwrap();
}

impl GameParser {
    fn new() -> Self {
        let mut users_cache = HashSet::new();
        users_cache.insert("?".to_string());
        users_cache.insert(ERDOS_ID.to_string());
        GameParser {
            users_cache,
            ..Default::default()
        }
    }
    fn add_user_to_db(&mut self, id: String) {
        if self.users_cache.contains(&id) {
            return;
        }
        let mut txn = ENV.write_txn().unwrap();
        if USERS
            .get(&mut txn, &id.to_ascii_lowercase())
            .unwrap()
            .is_none()
        {
            USERS
                .put(
                    &mut txn,
                    &id.to_ascii_lowercase(),
                    &User {
                        id: id.clone(),
                        erdos_links: vec![],
                    },
                )
                .unwrap();
        }
        txn.commit().unwrap();
        self.users_cache.insert(id);
    }
    fn parse_user_data(&mut self, for_white: bool) -> Result<(String, String, u32, i32)> {
        let id = self
            .headers
            .remove(if for_white { "White" } else { "Black" })
            .with_context(|| format!("No user id: {:?}", self.erdos_link))
            .unwrap();
        ensure!(id != "?", "Anonymous game");
        Ok((
            id,
            self.headers
                .remove(if for_white {
                    "WhiteTitle"
                } else {
                    "BlackTitle"
                })
                .unwrap_or_default(),
            self.headers
                .remove(if for_white { "WhiteElo" } else { "BlackElo" })
                .with_context(|| format!("No Elo: {:?}", self.erdos_link))
                .unwrap()
                .parse()
                .with_context(|| format!("Incorrect Elo format: {:?}", self.erdos_link))
                .unwrap(),
            self.headers
                .remove(if for_white {
                    "WhiteRatingDiff"
                } else {
                    "BlackRatingDiff"
                })
                .context("No rating diff, probably a cheater, skipping")?
                .parse()
                .with_context(|| format!("Incorrect RatingDiff format: {:?}", self.erdos_link))
                .unwrap(),
        ))
    }

    fn headers_to_erdos_link(&mut self) -> Result<()> {
        self.add_user_to_db(self.headers.get("White").unwrap().to_string());
        self.add_user_to_db(self.headers.get("Black").unwrap().to_string());
        let event = self
            .headers
            .remove("Event")
            .with_context(|| format!("No Event: {:?}", self.erdos_link))
            .unwrap();
        ensure!(
            event.starts_with("Rated Blitz")
                || event.starts_with("Rated Rapid")
                || event.starts_with("Rated Classical"),
            "Uninteresting event: {}",
            event,
        );
        self.erdos_link.winner_is_white = match self.headers.remove("Result").as_deref() {
            Some("1-0") => true,
            Some("0-1") => false,
            _ => {
                bail!("Uninteresting result");
            }
        };
        (
            self.user_id,
            self.erdos_link.title,
            self.erdos_link.rating,
            self.erdos_link.rating_diff,
        ) = self.parse_user_data(self.erdos_link.winner_is_white)?;
        (
            self.erdos_link.loser_id,
            self.erdos_link.loser_title,
            self.erdos_link.loser_rating,
            self.erdos_link.loser_rating_diff,
        ) = self.parse_user_data(!self.erdos_link.winner_is_white)?;
        self.erdos_link.time = NaiveDateTime::parse_from_str(
            &format!(
                "{} {}",
                self.headers.remove("UTCDate").context("No UTCDate")?,
                self.headers.remove("UTCTime").context("No UTCTime")?,
            ),
            "%Y.%m.%d %H:%M:%S",
        )
        .context("Timestamp parse failed")?
        .timestamp();
        self.erdos_link.erdos_number =
            get_erdos_number_at(&self.erdos_link.loser_id, self.erdos_link.time - 1) + 1;

        ensure!(
            get_erdos_number_at(&self.user_id, self.erdos_link.time) > self.erdos_link.erdos_number,
            "Winner Erdos number is not improving"
        );
        self.erdos_link.win_type = match self
            .headers
            .remove("Termination")
            .context("No Termination")
            .unwrap()
            .as_str()
        {
            "Normal" => WinType::Resign,
            "Time forfeit" => WinType::Timeout,
            term => bail!("Unexpected Termination: {}", term),
        }
        .into();

        self.erdos_link.game_id = self
            .headers
            .remove("Site")
            .with_context(|| format!("No Site: {:?}", self.headers))
            .unwrap()
            .strip_prefix("https://lichess.org/")
            .with_context(|| format!("Unexpected prefix: {:?}", self.headers))
            .unwrap()
            .to_string();
        self.erdos_link.time_control = self
            .headers
            .remove("TimeControl")
            .with_context(|| format!("No TimeControl: {:?}", self.erdos_link))
            .unwrap();
        self.erdos_link.moves_count = 0;
        Ok(())
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
        match self.headers_to_erdos_link() {
            Ok(()) => Skip(false),
            Err(err) => {
                debug!("PGN skipped: {:#?}", err);
                self.skip = true;
                Skip(true)
            }
        }
    }

    fn san(&mut self, san: SanPlus) {
        self.erdos_link.moves_count += 1;
        if san.suffix == Some(Suffix::Checkmate) {
            self.erdos_link.set_win_type(WinType::Mate);
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true)
    }

    fn end_game(&mut self) -> Self::Result {
        if !self.skip && self.erdos_link.moves_count >= 20 {
            add_erdos_link(&self.user_id, self.erdos_link.clone());
        }
    }
}

fn process_archive(url: &str) -> Result<()> {
    let response = get(url)?;
    let uncompressed = MultiBzDecoder::new(response);
    let mut pgn_read = pgn_reader::BufferedReader::new(uncompressed);
    let mut game_parser = GameParser::new();
    pgn_read.read_all(&mut game_parser)?;
    Ok(())
}

fn process_new_archives() -> Result<()> {
    info!("Processing new archives");
    let last_archive = {
        let mut txn = ENV.read_txn().unwrap();
        META.get(&mut txn, &())
            .unwrap()
            // .unwrap_or_default()
            .unwrap_or(Meta {
                last_processed_archive: "https://database.lichess.org/standard/lichess_db_standard_rated_2019-06.pgn.bz2".to_string(),
            })
            .last_processed_archive
    };
    let lichess_archives: Vec<String> = get(LICHESS_DB_LIST)?
        .text()?
        .split_ascii_whitespace()
        .rev()
        .map(|str| String::from(str))
        .skip_while(|archive| archive <= &last_archive)
        .collect();
    info!("New archives found: {}", lichess_archives.len());
    for archive in lichess_archives {
        info!("Processing archive url: {}", &archive);
        process_archive(&archive)?;
        {
            let mut txn = ENV.write_txn().unwrap();
            let mut meta = META.get(&mut txn, &()).unwrap().unwrap_or_default();
            meta.last_processed_archive = archive.clone();
            META.put(&mut txn, &(), &meta).unwrap();
            txn.commit().unwrap();
        }
        info!("Archive url processed: {}", &archive);
    }
    Ok(())
}

pub async fn process_new_archives_task() -> Result<()> {
    loop {
        spawn_blocking(process_new_archives).await??;
        sleep(Duration::from_secs(60 * 60)).await;
    }
}
