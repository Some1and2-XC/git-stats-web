
use std::{fs, io::Read};

use actix_web::{
    get, http::header::ContentType, middleware, web::{self, get, resource, Json}, App, Error, HttpRequest, HttpResponse, HttpServer, Resource, Responder, Result};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_files::{Files, NamedFile};
use log::{debug, info};
use serde::{Deserialize, Serialize};

static SESSION_SIGNING_KEY: &[u8] = &[0; 64];

async fn not_found(_req: HttpRequest) -> Result<HttpResponse> {
    let response = "<h1>404 Not FOUND!</h1>".to_string() +
        "<hr />" +
        "<p>Golly gee, the page you're looking for can't be found! Maybe try a different page or go back to the <a href='/'>homepage</a>?</p>";
    return Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(response));
}

// #[actix_web::main]
async fn run() -> std::io::Result<()> {

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("INFO"));
    let key = actix_web::cookie::Key::from(SESSION_SIGNING_KEY);

    let ip = "127.0.0.1";
    let port = 3000;

    info!("Starting HTTP server at `{ip}:{port}`!");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                .cookie_secure(false)
                .build(),
            )
            .wrap(middleware::Logger::default())
            .service(Files::new("/static", "static")
                // .show_files_listing() // Tree shows static files
                // .index_file("index.html")
                // .prefer_utf8(true)
                )
            .default_service(web::to(not_found))
    })
        .bind((ip, port))?
        .run()
        .await
}
