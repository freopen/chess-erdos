use chrono::{DateTime, Utc};

use crate::data::User;

pub const ERDOS_NUMBER_INF: u32 = u32::MAX - 1;
pub const ERDOS_ID: &str = "mamalak";

pub fn user_to_erdos_number(user: &User) -> u32 {
  if user.id == ERDOS_ID {
    0
  } else {
    user
      .erdos_links
      .last()
      .map(|link| link.erdos_number)
      .unwrap_or(ERDOS_NUMBER_INF)
  }
}

pub fn user_to_erdos_number_at(user: &User, time: DateTime<Utc>) -> u32 {
  if user.id == ERDOS_ID {
    0
  } else {
    user
      .erdos_links
      .iter()
      .filter(|erdos_link| erdos_link.time < time)
      .last()
      .map(|erdos_link| erdos_link.erdos_number)
      .unwrap_or(ERDOS_NUMBER_INF)
  }
}
