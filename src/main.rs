use cli::CliArgs;
use git2::{Commit, Cred, DiffOptions, RemoteCallbacks, Repository};
use regex::Regex;
use std::{env, path::Path, sync::{Arc, Mutex}};
use serde::{Serialize, Deserialize};
use clap::Parser;
use log::{debug, info};

use url::Url;

use anyhow::{Context, Result};

use std::collections::hash_map::HashMap;

use actix_web::{http::header::ContentType, middleware, web::{self, Data, Json}, App, HttpRequest, HttpResponse, HttpServer};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_files::Files;

mod cli;

static SESSION_SIGNING_KEY: &[u8] = &[0; 64];
static LOG_ENV_VAR: &str = "RUST_LOG";

use i64 as Timestamp;

#[derive(Serialize, Deserialize, Debug)]
struct CalendarValueArr (Vec<CalendarValue>);

impl CalendarValueArr {
    fn new() -> Self {
        Self(vec!())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CalendarValue {
    pub title: String,
    pub delta_t: Timestamp,
    pub start: Timestamp,
    pub end: Timestamp,
    pub projected: bool,
}

#[derive(Debug, Default, Clone)]
struct PredictionAttributes {
    sum: (i32, Timestamp),
    count: i32,
    min: (i32, Timestamp),
    max: (i32, Timestamp),
}

impl PredictionAttributes {
    /// Makes Prediction based on its own values
    fn predict(&self, value: i32) -> Timestamp {

        let projection = value as Timestamp * (self.sum.1 / self.sum.0 as Timestamp);

        if projection < self.min.1 { return self.min.1; }
        if projection > self.max.1 { return self.max.1; }

        return projection;
    }
}

#[derive(Debug)]
struct PredictionStructure {
    history_map: HashMap<String, PredictionAttributes>,
    keys: Vec<String>,
}

impl PredictionStructure {
    fn new() -> Self {
        Self {
            history_map: HashMap::new(),
            keys: vec![
                "files_changed".to_string(),
                "lines_added".to_string(),
                "lines_removed".to_string()
            ],
        }
    }

    /// Function for adding another item to the PredictionStructure
    /// `key` refers to the key of the dictionary as well as what to insert
    /// an example would be "lines_added" or "files_changed"
    /// the value is the value of the attribute
    /// Returns `true` if a new value was added
    /// Returns `false` if a value was modified
    fn insert_item(&mut self, key: String, value: i32, time: Timestamp) -> bool {

        assert!(self.keys.contains(&key));

        let mut attributes = match self.history_map.get_mut(&key) {
            Some(v) => v.clone(),
            None => PredictionAttributes::default(),
        };

        // Updates sum and count
        attributes.count += 1;
        attributes.sum.0 += value;
        attributes.sum.1 += time;

        // Updates Min
        if value < attributes.min.0 {
            attributes.min.0 = value;
            attributes.min.1 = time;
        }

        // Updates Max
        if value > attributes.max.0 {
            attributes.max.0 = value;
            attributes.max.1 = time;
        }

        return match self.history_map.insert(key, attributes) {
            Some(_) => true,
            None => false,
        };
    }

    /// Makes a prediction based on all the previous values it found
    fn predict(&self, files_changed: i32, lines_added: i32, lines_removed: i32) -> Timestamp {

        let mut results = vec![];

        let keys_and_values = [
            ("files_changed".to_string(), files_changed),
            ("lines_added".to_string(), lines_added),
            ("lines_removed".to_string(), lines_removed),
        ];

        for (k, v) in keys_and_values {
            let pred_value = match self.history_map.get(&k) {
                Some(v) => v,
                None => continue,
            };

            results.push(pred_value.predict(v));
        }

        let response = results.iter().sum::<Timestamp>() / results.len() as Timestamp;
        return response;

    }
}

fn fetch_repo(ssh_url: &str, ssh_key_path: &str, tmp_dir: &Path) -> Result<Repository> {

    // Gets the home directory
    let home_env = match env::var("HOME") {
        Ok(v) => v,
        Err(_) => {
            panic!("Can't find environment variable `$HOME` for ssh!");
            // return Err(git2::Error::new(ErrorCode::Directory, ErrorClass::Os, "Can't find environment variable `$HOME` for ssh!".to_string()))
        },
    };

    // Sets Credential callback
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(username_from_url.unwrap(),
            None,
            Path::new(&home_env).join(ssh_key_path).as_path(),
            None,
        )
    });
    debug!("Generated Credentials");

