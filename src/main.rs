mod server;

#[actix_web::main]
async fn main() {
  env_logger::init();
  if let Err(error) = server::serve().await {
    panic!("{:#?}", error);
  }
}
