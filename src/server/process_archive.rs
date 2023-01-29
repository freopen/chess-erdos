use std::{
    cmp::Ordering,
    process::{Command, Stdio},
};
use std::{collections::HashMap, time::Duration};

use anyhow::{ensure, Context, Result};
use chrono::{NaiveDate, NaiveTime, Utc};
use malachite::Natural;
use metrics::increment_counter;
use pgn_reader::{RawHeader, SanPlus, Skip, Visitor};
use reqwest::get;
use shakmaty::san::Suffix;
use tokio::{task::spawn_blocking, time::sleep};
use tracing::info;

use crate::{
    data::{db::DB, ErdosLink, ErdosLinkMeta, PlayerInfo, Termination, TimeControlType, User},
    util::ERDOS_ID,
};

const LICHESS_DB_LIST: &str = "https://database.lichess.org/standard/list.txt";
pub const ERDOS_NUMBER_INF: u32 = u32::MAX - 1;

#[derive(Clone, Default)]
struct ColorInfo {
    id: String,
    erdos_number: u32,
    player_info: PlayerInfo,
}

struct GameParser<'a> {
    db: &'a DB,
    fast_forward: Option<String>,
    erdos_link: ErdosLink,
    skip: bool,
    fields_bitset: u32,
    date: NaiveDate,
    time: chrono::NaiveTime,
    white: ColorInfo,
    black: ColorInfo,
    user_id: String,
    users_cache: HashMap<String, u32>,
}

impl<'a> GameParser<'a> {
    fn new(db: &'a DB) -> Self {
        let mut users_cache = HashMap::new();
        users_cache.insert("?".to_string(), ERDOS_NUMBER_INF);
        users_cache.insert(ERDOS_ID.to_string(), 0);
        GameParser {
            db,
            fast_forward: db.game_checkpoint.get(&()).unwrap(),
            erdos_link: ErdosLink::default(),
            skip: false,
            fields_bitset: 0,
            date: NaiveDate::default(),
            time: NaiveTime::default(),
            white: ColorInfo::default(),
            black: ColorInfo::default(),
            user_id: String::new(),
            users_cache,
        }
    }
    fn get_latest_erdos_number(&mut self, id: &str) -> Result<u32> {
        if let Some(erdos_number) = self.users_cache.get(id) {
            Ok(*erdos_number)
        } else if let Some(user) = self.db.users.get(&id.to_ascii_lowercase())? {
            let erdos_number = user
                .erdos_link_meta
                .get(0)
                .map_or(ERDOS_NUMBER_INF, |meta| meta.erdos_number);
            self.users_cache.insert(id.to_string(), erdos_number);
            Ok(erdos_number)
        } else {
            self.db.users.put(
                &id.to_ascii_lowercase(),
                &User {
                    id: id.to_string(),
                    erdos_link_meta: vec![],
                },
            )?;
            self.users_cache.insert(id.to_string(), ERDOS_NUMBER_INF);
            Ok(ERDOS_NUMBER_INF)
        }
    }
}

impl<'a> Visitor for GameParser<'a> {
    type Result = ();

