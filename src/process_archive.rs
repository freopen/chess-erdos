use std::convert::TryFrom;
use std::{collections::HashMap, time::Duration};

use anyhow::{bail, ensure, Context, Result};
use bzip2::read::MultiBzDecoder;
use chrono::NaiveDateTime;
use log::{debug, info};
use pgn_reader::{RawHeader, SanPlus, Skip, Visitor};
use prost_types::Timestamp;
use reqwest::blocking::get;
use shakmaty::san::Suffix;
use tokio::{task::spawn_blocking, time::sleep};

use crate::proto::{
    user_update::Update::NewErdosLink, ErdosLink, GameInfo, PlayerInfo, User, WinType,
};
use crate::{
    db::{DB, DB_USERS},
    proto::UserUpdate,
};

pub const ERDOS_INF: u32 = 1000000;
pub const ERDOS_ID: &str = "DrNykterstein";
pub const LICHESS_DB_LIST: &str = "https://database.lichess.org/standard/list.txt";

#[derive(Default)]
struct GameParser {
    erdos_link: ErdosLink,
    headers: HashMap<String, String>,
    skip: bool,
}

fn get_erdos_number(id: &String) -> u32 {
    if let Some(user) = DB_USERS.get(id.to_ascii_lowercase()).unwrap() {
        user.erdos_number
    } else {
        DB_USERS
            .put(
                id.to_ascii_lowercase(),
                &User {
                    id: id.clone(),
                    erdos_number: ERDOS_INF,
                    erdos_links: vec![],
                },
            )
            .unwrap();
        ERDOS_INF
    }
}

impl Visitor for GameParser {
    type Result = ();

    fn begin_game(&mut self) {
        *self = GameParser::default();
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        self.headers.insert(
            String::from_utf8(key.into()).unwrap(),
            value.decode_utf8().unwrap().to_string(),
        );
    }

    fn end_headers(&mut self) -> Skip {
        fn headers_to_erdos_link(headers: &mut HashMap<String, String>) -> Result<ErdosLink> {
            fn extract_player_info(
                color: &str,
                headers: &mut HashMap<String, String>,
            ) -> Result<(PlayerInfo, u32)> {
                let id = headers.remove(color).context("No id").unwrap();
                ensure!(id != "?", "Anonymous user, discarding");
                let erdos_number = get_erdos_number(&id);
                Ok((
                    PlayerInfo {
                        id,
                        title: headers
                            .remove(&format!("{}Title", color))
                            .unwrap_or_default(),
                        rating: headers
                            .remove(&format!("{}Elo", color))
                            .context("No Elo")
                            .unwrap()
                            .parse()
                            .unwrap(),
                        rating_diff: headers
                            .remove(&format!("{}RatingDiff", color))
                            .context("No RatingDiff, possible cheater")?
                            .parse()
                            .unwrap(),
                    },
                    erdos_number,
                ))
            }

            let event = headers.remove("Event").context("No Event")?;
            ensure!(
                event.starts_with("Rated Blitz")
                    || event.starts_with("Rated Rapid")
                    || event.starts_with("Rated Classical"),
                "Uninteresting event: {}",
                event
            );
            let (winner_color, loser_color) = match headers.remove("Result").as_deref() {
                Some("1-0") => ("White", "Black"),
                Some("0-1") => ("Black", "White"),
                _ => bail!("Uninteresting result"),
            };
            let (winner, winner_erdos_number) = extract_player_info(winner_color, headers)?;
            let (loser, loser_erdos_number) = extract_player_info(loser_color, headers)?;
            ensure!(
                winner_erdos_number > loser_erdos_number + 1,
                "Winner Erdos number is not improving"
            );

            let time = NaiveDateTime::parse_from_str(
                &format!(
                    "{} {}",
                    headers.remove("UTCDate").context("No UTCDate")?,
                    headers.remove("UTCTime").context("No UTCTime")?,
                ),
                "%Y.%m.%d %H:%M:%S",
            )
            .context("Timestamp parse failed")?;
            Ok(ErdosLink {
                erdos_number: loser_erdos_number + 1,
                time: Some(Timestamp {
                    seconds: time.timestamp(),
                    nanos: i32::try_from(time.timestamp_subsec_nanos())?,
                }),
                game_info: Some(GameInfo {
                    game_id: headers
                        .remove("Site")
                        .context("No Site")?
                        .strip_prefix("https://lichess.org/")
                        .context("Unexpected prefix")?
                        .to_string(),
                    winner: Some(winner),
                    loser: Some(loser),
                    time_control: headers.remove("TimeControl").context("No TimeControl")?,
                    moves: 0,
                    win_type: match headers
                        .remove("Termination")
                        .context("No Termination")?
                        .as_str()
                    {
                        "Normal" => WinType::Resign,
                        "Time forfeit" => WinType::Timeout,
                        term => bail!("Unexpected Termination: {}", term),
                    }
                    .into(),
                    winner_is_white: winner_color == "White",
                }),
            })
        }

        match headers_to_erdos_link(&mut self.headers) {
            Ok(erdos_link) => {
                self.erdos_link = erdos_link;
                Skip(false)
            }
            Err(err) => {
                debug!("PGN skipped: {:#?}", err);
                self.skip = true;
                Skip(true)
            }
        }
    }

    fn san(&mut self, san: SanPlus) {
        self.erdos_link.game_info.as_mut().unwrap().moves += 1;
        if san.suffix == Some(Suffix::Checkmate) {
            self.erdos_link
                .game_info
                .as_mut()
                .unwrap()
                .set_win_type(WinType::Mate);
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true)
    }

    fn end_game(&mut self) -> Self::Result {
        if !self.skip && self.erdos_link.game_info.as_ref().unwrap().moves >= 20 {
            DB_USERS
                .merge(
                    self.erdos_link
                        .game_info
                        .as_ref()
                        .unwrap()
                        .winner
                        .as_ref()
                        .unwrap()
                        .id
                        .to_ascii_lowercase(),
                    UserUpdate {
                        update: Some(NewErdosLink(self.erdos_link.clone())),
                    },
                )
                .unwrap();
        }
    }
}

fn process_archive(url: &str) -> Result<()> {
    let response = get(url)?;
    let uncompressed = MultiBzDecoder::new(response);
    let mut pgn_read = pgn_reader::BufferedReader::new(uncompressed);
    let mut game_parser = GameParser::default();
    pgn_read.read_all(&mut game_parser)?;
    Ok(())
}

fn process_new_archives() -> Result<()> {
    info!("Processing new archives");
    let last_archive = String::from_utf8(DB.get("last_processed_archive")?.unwrap_or_default())?;
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
        DB.put("last_processed_archive", &archive)?;
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
