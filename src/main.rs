mod server;

#[tokio::main]
async fn main() {
  env_logger::init();
  server::serve().await.unwrap();
}