    fn begin_game(&mut self) {
        self.skip = false;
        self.fields_bitset = 0;
        self.white.player_info.title = "".to_string();
        self.black.player_info.title = "".to_string();
        self.erdos_link.move_count = 0;
        self.erdos_link.game_id = "".to_string();
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        if self.skip {
            return;
        }
        match key {
            b"Event" => {
                assert!(self.fields_bitset & 1 << 0 == 0);
                self.fields_bitset |= 1 << 0;
                let event = value.decode();
                let without_rated = if let Some(without_rated) = event.strip_prefix(b"Rated ") {
                    without_rated
                } else {
                    increment_counter!("games_skipped", "reason" => "unrated");
                    self.skip = true;
                    return;
                };
                if without_rated.starts_with(b"Blitz ") {
                    self.erdos_link.time_control.game_type = TimeControlType::Blitz;
                } else if without_rated.starts_with(b"Rapid ") {
                    self.erdos_link.time_control.game_type = TimeControlType::Rapid;
                } else if without_rated.starts_with(b"Classical ") {
                    self.erdos_link.time_control.game_type = TimeControlType::Classical;
                } else {
                    increment_counter!(
                        "games_skipped",
                        "reason" => format!(
                            "timecontrol: {}",
                            std::str::from_utf8(without_rated)
                                .unwrap()
                                .split_ascii_whitespace()
                                .take(2)
                                .collect::<Vec<_>>()
                                .join(" ")
                        )
                    );
                    self.skip = true;
                }
            }
            b"Site" => {
                assert!(self.fields_bitset & 1 << 1 == 0);
                self.fields_bitset |= 1 << 1;
                self.erdos_link.game_id = value
                    .decode_utf8()
                    .unwrap()
                    .strip_prefix("https://lichess.org/")
                    .unwrap()
                    .to_string();
                if let Some(ff) = &self.fast_forward {
                    if ff == &self.erdos_link.game_id {
                        self.fast_forward = None;
                    } else {
                        increment_counter!("games_skipped", "reason" => "Fast forwarding after recovery");
                        self.skip = true;
                    }
                }
            }
            b"White" => {
                assert!(self.fields_bitset & 1 << 2 == 0);
                self.fields_bitset |= 1 << 2;
                let id = value.decode_utf8().unwrap().to_string();
                if id == "?" {
                    increment_counter!("games_skipped", "reason" => "unregistered: white");
                    self.skip = true;
                } else {
                    self.white.erdos_number = self.get_latest_erdos_number(&id).unwrap();
                    self.white.id = id;
                }
            }
            b"WhiteTitle" => {
                assert!(self.fields_bitset & 1 << 3 == 0);
                self.fields_bitset |= 1 << 3;
                self.white.player_info.title = value.decode_utf8().unwrap().to_string();
            }
            b"WhiteElo" => {
                assert!(self.fields_bitset & 1 << 4 == 0);
                self.fields_bitset |= 1 << 4;
                let rating_str = value.decode_utf8().unwrap();
                if rating_str == "?" {
                    increment_counter!("games_skipped", "reason" => "unregistered: white no elo");
                    self.skip = true;
                    return;
                }
                self.white.player_info.rating = rating_str.parse().unwrap();
            }
            b"WhiteRatingDiff" => {
                assert!(self.fields_bitset & 1 << 5 == 0);
                self.fields_bitset |= 1 << 5;
                self.white.player_info.rating_change =
                    value.decode_utf8().unwrap().parse().unwrap();
            }
            b"Black" => {
                assert!(self.fields_bitset & 1 << 6 == 0);
                self.fields_bitset |= 1 << 6;
                let id = value.decode_utf8().unwrap().to_string();
                if id == "?" {
                    increment_counter!("games_skipped", "reason" => "unregistered: black");
                    self.skip = true;
                } else {
                    self.black.erdos_number = self.get_latest_erdos_number(&id).unwrap();
                    self.black.id = id;
                    assert!(self.fields_bitset & 1 << 2 != 0);
                    if self.white.erdos_number == self.black.erdos_number {
                        increment_counter!("games_skipped", "reason" => "erdos: fast");
                        self.skip = true;
                    }
                }
            }
            b"BlackTitle" => {
                assert!(self.fields_bitset & 1 << 7 == 0);
                self.fields_bitset |= 1 << 7;
                self.black.player_info.title = value.decode_utf8().unwrap().to_string();
            }
            b"BlackElo" => {
                assert!(self.fields_bitset & 1 << 8 == 0);
                self.fields_bitset |= 1 << 8;
                let rating_str = value.decode_utf8().unwrap();
                if rating_str == "?" {
                    increment_counter!("games_skipped", "reason" => "unregistered: black no elo");
                    self.skip = true;
                    return;
                }
                self.black.player_info.rating = rating_str.parse().unwrap();
            }
            b"BlackRatingDiff" => {
                assert!(self.fields_bitset & 1 << 9 == 0);
                self.fields_bitset |= 1 << 9;
                self.black.player_info.rating_change =
                    value.decode_utf8().unwrap().parse().unwrap();
            }
            b"Result" => {
                assert!(self.fields_bitset & 1 << 10 == 0);
                self.fields_bitset |= 1 << 10;
                match value.decode().as_ref() {
                    b"1-0" => {
                        self.erdos_link.winner_is_white = true;
                    }
                    b"0-1" => {
                        self.erdos_link.winner_is_white = false;
                    }
                    b"1/2-1/2" => {
                        increment_counter!("games_skipped", "reason" => "result: draw");
                        self.skip = true;
                    }
                    unknown_result => {
                        increment_counter!("games_skipped", "reason" => format!("result: {}", std::str::from_utf8(unknown_result).unwrap()));
                        self.skip = true;
                    }
                }
            }
            b"UTCDate" => {
                assert!(self.fields_bitset & 1 << 11 == 0);
                self.fields_bitset |= 1 << 11;
                let date_str = value.decode_utf8().unwrap();
                self.date = NaiveDate::parse_from_str(&date_str, "%Y.%m.%d")
                    .unwrap_or_else(|_| panic!("Failed to parse date: {date_str}"));
            }
            b"UTCTime" => {
                assert!(self.fields_bitset & 1 << 12 == 0);
                self.fields_bitset |= 1 << 12;
                let time_str = value.decode_utf8().unwrap();
                self.time = chrono::NaiveTime::parse_from_str(&time_str, "%H:%M:%S")
                    .unwrap_or_else(|_| panic!("Failed to parse time: {time_str}"));
            }
            b"TimeControl" => {
                assert!(self.fields_bitset & 1 << 13 == 0);
                self.fields_bitset |= 1 << 13;
                let time_control = value.decode_utf8().unwrap();
                let (main_str, increment_str) = time_control
                    .split_once('+')
                    .expect("Unexpected TimeControl format");
                self.erdos_link.time_control.main = main_str.parse().unwrap();
                self.erdos_link.time_control.increment = increment_str.parse().unwrap();
            }
            b"Termination" => {
                assert!(self.fields_bitset & 1 << 14 == 0);
                self.fields_bitset |= 1 << 14;
                match value.decode().as_ref() {
                    b"Normal" => {
                        self.erdos_link.termination = Termination::Resign;
                    }
                    b"Time forfeit" => {
                        self.erdos_link.termination = Termination::Time;
                    }
                    unknown_termination => {
                        increment_counter!("games_skipped", "reason" => format!("termination: {}", std::str::from_utf8(unknown_termination).unwrap()));
                        self.skip = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn end_headers(&mut self) -> Skip {
        if !self.skip {
            self.fields_bitset |= 1 << 3 | 1 << 7;
            if self.fields_bitset != (1 << 15) - 1 {
                assert!(
                    self.fields_bitset | 1 << 5 | 1 << 9 == (1 << 15) - 1,
                    "{}",
                    self.erdos_link.game_id
                );
                increment_counter!("games_skipped", "reason" => "cheater: missing rating diff");
                self.skip = true;
            }
            let (winner, loser) = if self.erdos_link.winner_is_white {
                (self.white.clone(), self.black.clone())
            } else {
                (self.black.clone(), self.white.clone())
            };
            let winner_erdos = self.get_latest_erdos_number(&winner.id).unwrap();
            let loser_erdos = self.get_latest_erdos_number(&loser.id).unwrap();
            if winner_erdos < loser_erdos + 1 {
                increment_counter!("games_skipped", "reason" => "erdos: slow");
                self.skip = true;
                return Skip(true);
            }
            self.user_id = winner.id;
            self.erdos_link.loser_id = loser.id;
            self.erdos_link.winner_info = winner.player_info;
            self.erdos_link.loser_info = loser.player_info;
        }
        Skip(self.skip)
    }

    fn san(&mut self, san: SanPlus) {
        self.erdos_link.move_count += 1;
        if san.suffix == Some(Suffix::Checkmate) {
            self.erdos_link.termination = Termination::Checkmate;
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
            self.erdos_link.time =
                chrono::DateTime::from_utc(chrono::NaiveDateTime::new(self.date, self.time), Utc);
            let mut winner = self
                .db
                .users
                .get(&self.user_id.to_ascii_lowercase())
                .unwrap()
                .expect("User should be in DB at this point");
            let loser = self
                .db
                .users
                .get(&self.erdos_link.loser_id.to_ascii_lowercase())
                .unwrap()
                .expect("User should be in DB at this point");
            self.erdos_link.loser_link_count = loser.erdos_link_meta[0].link_count;
            self.erdos_link.loser_path_count = loser.erdos_link_meta[0].path_count.clone();
            let winner_erdos = self.get_latest_erdos_number(&winner.id).unwrap();
            let loser_erdos = self.get_latest_erdos_number(&loser.id).unwrap();
            match winner_erdos.cmp(&(loser_erdos + 1)) {
                Ordering::Greater => {
                    increment_counter!(
                      "erdos_updated",
                      "new" => format!("{}", loser_erdos + 1),
                      "old" => format!("{winner_erdos}")
                    );
                    winner.erdos_link_meta.insert(
                        0,
                        ErdosLinkMeta {
                            erdos_number: loser_erdos + 1,
                            link_count: 1,
                            path_count: loser.erdos_link_meta[0].path_count.clone(),
                        },
                    );
                    self.db
                        .erdos_links
                        .put(
                            &(winner.id.to_ascii_lowercase(), loser_erdos + 1, 0),
                            &self.erdos_link,
                        )
                        .unwrap();
                    self.db
                        .users
                        .put(&winner.id.to_ascii_lowercase(), &winner)
                        .unwrap();
                    self.db
                        .game_checkpoint
                        .put(&(), &self.erdos_link.game_id)
                        .unwrap();
                    // dbg!("erdos_updated", &self.erdos_link, &winner);
                    *self.users_cache.get_mut(&self.user_id).unwrap() =
                        winner.erdos_link_meta[0].erdos_number;
                }
                Ordering::Equal => {
                    increment_counter!(
                      "erdos_link_added",
                      "number" => format!("{winner_erdos}"),
                    );
                    winner.erdos_link_meta[0].link_count += 1;
                    winner.erdos_link_meta[0].path_count += &self.erdos_link.loser_path_count;
                    self.db
                        .erdos_links
                        .put(
                            &(
                                winner.id.to_ascii_lowercase(),
                                winner.erdos_link_meta[0].erdos_number,
                                winner.erdos_link_meta[0].link_count,
                            ),
                            &self.erdos_link,
                        )
                        .unwrap();
                    self.db
                        .users
                        .put(&winner.id.to_ascii_lowercase(), &winner)
                        .unwrap();
                    self.db
                        .game_checkpoint
                        .put(&(), &self.erdos_link.game_id)
                        .unwrap();
                    dbg!("erdos_link_added", &self.erdos_link, &winner);
                }
                Ordering::Less => {
                    panic!("Should have skipped earlier");
                }
            }
        }
    }
}

fn process_archive(db: &DB, url: &str) -> Result<()> {
    let mut curl_child = Command::new("curl")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let curl_output = curl_child.stdout.take().context("No curl stdout")?;
    let mut pbzip_child = Command::new("pzstd")
        .arg("-d")
        .arg("-c")
        .stdin(curl_output)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let pbzip_output = pbzip_child.stdout.take().context("No pbzip stdout")?;
    let mut pgn_read = pgn_reader::BufferedReader::new(pbzip_output);
    let mut game_parser = GameParser::new(db);
    pgn_read.read_all(&mut game_parser)?;
    ensure!(curl_child.wait()?.success(), "Curl failed");
    ensure!(pbzip_child.wait()?.success(), "Pbzip failed");
    Ok(())
}

pub async fn process_new_archives_task(db: &DB) -> Result<()> {
    loop {
        let last_archive = db
            .last_processed_archive
            .get(&())?
            .unwrap_or_else(|| String::from(""));
        if last_archive.is_empty() {
            db.users.put(
                &ERDOS_ID.to_ascii_lowercase(),
                &User {
                    id: ERDOS_ID.to_owned(),
                    erdos_link_meta: vec![ErdosLinkMeta {
                        erdos_number: 0,
                        link_count: 0,
                        path_count: Natural::from(1u32),
                    }],
                },
            )?;
        }
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
            {
                let db = db.clone();
                let archive = archive.clone();
                spawn_blocking(move || process_archive(&db, &archive)).await??;
            }
            info!(%archive, "Archive processed");
            db.game_checkpoint.delete(&())?;
            db.last_processed_archive.put(&(), &archive)?;
        }
        sleep(Duration::from_secs(60 * 60)).await;
    }
}
