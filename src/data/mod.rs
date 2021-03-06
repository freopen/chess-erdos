use serde::{Deserialize, Serialize};

#[cfg(unix)]
mod collections;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub erdos_links: Vec<ErdosLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErdosLink {
    pub erdos_number: u32,
    pub loser_id: String,
    pub time: chrono::DateTime<chrono::Utc>,

    pub winner_info: PlayerInfo,
    pub loser_info: PlayerInfo,

    pub game_id: String,
    pub move_count: u32,
    pub time_control: TimeControl,
    pub winner_is_white: bool,
    pub termination: Termination,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub title: String,
    pub rating: u32,
    pub rating_change: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeControl {
    pub game_type: TimeControlType,
    pub main: u32,
    pub increment: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeControlType {
    Blitz,
    Rapid,
    Classical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Termination {
    Checkmate,
    Resign,
    Time,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErdosChains {
    pub id: String,
    pub erdos_chains: Vec<Vec<ErdosLink>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerMetadata {
    pub last_processed_archive: String,
}
