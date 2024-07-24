use cli::CliArgs;
use git2::Repository;
use std::{path::Path, sync::{Arc, Mutex}};
use serde::{Serialize, Deserialize};
use clap::Parser;
use log::{debug, info};

use actix_web::{guard, http::header::ContentType, middleware, web::{self, Data, Json}, App, HttpRequest, HttpResponse, HttpServer};
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

static SESSION_SIGNING_KEY: &[u8] = &[0; 64];
static LOG_ENV_VAR: &str = "RUST_LOG";

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

/*
#[derive(Debug)]
struct CantFindRepoError {
    pub message: String,
}

impl CantFindRepoError {
    fn new(message: String) -> Self {
        return Self {
            message,
        };
    }
}

impl ResponseError for CantFindRepoError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        return actix_web::http::StatusCode::from_u16(500).unwrap();
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        return HttpResponse::Ok().body("It seems as though a repo can't be constructed.");
    }
}

impl Display for CantFindRepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}

async fn get_project(path: web::Path<(String, String, String)>, args: Data<Arc<Mutex<CliArgs>>>) -> Result<Json<Vec<CalendarValue>>, CantFindRepoError> {
    let unlocked_args = args.lock().unwrap();

    let (site, username, repo) = path.into_inner();
    let file_path = Path::new(&unlocked_args.tmp)
        .join(&site)
        .join(&username)
        .join(&repo)
        ;

    // If the directory isn't found
    //
    let repo = match file_path.join(".git").is_dir() {
        false => {
            info!("Cloning repo: {repo}");
            match fetch_repo(
                &format!("https://{site}/{username}/{repo}"),
                &unlocked_args.ssh_key,
                file_path.as_path()
                ) {
                    Ok(v) => v,
                    Err(e) => return Err(CantFindRepoError::new(e.to_string())),
            }
        },
        true => Repository::open(file_path).unwrap(),
    };

    return Ok(Json(utils::calculate_data(&unlocked_args, &repo)));
}
*/

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Gets CLI arguments
    let args = Arc::new(Mutex::new({
        let mut args = cli::CliArgs::parse();
        let terminator = ".git";
        if args.url.ends_with(terminator) {
            args.url = args.url[0..args.url.len() - terminator.len()]
                .to_lowercase()
                .to_string();
        }
        args
    }));

    let unlocked_args = args.lock().unwrap();

    // Initializes the logger
    if let Some(level) = unlocked_args.log.to_level() {
        std::env::set_var(LOG_ENV_VAR, level.to_string());
    }

    let env = env_logger::Env::new().filter(LOG_ENV_VAR);
    env_logger::init_from_env(env);

    let src_url = &unlocked_args.url;

    let url = utils::get_path(src_url);

    // Fetches repo
    let repo = Arc::new(Mutex::new(
        match url.scheme() {
            "http" | "https" | "ssh" => {
                let file_path = format!("{}{}", url.authority(), url.path()).to_lowercase();
                let repo = git::fetch_repo(
                    src_url,
                    &unlocked_args.ssh_key,
                    Path::new(&unlocked_args.tmp.to_string()).join(&file_path).as_path(),
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
    ));

    drop(unlocked_args); // releases the mutex

    debug!("Initialized repo!");

    // let head = get_head_commit(&repo);

    let key = actix_web::cookie::Key::from(SESSION_SIGNING_KEY);

    let ip = "127.0.0.1";
    let port = args.lock().unwrap().server_port;

    info!("Starting HTTP server at `{ip}:{port}`!");

    HttpServer::new(move || {

        App::new()
            .app_data(Data::new(repo.clone()))
            .app_data(Data::new(args.clone()))
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

            .service(web::resource("/sign-up").to(templates::auth::signup))
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
