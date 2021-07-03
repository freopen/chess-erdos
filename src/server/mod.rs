mod data;
mod http;
mod process_archive;

use anyhow::Result;
use tokio::select;

pub async fn serve() -> Result<()> {
  let db = data::Db::new()?;
  select! {
    res = process_archive::process_new_archives_task(db.clone()) => res?,
    res = http::http_server_task(db.clone()) => res?
  }
  Ok(())
}
