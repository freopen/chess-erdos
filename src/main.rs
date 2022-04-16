mod data;
mod util;
mod server;

#[tokio::main]
async fn main() {
    server::serve().await.unwrap();
}

