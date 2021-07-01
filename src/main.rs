mod server;

#[actix_web::main]
async fn main() {
  env_logger::init();
  server::serve().await.unwrap();
}
