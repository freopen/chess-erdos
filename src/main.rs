use anyhow::Result;
use bzip2::read::MultiBzDecoder;
use log::info;
use pgn_reader::{Outcome, Visitor};
use reqwest::blocking::get;

mod proto {
    tonic::include_proto!("chess_erdos");
}

struct GameParser {}

impl Visitor for GameParser {
    type Result = ();

    fn outcome(&mut self, outcome: Option<Outcome>) {
        info!("{:?}", outcome);
    }

    fn end_game(&mut self) -> Self::Result {
        ()
    }
}

fn process_archive(url: &str) -> Result<()> {
    let response = get(url)?;
    let uncompressed = MultiBzDecoder::new(response);
    let mut pgn_read = pgn_reader::BufferedReader::new(uncompressed);
    let mut game_parser = GameParser {};
    pgn_read.read_all(&mut game_parser)?;
    Ok(())
}

fn main() {
    env_logger::init();
    process_archive(
        "https://database.lichess.org/standard/lichess_db_standard_rated_2013-01.pgn.bz2",
    )
    .unwrap();
}
