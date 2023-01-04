mod data;
mod server;
mod util;

#[tokio::main]
async fn main() {
    server::serve().await.unwrap();
}
