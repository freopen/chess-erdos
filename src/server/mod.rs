mod db;
mod process_archive;

use anyhow::Result;

pub async fn serve() -> Result<()> {
  process_archive::process_new_archives_task().await
}
