use git2::Repository;
use templates::WithBase;
use std::{env, path::Path};
use serde::Deserialize;
use clap::Parser;
use log::{debug, info, warn};

use url::Url;

use sqlx::{migrate::MigrateDatabase, query, Sqlite, SqlitePool};

use actix_web::{http::{header::{ContentType, WARNING}, StatusCode}, middleware, web::{self, Data, Json}, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_files::Files;

use dotenv::dotenv;

use maud::html;

mod templates;
mod auth;

mod ws;

use git_stats_web::{
    aliases::*, calendar::CalendarValue, cli::{self, CliArgs}, database::User, errors, git, utils
};

/// The URL to the SQLite database.
const DB_URL: &str = "sqlite://sqlite.db";

/// The session signing key, defaults to an array of 0s. This sets how cookies get encrypted.
const SESSION_SIGNING_KEY: &[u8] = &[0; 64];
/// The environment variable for setting the rust logging level.
const LOG_ENV_VAR: &str = "RUST_LOG";

/// Method for handling 404 pages.
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

    let arc_args = args.into_inner();

    // Fetches repo
    let repo = match url.scheme() {
        "http" | "https" | "ssh" => {
            let file_path = format!("{}{}", url.authority(), url.path()).to_lowercase();
            let repo = git::fetch_repo(
                &src_url,
                Path::new(&arc_args.get_tmp_path())
                    .join(&file_path).as_path(),
                arc_args.clone(),
                ).unwrap();

            info!("Repo Cloned to `{file_path}`!");
            repo
        },
        "file" => {
            if !(&arc_args).allow_local {
                return Err(errors::AppError {
                    cause: Some(format!("`file://` schema is allowed but not configured! (must be enabled by CLI argument)")),
                    message: Some(format!("`file://` schema is allowed but not configured! (must be enabled by CLI argument)")),
                    error_type: StatusCode::BAD_REQUEST,
                });
            }

            let directory = ".".to_string() + url.path();

            match Repository::init(&directory) {
                Ok(v) => {
                    info!("Found repo in directory: `{directory}`!");
                    v
                }, Err(e) => {
                    return Err(errors::AppError {
                        cause: Some(format!("`file://` schema is configured but file not found! Error: {}", e)),
                        message: Some(format!("`file://` schema is configured but file not found! Error: {}", e)),
                        error_type: StatusCode::NOT_FOUND,
                    })
                },
            }
        },
        scheme => {
            return Err(errors::AppError {
                cause: Some(format!("Can't use scheme on URL: {} (source URL: {})", scheme, url)),
                message: Some(format!("Can't use scheme on URL: {}", scheme)),
                error_type: StatusCode::BAD_REQUEST,
            });
        }
    };

    return Ok(Json(utils::calculate_data(arc_args, &repo)));
}

/*
async fn get_id(session: Session, db: DbPool) -> impl Responder {
    return match auth::User::from_session(&session, db).await {
        Some(v) => format!("ID: #{:0>8}", v.id.unwrap()),
        None => "Not Signed In!".to_string(),
    };
}
*/

async fn get_info(session: Session, db: DbPool) -> impl Responder {
    let user = match User::from_session(&session, &**db).await {
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
    let args = Data::new(cli::CliArgs::parse().set_project_location());

    // Initializes the logger
    if let Some(level) = &args.log.to_level() {
        std::env::set_var(LOG_ENV_VAR, level.to_string());
    }

    let env = env_logger::Env::new().filter(LOG_ENV_VAR);
    env_logger::init_from_env(env);

    // Shows warning if the exec_location is just the current directory
    if args.get_project_location_as_ref() == &Path::new(".").to_path_buf() {
        warn!("Can't find exe path, defaulting to run location for relative files! (this may include web pages and tmp files)");
    } else {
        info!("Found project location at: `{}`", args.get_project_location_as_ref().to_string_lossy());
    }

    info!("Found tmp directory at: `{}`", args.get_tmp_path());

    // This is expecting the current executable to be in target/release/some_exe
    let static_path = match &args.static_dir {
        // Just returns the static_dir value if set
        Some(v) => v.clone(),
        // Joins the path to the executable location and casts to string
        None => args.get_project_location_as_ref().join("static").to_str().unwrap().to_string(),
    };

    info!("Setting Web Directory to `{}`", static_path);

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

    let ip = "0.0.0.0";
    let port = (&args).server_port;

    info!("Starting HTTP server at `{ip}:{port}`!");

    dotenv().ok();

    let github_client = Data::new(GithubClient {
        client_id: std::env::var("CLIENT_ID").unwrap_or("".to_string()),
        client_secret: std::env::var("CLIENT_SECRET").unwrap_or("".to_string()),
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
            /*
            .route("/get-id", web::get().to(get_id))
            .route("/get-info", web::get().to(get_info))
            */

            // Sets the calendar url
            .route("/repo", web::get().to(templates::calendar::calendar))

            // Sets the repo list url
            .route("/repos", web::get().to(templates::repo_list::repo_list))

            // Sets an echo route.
            // WS is a feature for the future
            // .route("/echo", web::get().to(ws::echo))

            // Sets api endpoints
            .service(
                web::scope("/api")
                    .route("/repo", web::get().to(get_data))
                    // .service(web::resource("/repo/{site}/{username}/{repo}").to(get_data))
                )

            // Sets the static server
            .service(Files::new("/static", static_path.as_str())
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
