use actix_web::http::StatusCode;
use clap::{
    Parser,
    command,
};

use std::{env, mem::{self, MaybeUninit}, path::{Path, PathBuf}};

use log::LevelFilter;

use super::errors;

/// Allows clap to use enum variants as variants in CLI.
#[doc(hidden)] #[macro_export]
macro_rules! clap_enum_variants {
    ($e: ty) => {{
        use clap::builder::TypedValueParser;
        clap::builder::PossibleValuesParser::new(
            <$e>::iter()
                .map(|v| {
                    v.to_string()
                })
                .collect::<Vec<String>>()
        )
        .map(|s| s.parse::<$e>().unwrap())
    }};
}


/// A utility for parsing through git repos
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct CliArgs {

    /// A flag for allowing the `file://` schema for urls
    #[clap(long, action, default_value="false")]
    pub allow_local: bool,

    /// The path to the temp directory for the repos
    /// If set to none, the system tries to find the tmp directory from the path of the executable "./tmp".
    #[clap(long, default_value="./tmp")]
    pub tmp: String,

    /// The path to where the static files are
    /// If set to none, the system tries to find the static directory from the path to the
    /// executable at "./static".
    #[clap(long, default_value=None)]
    pub static_dir: Option<String>,

    /// The path to where to executable is.
    /// Should be set every time an instance of this struct is initialized (should be set
    /// manually.)
    /// This value should always be Some(_) so it is safe to `.unwrap()` this value.
    project_location: Option<PathBuf>,

    /// The location of an ssh key for auth from the home directory.
    #[clap(short, long, default_value=".ssh/id_ed25519")]
    pub ssh_key: String,

    /// The amount of time in seconds that is allowed between commits.
    #[clap(short, long, default_value="18000")]
    pub time_allowed: i64,

    /// The amount of commits to go back when cloning a repo.
    #[clap(short='d', long, default_value="5000")]
    pub clone_depth: i64,

    /// The port to run the server on.
    #[clap(short='p', long, default_value="3000")]
    pub server_port: u16,

    /// Sets the verbosity of logs
    #[arg(long,
          default_value_t=LevelFilter::Info,
          value_name="LevelFilter",
          value_parser=clap_enum_variants!(LevelFilter))]
    pub log: LevelFilter,
}

impl CliArgs {


    /// Method for getting the value of tmp_path
    /// If self.tmp is absolute, this method simply returns self.tmp.
    /// If self.tmp is relative, this method joins the `self.tmp` value together with the `get_projection_location_as_ref()` value.
    pub fn get_tmp_path(&self) -> String {

        return match Path::new(&self.tmp).is_absolute() {
            true => (&self.tmp).clone(),
            false => self.get_project_location_as_ref().join(&self.tmp).display().to_string(),
        };

    }

    /// Method for initializing the project location value.
    pub fn set_project_location(mut self) -> Self {

        // Sets exec_location (the location of most project files
        self.project_location = Some(match env::current_exe() {
            Ok(v) => {
                v.parent().unwrap_or(Path::new("."))
                    .parent().unwrap_or(Path::new("."))
                    .parent().unwrap_or(Path::new("."))
                    .to_path_buf()
            },
            Err(_) => Path::new(".").to_path_buf(),
        });

        return self;

    }

    /// Method for getting the value of project_location from a reference
    /// Panics if project_location isn't set.
    pub fn get_project_location_as_ref(&self) -> &PathBuf {

        return match &self.project_location {
            Some(v) => v,
            None => {

                let err = errors::AppError {
                    cause: Some("Error: project_location isn't set!".to_string()),
                    message: Some("Error: project_location isn't set!".to_string()),
                    error_type: StatusCode::INTERNAL_SERVER_ERROR,
                };

                panic!("Error: project_location isn't set! `{:?}` (have you tried using the `set_project_location()` method?)", err);

            },
        };



    }

}
