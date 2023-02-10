use std::{str::FromStr, time::Duration};

use anyhow::{anyhow, Context};
use axum::{
    debug_handler,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use headers::{CacheControl, ContentType, HeaderMap, HeaderMapExt};
use include_dir::{include_dir, Dir};
use malachite::Natural;
use tracing::Level;

use crate::data::{db::DB, ErdosChainLink, User};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/generated/dist");

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", self.0)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

type Result<T> = std::result::Result<T, AppError>;

#[debug_handler]
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

#[debug_handler]
async fn user_handler(
    Path(id): Path<String>,
    Extension(db): Extension<DB>,
) -> Result<(HeaderMap, Json<Option<User>>)> {
    let mut headers = HeaderMap::new();
    headers.typed_insert(CacheControl::new().with_max_age(Duration::from_secs(60 * 60)));
    Ok((headers, Json(db.users.get(&id)?)))
}

#[debug_handler]
async fn chain_handler(
    Path((uid, erdos_number, path_number)): Path<(String, u32, String)>,
    Extension(db): Extension<DB>,
) -> Result<(HeaderMap, Json<Vec<ErdosChainLink>>)> {
    let user = db
        .users
        .get(&uid.to_ascii_lowercase())?
        .context("User not found")?;
    let meta = user
        .erdos_link_meta
        .iter()
        .find(|meta| meta.erdos_number == erdos_number)
        .context("Erdos number not found")?;
    let path_number = Natural::from_str(&path_number).or(Err(anyhow!("path_id didn't parse")))?;
    if path_number >= meta.path_count {
        return Err(anyhow!("path_number is too high").into());
    }

    let mut paths_left = path_number;
    let mut erdos_chain = vec![];
    let mut current_user = uid.to_ascii_lowercase();

    for current_erdos_number in (1..erdos_number + 1).rev() {
        let mut current_link = 0;
        erdos_chain.push(loop {
            dbg!(
                &current_user,
                &current_erdos_number,
                &current_link,
                &paths_left
            );
            let erdos_link = db
                .erdos_links
                .get(&(current_user.clone(), current_erdos_number, current_link))?
                .context("Intermediate chain not found")?;
            dbg!(&erdos_link);
            if erdos_link.loser_path_count <= paths_left {
                paths_left -= erdos_link.loser_path_count;
                current_link += 1;
            } else {
                current_user = erdos_link.loser_id.to_ascii_lowercase();
                break ErdosChainLink {
                    link: erdos_link,
                    link_number: current_link,
                    path_number: paths_left.clone(),
                };
            }
        })
    }

    let mut headers = HeaderMap::new();
    headers.typed_insert(CacheControl::new().with_max_age(Duration::from_secs(60 * 60)));
    Ok((headers, Json(erdos_chain)))
}

#[debug_handler]
async fn index_handler() -> (HeaderMap, &'static [u8]) {
    let mut header_map = HeaderMap::new();
    header_map.typed_insert(CacheControl::new().with_max_age(Duration::from_secs(60)));
    header_map.typed_insert(ContentType::html());
    (header_map, DIST.get_file("index.html").unwrap().contents())
}

#[debug_handler]
async fn last_processed_handler(Extension(db): Extension<DB>) -> (HeaderMap, String) {
    let mut header_map = HeaderMap::new();
    header_map.typed_insert(CacheControl::new().with_max_age(Duration::from_secs(60)));
    header_map.typed_insert(ContentType::text());
    let last_archive = db
        .last_processed_archive
        .get(&())
        .unwrap()
        .unwrap_or_default();
    let last_time = last_archive[(last_archive.len() - 15)..(last_archive.len() - 8)].to_string();
    (header_map, last_time)
}

pub async fn serve(db: &DB) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/api/user/:id", get(user_handler))
        .route(
            "/api/chain/:uid/:erdos_number/:path_number",
            get(chain_handler),
        )
        .route("/api/last_processed", get(last_processed_handler))
        .route("/assets/*path", get(static_handler))
        .fallback(index_handler)
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
