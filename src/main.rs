mod db;

use std::collections::HashMap;
use std::convert::TryFrom;

use anyhow::{bail, ensure, Context, Result};
use bzip2::read::MultiBzDecoder;
use chrono::NaiveDateTime;
use log::{debug, info};
use pgn_reader::{RawHeader, SanPlus, Skip, Visitor};
use prost_types::Timestamp;
use reqwest::blocking::get;
use shakmaty::san::Suffix;

mod proto {
    tonic::include_proto!("chess_erdos");
}

#[derive(Default)]
struct GameParser {
    erdos_link: proto::ErdosLink,
    headers: HashMap<String, String>,
    skip: bool,
}

impl Visitor for GameParser {
    type Result = ();

    fn begin_game(&mut self) {
        *self = GameParser::default();
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        self.headers.insert(
            String::from_utf8_lossy(key).to_string(),
            value.decode_utf8_lossy().to_string(),
        );
    }

    fn end_headers(&mut self) -> Skip {
        fn headers_to_erdos_link(
            headers: &mut HashMap<String, String>,
        ) -> Result<proto::ErdosLink> {
            fn extract_player_info(
                color: &str,
                headers: &mut HashMap<String, String>,
            ) -> Result<proto::PlayerInfo> {
                Ok(proto::PlayerInfo {
                    id: headers.remove(color).context("No id")?,
                    title: headers
                        .remove(&format!("{}Title", color))
                        .unwrap_or_default(),
                    rating: headers
                        .remove(&format!("{}Elo", color))
                        .context("No Elo")?
                        .parse()?,
                    rating_diff: headers
                        .remove(&format!("{}RatingDiff", color))
                        .context("No RatingDiff")?
                        .parse()?,
                })
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
            let time = NaiveDateTime::parse_from_str(
                &format!(
                    "{} {}",
                    headers.remove("UTCDate").context("No UTCDate")?,
                    headers.remove("UTCTime").context("No UTCTime")?,
                ),
                "%Y.%m.%d %H:%M:%S",
            )
            .context("Timestamp parse failed")?;
            Ok(proto::ErdosLink {
                erdos_number: 0,
                time: Some(Timestamp {
                    seconds: time.timestamp(),
                    nanos: i32::try_from(time.timestamp_subsec_nanos())?,
                }),
                game_info: Some(proto::GameInfo {
                    game_id: headers
                        .remove("Site")
                        .context("No Site")?
                        .strip_prefix("https://lichess.org/")
                        .context("Unexpected prefix")?
                        .to_string(),
                    winner: Some(extract_player_info(winner_color, headers)?),
                    loser: Some(extract_player_info(loser_color, headers)?),
                    time_control: headers.remove("TimeControl").context("No TimeControl")?,
                    moves: 0,
                    win_type: match headers
                        .remove("Termination")
                        .context("No Termination")?
                        .as_str()
                    {
                        "Normal" => proto::WinType::Resign,
                        "Time forfeit" => proto::WinType::Timeout,
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
                .set_win_type(proto::WinType::Mate);
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true)
    }

    fn end_game(&mut self) -> Self::Result {
        if !self.skip && self.erdos_link.game_info.as_ref().unwrap().moves >= 20 {
            
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

fn main() {
    env_logger::init();
    process_archive(
        "https://database.lichess.org/standard/lichess_db_standard_rated_2014-07.pgn.bz2",
    )
    .unwrap();
}
