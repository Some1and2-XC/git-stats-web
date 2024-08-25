use actix_web::web::Data;
use git2::{Commit, FetchOptions, Progress, RemoteCallbacks, Repository};
use std::{env, path::Path, sync::Arc};
use anyhow::{Context, Result};
use log::debug;

use super::cli::CliArgs;

fn print_callback(v: Progress) -> bool {
    debug!("Received Objects: #{:?}", v.received_objects());
    true
}

/// Function for cloning a repo
/// Returns error if it can't clone the repo.
pub fn fetch_repo(ssh_url: &str, out_dir: &Path, args: Arc<CliArgs>) -> Result<Repository> {

    // Gets the home directory
    /*
    let home_env = match env::var("HOME") {
        Ok(v) => v,
        Err(_) => {
            panic!("Can't find environment variable `$HOME` for ssh!");
            // return Err(git2::Error::new(ErrorCode::Directory, ErrorClass::Os, "Can't find environment variable `$HOME` for ssh!".to_string()))
        },
    };
    */

    // Sets Credential callback
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(print_callback);
    /*
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(username_from_url.unwrap(),
            None,
            Path::new(&home_env).join(ssh_key_path).as_path(),
            None,
        )
    });
    */

    let repo = match out_dir.is_dir() {
        true => Repository::open(out_dir).with_context(|| "Can't find the repo directory (do you have the right path?)")?,
        false => {

            // Prepares fetch options
            let mut fo = git2::FetchOptions::new();
            fo.remote_callbacks(callbacks);
            fo.depth(args.clone_depth as i32); // Sets max depth to the repo clone

            // Prepare builder.
            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fo);
            builder.bare(true);

            debug!("Cloning for the first time...");

            let git_out_dir = out_dir.join(".git/");
            debug!("Cloning into: {git_out_dir:?}");
            let _ = std::fs::create_dir_all(&git_out_dir).with_context(|| "Can't create directory!")?;
            let repo = Repository::init_bare(git_out_dir).with_context(|| "Can't initialize repo")?;

            // let repo = builder.clone(ssh_url, &git_out_dir).with_context(|| format!("Can't clone repo to `{out_dir:?}` (do you have the right URL?)"))?;

            // Sets some config rules
            let mut config = repo.config()?;

            // Makes repo not bare
            config.set_bool("core.bare", false)?;

            // Sets partial clone rules
            /*
            config.set_str( "remote.origin.partialclonefilter", "blob:limit=1")?;
            config.set_bool("remote.origin.promisor", true)?;
            */

            // Sets remotes
            let _remote = match repo.remote("origin", ssh_url) {
                Ok(v) => {
                    debug!("Added remote: {}", v.name().unwrap());
                    ()
                },
                Err(_e) => {
                    debug!("Couldn't add remote!");
                    ()
                }
            };

            // Reports configuration
            let mut entries = config.entries(None)?;

            while let Some(entry) = entries.next() {
                let res = entry.unwrap();
                debug!("Config Entry: {} = {}", res.name().unwrap(), res.value().unwrap());
            }

            repo
        }
    };

    // Updates the repo
    debug!("Updating Index...");

    let refs = repo.references_glob("refs/remotes/*/*").unwrap();
    for refname in refs {
        let oid = repo.refname_to_id(refname.unwrap().name().unwrap()).unwrap();
        let object = repo.find_object(oid, None).unwrap();
        repo.reset(&object, git2::ResetType::Hard, None).unwrap();
    }
    debug!("Updating Refs...");

    let remotes = repo.remotes()?;
    debug!("Found remotes: {:?}",
        remotes.iter().map(|v| v.unwrap()).collect::<Vec<&str>>()
    );

    let branches = repo.branches(None)?
        .map(|branch_res| {
            let (branch, _branch_type) = branch_res.unwrap();
            branch.name().unwrap().unwrap().to_string()
        })
        .collect::<Vec<String>>();
    debug!("Found branches: {branches:?}");

    for remote_str in remotes.iter() {
        let mut remote = repo.find_remote(remote_str.unwrap()).unwrap();

        let ref_specs_raw = remote.fetch_refspecs().unwrap();
        let ref_specs = ref_specs_raw
            .iter().map(|v| { v.unwrap() })
            .collect::<Vec<&str>>()
            ;

        let mut fo = FetchOptions::new();
        /*
        fo.custom_headers(&[r#""filter" SP blob:limit=15"#]); // Doesn't work!
        */

        let _ = match remote.fetch(&ref_specs, Some(&mut fo), None) {
            Ok(_) => (),
            Err(v) => {
                debug!(
                    "Can't clone from remote: {:?} with refspecs: {:?}. Error: {v:?}",
                    remote.name(),
                    ref_specs
                    );
            },
        };

        debug!("Fetching Updates...");

    }

    return Ok(repo);

}

/// Gets the head commit from a repo
pub fn get_head_commit(repo: &Repository) -> Commit {

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

