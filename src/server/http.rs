use std::time::Duration;

use anyhow::{Context, Result};
use axum::{extract::Path, http::StatusCode, routing::get, Extension, Router};
use headers::{CacheControl, ContentType, HeaderMap, HeaderMapExt};
use include_dir::{include_dir, Dir};
use rkyvdb::{Collection, Database};
use tracing::Level;

use crate::data::{ErdosChains, ErdosLink, ServerMetadata, User};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/generated/dist");

async fn static_handler(Path(path): Path<String>) -> (StatusCode, HeaderMap, &'static [u8]) {
    if let Some(file) = DIST.get_file(format!("assets/{path}")) {
        let mut header_map = HeaderMap::new();
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

#[tracing::instrument(skip_all, fields(erdos_number = %erdos_link.erdos_number))]
fn expand_erdos_chain(erdos_link: ErdosLink, db: &Database) -> Result<Vec<ErdosLink>> {
    let mut erdos_links = vec![erdos_link];
    for erdos_number in (1..erdos_links[0].erdos_number).rev() {
        let next_user =
            User::get(erdos_links.last().unwrap().loser_id.as_str(), db)?.context("Broken chain in DB")?;
        let next_erdos_link = next_user
            .erdos_links
            .into_iter()
            .find(|erdos_link| erdos_link.erdos_number == erdos_number)
            .context("Broken chain in DB")?;
        erdos_links.push(next_erdos_link);
    }
    Ok(erdos_links)
}

#[tracing::instrument(skip_all, fields(user = %user.id))]
fn build_erdos_chains(user: User, db: &Database) -> Result<ErdosChains> {
    Ok(ErdosChains {
        id: user.id.to_string(),
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
            rmp_serde::encode::to_vec(&build_erdos_chains(user, &db).unwrap()).unwrap(),
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

async fn last_processed_handler(Extension(db): Extension<Database>) -> (HeaderMap, String) {
    let mut header_map = HeaderMap::new();
    header_map.typed_insert(CacheControl::new().with_max_age(Duration::from_secs(60)));
    header_map.typed_insert(ContentType::text());
    let last_archive = ServerMetadata::get((), &db)
        .unwrap()
        .map(|x| x.last_processed_archive)
        .unwrap_or_default();
    let last_time = last_archive[(last_archive.len() - 15)..(last_archive.len() - 8)].to_string();
    (header_map, last_time)
}

pub async fn serve(db: &Database) -> Result<()> {
    let app = Router::new()
        .route("/api/erdos_chains/:id", get(erdos_chains_handler))
        .route("/api/last_processed", get(last_processed_handler))
        .route("/assets/*path", get(static_handler))
        .fallback(get(index_handler))
        .layer(Extension(db.clone()))
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_request(tower_http::trace::DefaultOnRequest::new().level(Level::INFO))
                .on_response(tower_http::trace::DefaultOnResponse::new().level(Level::INFO)),
        );
    axum::Server::bind(&([127, 0, 0, 1], 3001).into())
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
