use clap::{
    Parser,
    command,
};

use log::LevelFilter;

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
    #[clap(long, default_value="./tmp")]
    pub tmp: String,

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
