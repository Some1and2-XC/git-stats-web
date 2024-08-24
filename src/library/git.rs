use actix_web::web::Data;
use git2::{Commit, RemoteCallbacks, Repository, Progress};
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

    // Prepares fetch options
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);
    fo.depth(args.clone_depth as i32); // Sets max depth to the repo clone

    // Prepare builder.
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    // Clones the repo
    if out_dir.is_dir() {
        debug!("Updating Index...");
        let repo = Repository::open(out_dir).with_context(|| "Can't find the repo directory (do you have the right path?)")?;

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

            let _ = match remote.fetch(&ref_specs, None, None) {
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

    } else {
        debug!("Cloning for the first time...");
        return Ok(builder.clone(ssh_url, out_dir).with_context(|| format!("Can't clone repo to `{out_dir:?}` (do you have the right URL?)"))?);
    }

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

