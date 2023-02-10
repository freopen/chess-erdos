#[cfg(unix)]
pub mod db;

use malachite::Natural;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErdosLinkMeta {
    pub erdos_number: u32,
    pub link_count: u32,
    pub path_count: Natural,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub erdos_link_meta: Vec<ErdosLinkMeta>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ErdosLink {
    pub loser_id: String,
    pub time: chrono::DateTime<chrono::Utc>,
    pub loser_link_count: u32,
    pub loser_path_count: Natural,

    pub winner_info: PlayerInfo,
    pub loser_info: PlayerInfo,

    pub game_id: String,
    pub move_count: u32,
    pub time_control: TimeControl,
    pub winner_is_white: bool,
    pub termination: Termination,
}

#[derive(Serialize, Deserialize)]
pub struct ErdosChainLink {
    pub link: ErdosLink,
    pub link_number: u32,
    pub path_number: Natural,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub title: String,
    pub rating: u32,
    pub rating_change: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TimeControl {
    pub game_type: TimeControlType,
    pub main: u32,
    pub increment: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum TimeControlType {
    #[default]
    Blitz,
    Rapid,
    Classical,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum Termination {
    #[default]
    Checkmate,
    Resign,
    Time,
}
