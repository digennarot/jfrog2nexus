pub mod commands;

use self::commands::Commands;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "jfrog2nexus", 
    about = "Migrate from JFrog Artifactory to Nexus", 
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
