mod server;
mod data;
mod util;

#[tokio::main]
async fn main() {
    server::serve().await.unwrap();
}
