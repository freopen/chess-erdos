use std::{net::SocketAddr, str::FromStr, time::Duration};

use anyhow::{Context, Result};
use axum::{extract::Path, http::StatusCode, routing::get, Extension, Router};
use bonsaidb::{core::schema::SerializedCollection, local::Database};
use headers::{CacheControl, ContentType, HeaderMap, HeaderMapExt};
use include_dir::{include_dir, Dir};

use crate::data::{ErdosChains, ErdosLink, User};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/generated/dist");

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

fn expand_erdos_chain(erdos_link: ErdosLink, db: &Database) -> Result<Vec<ErdosLink>> {
    let mut erdos_links = vec![erdos_link];
    for erdos_number in (1..erdos_links[0].erdos_number).rev() {
        let next_user = User::get(&erdos_links.last().unwrap().loser_id, db)?
            .context("Broken chain in DB")?
            .contents;
        let next_erdos_link = next_user
            .erdos_links
            .into_iter()
            .find(|erdos_link| erdos_link.erdos_number == erdos_number)
            .context("Broken chain in DB")?;
        erdos_links.push(next_erdos_link);
    }
    Ok(erdos_links)
}

fn build_erdos_chains(user: User, db: &Database) -> Result<ErdosChains> {
    Ok(ErdosChains {
        id: user.id.clone(),
        erdos_chains: user
            .erdos_links
            .into_iter()
            .map(|x| expand_erdos_chain(x, db))
            .rev()
            .collect::<Result<Vec<_>>>()?,
    })
}

async fn erdos_chains_handler(
    Path(id): Path<String>,
    Extension(db): Extension<Database>,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();
    headers.typed_insert(ContentType::octet_stream());
    headers.typed_insert(CacheControl::new().with_max_age(Duration::from_secs(60 * 60)));
    if let Some(user) = User::get(&id, &db).unwrap() {
        (
            StatusCode::OK,
            headers,
            pot::to_vec(&build_erdos_chains(user.contents, &db).unwrap()).unwrap(),
        )
    } else {
        (StatusCode::NOT_FOUND, headers, vec![])
    }
}

async fn index_handler() -> (HeaderMap, &'static [u8]) {
    let mut header_map = HeaderMap::new();
    header_map.typed_insert(CacheControl::new().with_max_age(Duration::from_secs(10)));
    header_map.typed_insert(ContentType::html());
    (header_map, DIST.get_file("index.html").unwrap().contents())
}

pub async fn serve(db: &Database) -> Result<()> {
    let app = Router::new()
        .route("/assets/*path", get(static_handler))
        .route("/api/erdos_chains/:id", get(erdos_chains_handler))
        .fallback(get(index_handler))
        .layer(Extension(db.clone()));
    let addr = SocketAddr::from_str(&std::env::var("HTTP_ADDRESS")?)?;
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
