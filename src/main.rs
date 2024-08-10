use auth::SESSION_USER_ID_KEY;
use cli::CliArgs;
use templates::WithBase;
use std::path::Path;
use serde::{Serialize, Deserialize};
use clap::Parser;
use log::{debug, info};

use url::Url;

use sqlx::{migrate::MigrateDatabase, query, Sqlite, SqlitePool};

use actix_web::{http::{header::ContentType, StatusCode}, middleware, web::{self, Data, Json}, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_files::Files;

use dotenv::dotenv;

use maud::html;

mod cli;
mod git;
mod utils;
mod db;
mod templates;
mod auth;
mod errors;

use git_stats_web::{prediction, aliases::*};

const DB_URL: &str = "sqlite://sqlite.db";

const SESSION_SIGNING_KEY: &[u8] = &[0; 64];
const LOG_ENV_VAR: &str = "RUST_LOG";

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
    }.template_base();
    return HttpResponse::NotFound()
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
async fn get_data(req: HttpRequest, args: Data<CliArgs>) -> Result<Json<Vec<CalendarValue>>, errors::AppError> {

    let src_url = match web::Query::<templates::calendar::RepoUrl>::from_query((&req).query_string()) {
        Ok(v) => v,
        Err(_) => {
            return Err(errors::AppError {
                cause: Some(format!("Invalid get request parameters! Query: `{}`", req.query_string())),
                message: Some("Invalid get request parameters!".to_string()),
                error_type: StatusCode::BAD_REQUEST,
            });
        },
    }.0.url;

    let url = match Url::parse(&src_url) {
        Ok(v) => v,
        Err(_) => {
            return Err(errors::AppError {
                cause: Some(format!("Failed to parse URL from: `{}`", src_url)),
                message: Some(format!("Failed to parse URL from: `{}`", src_url)),
                error_type: StatusCode::BAD_REQUEST,
            });
        },
    };


    // Fetches repo
    let repo = match url.scheme() {
        "http" | "https" | "ssh" => {
            let file_path = format!("{}{}", url.authority(), url.path()).to_lowercase();
            let repo = git::fetch_repo(
                &src_url,
                &args.ssh_key,
                Path::new(&args.tmp.to_string()).join(&file_path).as_path(),
                ).unwrap();
            info!("Repo Cloned to `{file_path}`!");
            repo
        },
        /*
        "file" => {
            let directory = ".".to_string() + url.path();
            info!("Found repo in directory: `{directory}`!");
            Repository::init(directory).unwrap()
        },
        */
        scheme => {
            return Err(errors::AppError {
                cause: Some(format!("Can't use scheme on URL: {} (source URL: {})", scheme, url)),
                message: Some(format!("Can't use scheme on URL: {}", scheme)),
                error_type: StatusCode::BAD_REQUEST,
            });
        }
    };

    return Ok(Json(utils::calculate_data(&*args, &repo)));
}

/// Gets the name of the repository with link
async fn repo_name(args: Data<CliArgs>) -> String {
    let url = utils::get_path(&args.url);

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
async fn repo_url(args: Data<CliArgs>) -> String {
    let url = utils::get_path(&args.url);

    if !url.has_host() {
        return "".to_string();
    }

    return url.to_string();
}

async fn get_id(session: Session, db: DbPool) -> impl Responder {
    return match auth::User::from_session(&session, db).await {
        Some(v) => format!("ID: #{:0>8}", v.id.unwrap()),
        None => "Not Signed In!".to_string(),
    };
}

async fn get_info(session: Session, db: DbPool) -> impl Responder {
    let user = match auth::User::from_session(&session, db).await {
        Some(v) => v,
        None => return "Not Logged In!".to_string(),
    };

    return format!("User: {:?}", user);
}

#[derive(Debug, Deserialize)]
pub struct GithubRequest {
    pub code: String,
}

async fn github_callback(info: web::Query<GithubRequest>) -> impl Responder {

    return html! {
        p {
            "Your key is: "
            code style="color: var(--link-color);" {
                (info.into_inner().code)
            }
        }

    }.template_base();
}

#[derive(Debug)]
pub struct GithubClient {
    client_id: String,
    client_secret: String,
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

    dotenv().ok();

    let github_client = Data::new(GithubClient {
        client_id: std::env::var("CLIENT_ID").unwrap().to_string(),
        client_secret: std::env::var("CLIENT_SECRET").unwrap().to_string(),
    });

    HttpServer::new(move || {

        App::new()

            // Sets global values
            .app_data(Data::clone(&args))
            .app_data(Data::clone(&db))
            .app_data(Data::clone(&github_client))

            // Sets middle wares
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Compress::default())
            .wrap(SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                .cookie_secure(false)
                .build(),
            )

            // Home page
            .service(web::resource("/").to(templates::home::home))

            // Auth
            .route("/login", web::get().to(templates::auth::login))
            .route("/login", web::post().to(auth::login_handler))

            .route("/sign-up", web::get().to(templates::auth::signup))
            .route("/sign-up", web::post().to(auth::signup_handler))

            .route("/logout", web::get().to(auth::logout))

            // Github Auth
            .route("/github/callback", web::get().to(github_callback))

            // Random attributes
            .route("/get-id", web::get().to(get_id))
            .route("/get-info", web::get().to(get_info))

            // Sets the calendar url
            .route("/repo", web::get().to(templates::calendar::calendar))

            // Sets the repo list url
            .route("/repos", web::get().to(templates::repo_list::repo_list))

            // Sets api endpoints
            .service(
                web::scope("/api")
                    .service(web::resource("/data").to(get_data))
                    .service(web::resource("/repo-name").to(repo_name))
                    .service(web::resource("/repo-url").to(repo_url))
                    // .service(web::resource("/repo/{site}/{username}/{repo}").to(get_data))
                    .route("/repo", web::get().to(get_data))
                )

            // Sets the static server
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
