use anyhow::Result;
use clap::Parser;
use jfrog2nexus::cli::Cli;
use jfrog2nexus::cli::commands::{Commands, ConfigSubcommands};
use tracing::{info, error, warn};
use tracing_subscriber::EnvFilter;
use std::sync::Arc;


#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with JSON formatter and environment filter
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Sync(args) => {
            // Setup metrics
            let metrics_handle = jfrog2nexus::observability::metrics::setup_metrics_recorder();
            let metrics_addr: std::net::SocketAddr = args.metrics_addr.parse()
                .expect("Invalid metrics address");
            
            tokio::spawn(jfrog2nexus::observability::metrics::start_metrics_server(
                metrics_handle,
                metrics_addr,
            ));

            info!(path = %args.config, "Loading configuration for sync");
            let app_config = jfrog2nexus::config::load_config(&args.config).await?;
            
            let client = Arc::new(jfrog2nexus::engine::create_client(app_config.proxy.as_ref().map(|p| &p.url))?);
            let scanner = jfrog2nexus::engine::scanner::Scanner::new(&client, &app_config.jfrog);
            
            if args.dry_run {
                info!("Dry-run mode enabled. Scanning repositories...");
                let plan = scanner.build_plan(&app_config.mappings).await?;
                
                info!("--- SYNC PLAN ---");
                for artifact in &plan.artifacts {
                    info!(
                        path = %artifact.path,
                        size = artifact.size,
                        repo_type = ?artifact.repo_type,
                        "Artifact found"
                    );
                }
                info!(
                    total_artifacts = plan.artifacts.len(),
                    total_size = plan.total_size,
                    "Dry-run complete"
                );
            } else {
                info!("Starting sync execution...");
                let plan = scanner.build_plan(&app_config.mappings).await?;
                
                let state_store = if args.resume_by_checksum {
                    Some(Arc::new(jfrog2nexus::engine::state_store::StateStore::new(".j2n/state.db").await?))
                } else {
                    None
                };

                let rate_limiter = if args.max_kbps > 0 {
                    Some(jfrog2nexus::engine::throttler::create_limiter(args.max_kbps))
                } else {
                    None
                };

                let orchestrator = jfrog2nexus::engine::transfer::TransferOrchestrator::new(
                    client.clone(),
                    app_config.jfrog,
                    app_config.nexus,
                    args.concurrency,
                    state_store,
                    rate_limiter,
                );
                
                if let Err(e) = orchestrator.execute_plan(plan).await {
                    error!(error = %e, "Sync failed");
                    std::process::exit(1);
                }
            }
        },
        Commands::Status(args) => {
            let db_url = format!("sqlite:{}", args.db_path);
            let state_store = jfrog2nexus::engine::state_store::StateStore::new(&db_url).await?;
            let (count, total_size_bytes) = state_store.get_stats().await?;
            let total_size_mb = total_size_bytes as f64 / 1024.0 / 1024.0;

            info!(
                database = %args.db_path,
                completed_artifacts = count,
                total_migrated_mb = total_size_mb,
                "Migration Status"
            );
            
            // Try to reach metrics server for real-time info
            let client = reqwest::Client::new();
            match client.get(format!("{}/metrics", args.metrics_url)).send().await {
                Ok(resp) => {
                    if let Ok(text) = resp.text().await {
                        let mut bytes_total = 0.0;
                        for line in text.lines() {
                            if line.starts_with("j2n_transfer_bytes_total") {
                                if let Some(val_str) = line.split_whitespace().last() {
                                    bytes_total = val_str.parse().unwrap_or(0.0);
                                }
                            }
                        }
                        info!(
                            metrics_server = %args.metrics_url,
                            live_bytes_transferred = bytes_total,
                            "Metrics Server reachable"
                        );
                    }
                }
                Err(_) => {
                    warn!(
                        metrics_server = %args.metrics_url,
                        "Metrics Server not reachable (migration might not be running)"
                    );
                }
            }
        },
        Commands::Report(args) => {
            use jfrog2nexus::cli::commands::ReportSubcommands;
            match args.command {
                ReportSubcommands::Generate { db_path, output } => {
                    let db_url = format!("sqlite:{}", db_path);
                    let state_store = jfrog2nexus::engine::state_store::StateStore::new(&db_url).await?;
                    let records = state_store.get_all_records().await?;
                    
                    info!(count = records.len(), "Generating audit report");
                    jfrog2nexus::audit::generate_csv_report(&records, &output)?;
                    info!(path = %output, "Audit report generated successfully");
                }
            }
        },
        Commands::GenerateCompletions(args) => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(args.shell, &mut cmd, bin_name, &mut std::io::stdout());
        },
        Commands::Config(args) => {
            match args.command {
                ConfigSubcommands::Validate { config } => {
                    info!(path = %config, "Validating configuration");
                    let app_config = jfrog2nexus::config::load_config(&config).await?;
                    
                    info!("Checking upstream connectivity...");
                    let client = jfrog2nexus::engine::create_client(app_config.proxy.as_ref().map(|p| &p.url))?;
                    
                    jfrog2nexus::engine::check_jfrog_connectivity(&app_config.jfrog, &client).await?;
                    info!("JFrog connectivity: OK");
                    
                    jfrog2nexus::engine::check_nexus_connectivity(&app_config.nexus, &client).await?;
                    info!("Nexus connectivity: OK");
                    
                    info!("Configuration and connectivity are valid");
                }
            }
        }
    }

    Ok(())
}
