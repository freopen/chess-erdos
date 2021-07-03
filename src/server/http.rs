use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};

use super::data::{build_schema, Db, SchemaType};

#[get("/")]
async fn hello() -> impl Responder {
  HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(playground_source(
      GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
    ))
}

async fn graphql(
  request: async_graphql_actix_web::Request,
  schema: web::Data<SchemaType>,
) -> async_graphql_actix_web::Response {
  schema.execute(request.into_inner()).await.into()
}

pub async fn http_server_task(db: Db) -> anyhow::Result<()> {
  HttpServer::new(move || {
    App::new()
      .wrap(Cors::permissive())
      .app_data(web::Data::new(build_schema(db.clone())))
      .service(hello)
      .service(web::resource("/graphql").to(graphql))
  })
  .bind("127.0.0.1:3000")?
  .run()
  .await?;
  Ok(())
}
