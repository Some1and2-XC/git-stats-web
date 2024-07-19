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

    /// The path to the repo
    #[clap(short, long, default_value=".")]
    pub directory: String,

    /// The URL of the repo, if this is set, the repo will be fetched.
    #[clap(short, long, default_value="file:")]
    pub url: String,

    /// The path to the temp directory for the repos
    #[clap(long, default_value="./tmp")]
    pub tmp: String,

    /// The file to write the output to
    #[clap(short, long, default_value=None)]
    pub outfile: Option<String>,

    /// The location of an ssh key for auth from the home directory.
    #[clap(short, long, default_value=".ssh/id_ed25519")]
    pub ssh_key: String,

    /// The amount of time in seconds that is allowed between commits
    #[clap(short, long, default_value="18000")]
    pub time_allowed: i64,

    /// Flag refering to if the server should start.
    #[clap(short='S', long, action)]
    pub server: bool,

    /// The directory for the static server files.
    #[clap(short='D', long, default_value="./static")]
    pub server_directory: String,

    /// The port to run the server on.
    #[clap(short='P', long, default_value="3000")]
    pub server_port: u16,

    /// Sets the verbosity of logs
    #[arg(long,
          default_value_t=LevelFilter::Off,
          value_name="LevelFilter",
          value_parser=clap_enum_variants!(LevelFilter))]
    pub logs: LevelFilter,
}
