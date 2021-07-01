mod db;
mod http;
mod process_archive;

use anyhow::Result;
use tokio::spawn;

pub async fn serve() -> Result<()> {
  // let process_new_archives_task = spawn(process_archive::process_new_archives_task());
  http::http_server_task().await?;
  // process_new_archives_task.abort();
  Ok(())
}
