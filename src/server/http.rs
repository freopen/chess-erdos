use std::{net::SocketAddr, str::FromStr, time::Duration};

use anyhow::Result;
use axum::{extract::Path, http::StatusCode, routing::get, Router};
use headers::{CacheControl, ContentType, HeaderMap, HeaderMapExt};
use include_dir::{include_dir, Dir};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/dist");

async fn static_handler(Path(path): Path<String>) -> (StatusCode, HeaderMap, &'static [u8]) {
    if let Some(file) = DIST.get_file(format!("assets/{path}")) {
        let mut header_map = HeaderMap::new();
        dbg!(file.path());
        let mime_type = mime_guess::from_path(file.path()).first_or_octet_stream();
        header_map.typed_insert(
            CacheControl::new().with_max_age(Duration::from_secs(365 * 24 * 60 * 60)),
        );
        header_map.typed_insert(ContentType::from(mime_type));
        (StatusCode::OK, header_map, file.contents())
    } else {
        (StatusCode::NOT_FOUND, HeaderMap::new(), &[])
    }
}

async fn index_handler() -> (HeaderMap, &'static [u8]) {
    let mut header_map = HeaderMap::new();
    header_map.typed_insert(CacheControl::new().with_max_age(Duration::from_secs(10)));
    header_map.typed_insert(ContentType::html());
    (header_map, DIST.get_file("index.html").unwrap().contents())
}

pub async fn serve() -> Result<()> {
    let app = Router::new()
        .route("/assets/*path", get(static_handler))
        .fallback(get(index_handler));
    let addr = SocketAddr::from_str(&std::env::var("HTTP_ADDRESS")?)?;
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
