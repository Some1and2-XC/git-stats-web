use cli::CliArgs;
use git2::{Commit, Cred, RemoteCallbacks, Repository};
use std::{env, path::Path, sync::{Arc, Mutex}};
use serde::{Serialize, Deserialize};
use clap::Parser;
use log::info;
use anyhow::{anyhow, Result};

use actix_web::{http::header::ContentType, middleware, web::{self, Data}, App, HttpRequest, HttpResponse, HttpServer};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_files::Files;

mod git;
mod cli;

static SESSION_SIGNING_KEY: &[u8] = &[0; 64];


#[derive(Serialize, Deserialize, Debug)]
struct OutputArr (Vec<OutputValue>);

#[derive(Serialize, Deserialize, Debug)]
struct OutputValue {
    pub title: String,
    pub delta_t: u32,
    pub start: String,
    pub end: String,
}

fn fetch_repo(ssh_url: &str, ssh_key_path: &str, tmp_dir: &Path) -> Repository {

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

    // Prepares fetch options
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    // Prepare builder.
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    // Clones the repo
    return match tmp_dir.is_dir() {
        true => Repository::open(tmp_dir).unwrap(),
        false => builder.clone(ssh_url, tmp_dir).unwrap(),
    };
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

async fn not_found(_req: HttpRequest) -> HttpResponse {
    let response = "<h1>404 Not FOUND!</h1>".to_string() +
        "<hr />" +
        "<p>Golly gee, the page you're looking for can't be found! Maybe try a different page or go back to the <a href='/'>homepage</a>?</p>";
    return HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(response);
}

async fn get_data(_req: HttpRequest, args_mutex: Data<Mutex<CliArgs>>, repo: Data<Mutex<Repository>>) -> anyhow::Result<OutputArr> {
    return Err(anyhow!("Can't do the smthn"));


}


#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Gets CLI arguments
    let args = cli::CliArgs::parse();

    // Initializes the logger
    if let Some(level) = args.logs.to_level() {
        std::env::set_var("RUST_LOG", level.to_string());
        env_logger::init();
    }

    // "git@github.com:some1and2-xc/git-stats/",

    // Fetches repo
    let repo = Arc::new(Mutex::new(
        match &args.url {
            Some(url) => fetch_repo(
                &url,
                &args.ssh_key,
                Path::new(&"./tmp/git-stats".to_string()),
                ),
            None => Repository::init(args.directory).unwrap(),
        }
    ));

    // let head = get_head_commit(&repo);

    let key = actix_web::cookie::Key::from(SESSION_SIGNING_KEY);

    let ip = "127.0.0.1";
    let port = &args.server_port;

    info!("Starting HTTP server at `{ip}:{port}`!");

    HttpServer::new(move || {
        let args_mutex = Data::new(Mutex::new(args));

        App::new()
            .app_data(repo_mutex.clone())
            .app_data(args_mutex.clone())
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
            .route(&args.server_uri, web::get().to(get_data))
            .default_service(web::to(not_found))
    })
        .bind((ip, port))?
        .run()
        .await.unwrap();

    return Ok(());


    // Some example code on how to get stuff using git2
    /*
    for i in 0..1000 {

        let mut diff_opts = DiffOptions::new();

        let Ok(parent) = head.parent(0) else {
            println!("Last IDX: {i}");
            break;
        };

        let res = repo.diff_tree_to_tree(
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

        println!("{}\t{:?} Timestamp: {} Committer: {}",
            i,
            res,
            head.time().seconds(),
            head.committer().email().unwrap_or("EMAIL_NOT_FOUND")
            );

        // println!("{}", head.message().unwrap());
        head = parent;
    }
    */

}
