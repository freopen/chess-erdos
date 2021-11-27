use crate::proto::User;

pub const ERDOS_NUMBER_INF: i32 = i32::MAX - 1;
pub const ERDOS_ID: &str = "DrNykterstein";

pub fn user_to_erdos_number(user: &User) -> i32 {
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

pub fn user_to_erdos_number_at(user: &User, time: i64) -> i32 {
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
