use cli::CliArgs;
use git2::Repository;
use std::{path::Path, sync::{Arc, Mutex}};
use serde::{Serialize, Deserialize};
use clap::Parser;
use log::{debug, info};

use sqlx::{migrate::MigrateDatabase, query, Sqlite, SqlitePool};

use actix_web::{http::header::ContentType, middleware, web::{self, Data, Json}, App, HttpRequest, HttpResponse, HttpServer};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_files::Files;

use maud::html;

mod cli;
mod prediction;
mod git;
mod utils;
mod db;
mod templates;
mod auth;

const DB_URL: &str = "sqlite://sqlite.db";

const SESSION_SIGNING_KEY: &[u8] = &[0; 64];
const LOG_ENV_VAR: &str = "RUST_LOG";

use i64 as Timestamp;

async fn not_found(_req: HttpRequest) -> HttpResponse {
    let response = html! {
        h1 { "404 Not FOUND!" }
        hr;
        p {
            "Golly gee, the page you're looking for can't be found! Maybe try a different page or go back to the "
            a href="/" {
                "homepage"
            }
            "?"
        }
    };
    return HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(response.into_string());
}

#[derive(Serialize, Deserialize, Debug)]
struct CalendarValue {
    pub title: String,
    pub delta_t: Timestamp,
    pub start: Timestamp,
    pub end: Timestamp,
    pub projected: bool,
}

/// Function for getting commit data and returning json
async fn get_data(_req: HttpRequest, args: Data<Arc<Mutex<CliArgs>>>, repo: Data<Arc<Mutex<Repository>>>) -> Json<Vec<CalendarValue>> {

    let unlocked_repo = repo.lock().unwrap();
    let unlocked_args = args.lock().unwrap();

    return Json(utils::calculate_data(&unlocked_args, &unlocked_repo));
}

/// Gets the name of the repository with link
async fn repo_name(args: Data<Arc<Mutex<CliArgs>>>) -> String {
    let url = utils::get_path(&args.lock().unwrap().url);

    if url.scheme() == "file" {
        return "LOCAL REPO".to_string();
    }

    url.path() // Gets the path
        .trim_start_matches("/") // Removes the starting / (if applicable)
        .splitn(2, ".") // Gets the part before the .
        .nth(0)
        .unwrap()
        .to_string()
}

/// Gets the URL of the repository
async fn repo_url(args: Data<Arc<Mutex<CliArgs>>>) -> String {
    let url = utils::get_path(&args.lock().unwrap().url);

    if !url.has_host() {
        return "".to_string();
    }

    return url.to_string();
}

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Gets CLI arguments
    let args = Data::new({
        let mut args = cli::CliArgs::parse();
        let terminator = ".git";
        if args.url.ends_with(terminator) {
            args.url = args.url[0..args.url.len() - terminator.len()]
                .to_lowercase()
                .to_string();
        }
        args
    });

    // Initializes the logger
    if let Some(level) = args.log.to_level() {
        std::env::set_var(LOG_ENV_VAR, level.to_string());
    }

    let env = env_logger::Env::new().filter(LOG_ENV_VAR);
    env_logger::init_from_env(env);

    let src_url = &args.url;

    let url = utils::get_path(src_url);

    // Fetches repo
    let repo = Mutex::new(
        match url.scheme() {
            "http" | "https" | "ssh" => {
                let file_path = format!("{}{}", url.authority(), url.path()).to_lowercase();
                let repo = git::fetch_repo(
                    src_url,
                    &args.ssh_key,
                    Path::new(&args.tmp.to_string()).join(&file_path).as_path(),
                    ).unwrap();
                info!("Repo Cloned to `{file_path}`!");
                repo
            },
            "file" => {
                let directory = ".".to_string() + url.path();
                info!("Found repo in directory: `{directory}`!");
                Repository::init(directory).unwrap()
            },
            _ => {
                panic!("Unknown Format!");
            }
        }
    );

    debug!("Initializing Database!");

    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        info!("Creating Database: {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => info!("Created DB Successfully!"),
            Err(e) => panic!("Error: {}", e),
        }
    } else {
        info!("Database Already Exists!");
    }

    let db = Data::new(SqlitePool::connect(DB_URL).await.unwrap());
    let db_schema_filename = "schema.sql";
    let db_schema = match std::fs::read_to_string(db_schema_filename) {
        Ok(v) => v,
        Err(_) => {
            info!("Can't schema file: `{}`", db_schema_filename);
            String::new()
        },
    };

    let _result = query(&db_schema).execute(&**db).await.unwrap();

    debug!("Initialized repo!");

    // let head = get_head_commit(&repo);

    let key = actix_web::cookie::Key::from(SESSION_SIGNING_KEY);

    let ip = "127.0.0.1";
    let port = args.server_port;

    info!("Starting HTTP server at `{ip}:{port}`!");

    HttpServer::new(move || {

        App::new()
            .app_data(Data::clone(&args))
            .app_data(Data::clone(&db))
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Compress::default())
            .wrap(SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                .cookie_secure(false)
                .build(),
            )

            .service(web::resource("/").to(templates::home::home))

            .route("/login", web::get().to(templates::auth::login))
            .route("/login", web::post().to(auth::login_handler))

            .route("/sign-up", web::get().to(templates::auth::signup))
            .route("/sign-up", web::post().to(auth::signup_handler))

            .service(web::resource("/repo/{site}/{username}/{repo}").to(templates::calendar::calendar))

            .service(
                web::scope("/api")
                    .service(web::resource("/data").to(get_data))
                    .service(web::resource("/repo-name").to(repo_name))
                    .service(web::resource("/repo-url").to(repo_url))
                    .service(web::resource("/repo/{site}/{username}/{repo}").to(get_data))
                )

            .service(Files::new("/static", "static")
                // .index_file("index.html")
                // .show_files_listing() // Tree shows static files
                // .prefer_utf8(true)
                )

            .default_service(web::to(not_found))
    })
        .bind((ip, port))?
        .run()
        .await.unwrap();

    return Ok(());
}
