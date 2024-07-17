use git2::{Cred, DescribeFormatOptions, DescribeOptions, Error, ErrorClass, ErrorCode, ObjectType, RemoteCallbacks, Repository};
use std::{env, path::Path};

fn main() {

    // Sets Credential callback
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        let home_env = match env::var("HOME") {
            Ok(v) => v,
            Err(_) => return Err(Error::new(ErrorCode::Directory, ErrorClass::Os, "Can't find environment variable `$HOME` for ssh!".to_string())),
        };

        Cred::ssh_key(username_from_url.unwrap(),
            None,
            Path::new(&format!("{}/.ssh/id_ed25519", home_env)),
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
    let repo_dir = Path::new("./tmp/git-stats");
    let ssh_url = "git@github.com:some1and2-xc/git-stats/";
    let repo = match repo_dir.is_dir() {
        true => Repository::open(repo_dir).unwrap(),
        false => builder.clone(ssh_url, repo_dir).unwrap(),
    };

    let head_oid = repo
        .head()
        .unwrap()
        .target()
        .unwrap();

    let head_object = repo
        .find_object(head_oid, Some(ObjectType::Commit))
        .unwrap();

    let head = head_object
        .as_commit()
        .unwrap();


    println!("HEAD OID: `{head:?}`!");

    let header = head
        .header_field_bytes("committer")
        .unwrap();

    let header_str = header
        .as_str()
        .unwrap();

    println!("Head:```\n{header_str}\n```!");
}
