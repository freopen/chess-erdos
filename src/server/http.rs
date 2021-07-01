use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use super::db::ErdosLink;

#[get("/")]
async fn hello() -> impl Responder {
  HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(playground_source(
      GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
    ))
}

struct Query;

#[Object]
impl Query {
  async fn hello(&self, username: String) -> String {
    format!("Hello, {}", username)
  }
}

async fn graphql(
  request: async_graphql_actix_web::Request,
  schema: web::Data<Schema<Query, EmptyMutation, EmptySubscription>>,
) -> async_graphql_actix_web::Response {
  schema.execute(request.into_inner()).await.into()
}

pub async fn http_server_task() -> anyhow::Result<()> {
  let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();
  HttpServer::new(move || {
    App::new()
      .app_data(web::Data::new(schema.clone()))
      .service(hello)
      .service(web::resource("/graphql").to(graphql))
  })
  .bind("127.0.0.1:3000")?
  .run()
  .await?;
  Ok(())
}
