use anyhow::Result;

pub async fn serve() -> Result<()> {
    tokio::time::sleep(std::time::Duration::from_secs(1000000000)).await;
    Ok(())
}
