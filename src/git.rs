/// Module for holding utility functions for git methods

use git2::{Commit, Cred, RemoteCallbacks, Repository};
use std::{env, path::Path};
use anyhow::{Context, Result};
use log::debug;
use super::Timestamp;

#[derive(Clone)]
pub struct CommitData {
    pub message: String,
    pub timestamp: Timestamp,
    pub prev_timestamp: Timestamp,
    pub delta_t: Timestamp,
    pub files_changed: i32,
    pub lines_added: i32,
    pub lines_removed: i32,
    pub projected: bool,
}

/// Function for cloning a repo
pub fn fetch_repo(ssh_url: &str, ssh_key_path: &str, out_dir: &Path) -> Result<Repository> {

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
    return Ok(match out_dir.is_dir() {
        true => Repository::open(out_dir).with_context(|| "Can't find the repo directory (do you have the right path?)")?,
        false => builder.clone(ssh_url, out_dir).with_context(|| format!("Can't clone repo to `{out_dir:?}` (do you have the right URL?)"))?,
    });
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

