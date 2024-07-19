use cli::CliArgs;
use git2::{DiffOptions, Repository};
use std::{path::Path, sync::{Arc, Mutex}};
use serde::{Serialize, Deserialize};
use clap::Parser;
use log::{debug, info};

use actix_web::{http::header::ContentType, middleware, web::{self, Data, Json}, App, HttpRequest, HttpResponse, HttpServer};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_files::Files;

mod cli;
mod prediction;
mod git;
mod utils;

static SESSION_SIGNING_KEY: &[u8] = &[0; 64];
static LOG_ENV_VAR: &str = "RUST_LOG";

use i64 as Timestamp;

async fn not_found(_req: HttpRequest) -> HttpResponse {
    let response = "<h1>404 Not FOUND!</h1>".to_string() +
        "<hr />" +
        "<p>Golly gee, the page you're looking for can't be found! Maybe try a different page or go back to the <a href='/'>homepage</a>?</p>";
    return HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(response);
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
    let mut commit_arr: Vec<git::CommitData> = Vec::new();

    let mut head = git::get_head_commit(&unlocked_repo);

    let session_time = {
        let unlocked_args = args.lock().unwrap();
        unlocked_args.time_allowed
    };

    let mut prediction = prediction::PredictionStructure::new();

    // Adds data the commit_arr
    // let max_commit_depth = 25;
    // for _i in 0..max_commit_depth {
    loop {

        let mut diff_opts = DiffOptions::new();

        let Ok(parent) = head.parent(0) else {
            break;
        };

        let diff = unlocked_repo.diff_tree_to_tree(
            Some(&parent.tree().unwrap()),
            Some(&head.tree().unwrap()),
            Some(
                diff_opts
                    .force_text(true)
                )
            )
            .unwrap()
            .stats()
            .unwrap();

        let timestamp = head.time().seconds();
        let prev_timestamp = parent.time().seconds();
        let delta_t = timestamp - prev_timestamp;

        let commit_data = git::CommitData {
            message: head.message().unwrap_or("MESSAGE_NOT_FOUND").trim().to_string(),
            timestamp,
            prev_timestamp,
            delta_t,
            files_changed: diff.files_changed() as i32,
            lines_added: diff.insertions() as i32,
            lines_removed: diff.deletions() as i32,
            projected: false,
        };

        commit_arr.push(commit_data);

        if delta_t < session_time {
            prediction.insert_item("files_changed".to_string(), diff.files_changed() as i32, delta_t);
            prediction.insert_item("lines_added".to_string(), diff.insertions() as i32, delta_t);
            prediction.insert_item("lines_removed".to_string(), diff.deletions() as i32, delta_t);
        }

        head = parent;
    }

    let output_arr = commit_arr
        .split_inclusive(|v| session_time <= v.delta_t)
        .collect::<Vec<&[git::CommitData]>>()
        .iter_mut()
        .map(|v| {
            let mut items = v.to_vec();
            let item = items.last_mut().unwrap();

            // Makes prediction for last item
            let prediction = prediction.predict(
                item.files_changed,
                item.lines_added,
                item.lines_removed);

            // Updates item with projections
            item.delta_t = prediction;
            item.prev_timestamp = item.timestamp - prediction;
            item.projected = true;

            items
        })
        .collect::<Vec<Vec<git::CommitData>>>()
        ;

    // let mut calendar_items = CalendarValueArr::new();
    let mut calendar_items = Vec::new();

    // Converts the list of list of `CommitData`s into a single array of `CalendarValues`s
    for item_lst in output_arr {
        for value in item_lst {
            calendar_items.push(
                CalendarValue {
                    title: value.message.clone(),
                    delta_t: value.delta_t,
                    start: value.prev_timestamp + 1, // Adding one so the timestamp don't overlap
                    end: value.timestamp,
                    projected: value.projected,
                }
            );
        }
    }

    Json(calendar_items)
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
    let args = Arc::new(Mutex::new({
        let mut args = cli::CliArgs::parse();
        let terminator = ".git";
        if args.url.ends_with(terminator) {
            args.url = args.url[0..args.url.len() - terminator.len()].to_string();
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
                let file_path = ".".to_string() + url.path();
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
            .service(web::resource("/api/data").to(get_data))
            .service(web::resource("/api/repo-name").to(repo_name))
            .service(web::resource("/api/repo-url").to(repo_url))
            .service(Files::new("/", "static")
                .index_file("index.html")
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
