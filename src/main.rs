#[macro_use]
extern crate diesel;

use actix::*;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, http, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};

mod db;
mod models;
mod routes;
mod schema;
mod server;
mod session;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = server::ChatServer::new().start();

    // load TLS keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    let conn_spec = "chat.db";
    let manager = ConnectionManager::<SqliteConnection>::new(conn_spec);
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    let server_addr = "0.0.0.0";
    let server_port = 8080;

    let app = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://192.168.112.95:3000")
            .allowed_origin("https://192.168.112.95:8080")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(server.clone()))
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .service(web::resource("/").to(routes::index))
            .route("/ws", web::get().to(routes::chat_server))
            .service(routes::create_user)
            .service(routes::get_user_by_id)
            .service(routes::get_user_by_phone)
            .service(routes::get_conversation_by_id)
            .service(routes::get_rooms)
            .service(Files::new("/", "./static"))
    })
    .workers(2)
    .bind_openssl((server_addr, server_port), builder)?
    .run();

    println!("Server running at https://{server_addr}:{server_port}/");
    app.await
}
