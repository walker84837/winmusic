use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    /// The path to the config file
    #[clap(short, long, default_value = "config.kdl")]
    pub config: PathBuf,
}
