use clap::{Args, Subcommand};

/// Top-level subcommands available on the `jfrog2nexus` CLI.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Sync artifacts from JFrog to Nexus
    Sync(SyncArgs),
    /// Get status of the migration
    Status(StatusArgs),
    /// Generate migration reports
    Report(ReportArgs),
    /// Generate shell autocompletion scripts
    GenerateCompletions(GenerateCompletionsArgs),
    /// Validate configuration and secrets
    Config(ConfigArgs),
}

/// Arguments for the `sync` subcommand.
#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Path to matching configuration (e.g., j2n.yaml)
    #[arg(short, long, default_value = ".j2n/j2n.yaml")]
    pub config: String,

    /// Run without performing actual transfers
    #[arg(long)]
    pub dry_run: bool,

    /// Resume partially completed transfers based on local state database
    #[arg(long)]
    pub resume_by_checksum: bool,

    /// Maximum transfer rate in KB/s (0 for unlimited)
    #[arg(long, default_value_t = 0)]
    pub max_kbps: u64,

    /// Number of concurrent transfers
    #[arg(short = 'n', long, default_value_t = 50)]
    pub concurrency: usize,

    /// Address to bind metrics server to (e.g. 127.0.0.1:9090)
    #[arg(long, default_value = "127.0.0.1:9090")]
    pub metrics_addr: String,
}

/// Arguments for the `status` subcommand.
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Path to state database
    #[arg(long, default_value = ".j2n/state.db")]
    pub db_path: String,

    /// URL of the metrics server to query for real-time stats
    #[arg(long, default_value = "http://127.0.0.1:9090")]
    pub metrics_url: String,
}

/// Arguments for the `report` subcommand.
#[derive(Args, Debug)]
pub struct ReportArgs {
    #[command(subcommand)]
    pub command: ReportSubcommands,
}

/// Subcommands available under `report`.
#[derive(Subcommand, Debug)]
pub enum ReportSubcommands {
    /// Generate a CSV audit report
    Generate {
        /// Path to state database
        #[arg(long, default_value = ".j2n/state.db")]
        db_path: String,
        /// Path to output CSV file
        #[arg(short, long, default_value = "migration_report.csv")]
        output: String,
    },
}

/// Arguments for the `generate-completions` subcommand.
#[derive(Args, Debug)]
pub struct GenerateCompletionsArgs {
    /// Shell to generate completions for
    pub shell: clap_complete::Shell,
}

/// Arguments for the `config` subcommand.
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigSubcommands,
}

/// Subcommands available under `config`.
#[derive(Subcommand, Debug)]
pub enum ConfigSubcommands {
    /// Validate configuration file and environment variables
    Validate {
        /// Path to configuration file
        #[arg(short, long, default_value = ".j2n/j2n.yaml")]
        config: String,
    },
}
