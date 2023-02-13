use actix_cors::Cors;
use actix_web::{
    middleware::{Logger, NormalizePath, TrailingSlash},
    web, App, HttpResponse, HttpServer,
};
use futures::future;
use tonic::transport::Channel;

mod auth {
    tonic::include_proto!("auth_session");
}

mod db;
mod guard;
mod sync;

use auth::auth_session_client::AuthSessionClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let listen_port = std::env::var("LISTEN_PORT").expect("LISTEN_PORT is required");

    let auth_session_uri = std::env::var("AUTH_SESSION_URI").expect("AUTH_SESSION_URI not set");
    let auth_session_client: AuthSessionClient<Channel> =
        AuthSessionClient::connect(auth_session_uri.clone())
            .await
            .unwrap();

    let master_db_cluster = std::env::var("MASTER_DB_CLUSTER").expect("MASTER_DB_CLUSTER not set");
    // std::env::var("MASTER_DB_NAME").expect("MASTER_DB_NAME not set");

    db::initialize(&master_db_cluster).await;
    // println!("db initialized");
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .send_wildcard()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin()
                    .max_age(3600),
            )
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::QueryConfig::default().error_handler(|err, _res| {
                actix_web::error::InternalError::from_response(
                    err,
                    HttpResponse::BadRequest().finish(),
                )
                .into()
            }))
            .app_data(web::QueryConfig::default().error_handler(|err, _res| {
                actix_web::error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(
                            r#"{{"msg": "{}", "code": "DATA INPUT ERROR"}}"#,
                            err
                        )),
                )
                .into()
            }))
            .app_data(web::JsonConfig::default().error_handler(|err, _res| {
                actix_web::error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(
                            r#"{{"msg": "{}", "code": "DATA INPUT ERROR"}}"#,
                            err
                        )),
                )
                .into()
            }))
            .app_data(web::Data::new(auth_session_client.clone()))
            .configure(|cfg| sync::init_routes("", cfg))
    })
    .bind(format!("0.0.0.0:{}", listen_port))?
    .run();
    let print_message = async {
        println!("Server started successfully on port {}", listen_port);
    };
    let _ = future::join(server, print_message).await;
    Ok(())
}