    // Prepares fetch options
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);
    debug!("Prepared fetch options");

    // Prepare builder.
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);
    debug!("Finished preparing builder");

    // Clones the repo
    return Ok(match tmp_dir.is_dir() {
        true => Repository::open(tmp_dir).with_context(|| "Can't find the repo directory (do you have the right path?)")?,
        false => builder.clone(ssh_url, tmp_dir).with_context(|| format!("Can't clone repo to `{tmp_dir:?}` (do you have the right URL?)"))?,
    });
}

/// Gets the head commit from a repo
fn get_head_commit(repo: &Repository) -> Commit {

    let head_oid = repo
        .head()
        .unwrap()
        .target()
        .unwrap();

    let head_object = repo
        .find_commit(head_oid)
        .unwrap();

    return head_object;
}

/// Gets the path of a url from URI
/// May modify the url to work with git@xyz:your/project domains
/// Rewrites them to https://xyz/your/project
fn get_path(src_url: &str) -> Url {
    let mut tmp_url = src_url.to_string();

    let re = Regex::new(r"git@(?<domain>.+):(?<path>.+)").unwrap();

    if let Some(caps) = re.captures(&tmp_url) {
        tmp_url = format!("https://{}/{}", &caps["domain"], &caps["path"]);
    }

    debug!("Parsing URL: {tmp_url}");
    Url::parse(&tmp_url).unwrap()
}

async fn not_found(_req: HttpRequest) -> HttpResponse {
    let response = "<h1>404 Not FOUND!</h1>".to_string() +
        "<hr />" +
        "<p>Golly gee, the page you're looking for can't be found! Maybe try a different page or go back to the <a href='/'>homepage</a>?</p>";
    return HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(response);
}

#[derive(Clone)]
struct CommitData {
    message: String,
    timestamp: Timestamp,
    prev_timestamp: Timestamp,
    delta_t: Timestamp,
    files_changed: i32,
    lines_added: i32,
    lines_removed: i32,
    projected: bool,
}

async fn get_data(_req: HttpRequest, args: Data<Arc<Mutex<CliArgs>>>, repo: Data<Arc<Mutex<Repository>>>) -> Json<CalendarValueArr> {

    let unlocked_repo = repo.lock().unwrap();
    let mut commit_arr: Vec<CommitData> = Vec::new();

    let mut head = get_head_commit(&unlocked_repo);

    let session_time = {
        let unlocked_args = args.lock().unwrap();
        unlocked_args.time_allowed
    };

    let mut prediction = PredictionStructure::new();

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

        let commit_data = CommitData {
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
        .collect::<Vec<&[CommitData]>>()
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
        .collect::<Vec<Vec<CommitData>>>()
        ;

    let mut calendar_items = CalendarValueArr::new();

    // Converts the list of list of `CommitData`s into a single array of `CalendarValues`s
    for item_lst in output_arr {
        for value in item_lst {
            calendar_items.0.push(
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
    let url = &args.lock().unwrap().url;
    get_path(url)
        .path() // Gets the path
        .trim_start_matches("/") // Removes the starting / (if applicable)
        .splitn(2, ".") // Gets the part before the .
        .nth(0)
        .unwrap()
        .to_string()
}

/// Gets the URL of the repository
async fn repo_url(args: Data<Arc<Mutex<CliArgs>>>) -> String {
    let url = get_path(&args.lock().unwrap().url);
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
    if let Some(level) = unlocked_args.logs.to_level() {
        std::env::set_var(LOG_ENV_VAR, level.to_string());
    }

    let env = env_logger::Env::new().filter(LOG_ENV_VAR);
    env_logger::init_from_env(env);

    let src_url = &unlocked_args.url;

    let url = get_path(src_url);

    // Fetches repo
    let repo = Arc::new(Mutex::new(
        match url.scheme() {
            "http" | "https" | "ssh" => {
                let file_path = ".".to_string() + url.path();
                let repo = fetch_repo(
                    src_url,
                    &unlocked_args.ssh_key,
                    Path::new(&unlocked_args.tmp.to_string()).join(&file_path).as_path(),
                    ).unwrap();
                info!("Repo Cloned to `{file_path}`!");
                repo
            },
            "file" => {
                let directory = &unlocked_args.directory;
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
