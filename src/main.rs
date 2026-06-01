use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use std::process::Command;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "zetaprint", version = "0.1.0", author = "ZetaPrint Team", about = "ZetaPrint Print Management System")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install ZetaPrint as a systemd service
    Install,
    /// Start the ZetaPrint server (default)
    Start,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    match &cli.command {
        Some(Commands::Install) => {
            install_systemd_service()?;
        }
        Some(Commands::Start) | None => {
            start_server().await?;
        }
    }

    Ok(())
}

fn install_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    info!("Installing ZetaPrint Systemd Service...");
    
    // Create data directory
    let data_dir = "/var/lib/zetaprint";
    if !Path::new(data_dir).exists() {
        info!("Creating data directory at {}...", data_dir);
        fs::create_dir_all(data_dir)?;
    }
    
    // Create systemd service file
    let service_file = "/etc/systemd/system/zetaprint.service";
    let executable = std::env::current_exe()?;
    let exec_path = executable.to_str().unwrap_or("/usr/local/bin/zetaprint");
    
    let service_content = format!(r#"[Unit]
Description=ZetaPrint Management Service
After=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory={}
Environment=DATABASE_URL=sqlite:///var/lib/zetaprint/zetaprint.db
ExecStart={} start
Restart=on-failure
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
"#, data_dir, exec_path);

    info!("Writing systemd service file to {}...", service_file);
    fs::write(service_file, service_content)?;

    info!("Reloading systemd daemon...");
    let _ = Command::new("systemctl").arg("daemon-reload").status()?;

    info!("Enabling and starting zetaprint service...");
    let _ = Command::new("systemctl").args(&["enable", "--now", "zetaprint"]).status()?;

    info!("ZetaPrint installation completed successfully!");
    Ok(())
}

async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    if let Err(e) = dotenv() {
        println!("Warning: Could not load .env file: {}", e);
    }
    info!("Starting ZetaPrint Modular Monolith Server...");

    // 1. Initialize Database Module
    info!("Initializing core_db...");
    if let Err(e) = core_db::init().await {
        warn!("Failed to initialize database: {}. Continuing with fallback...", e);
    }

    // 2. Initialize Auth Module
    info!("Initializing core_auth...");
    core_auth::init().await?;

    // 3. Initialize Accounting / PDF Parsing Module
    info!("Initializing core_accounting...");
    core_accounting::init().await?;

    // 4. Initialize Dispatcher Module
    info!("Initializing core_dispatcher...");
    core_dispatcher::init().await?;

    // 5. Initialize IPP Server Module (Spawns background listener on Port 631)
    info!("Initializing core_ipp (IPP Print Server on port 631)...");
    let ipp_handle = tokio::spawn(async {
        if let Err(e) = core_ipp::start_server().await {
            tracing::error!("IPP Server error: {}", e);
        }
    });

    // 6. Initialize Web Server Axum (Serves API, SSE, and SPA Svelte UI)
    info!("Starting core_api HTTP Axum Web Server...");
    core_api::start_web_server().await?;

    // Wait for the IPP server task (though start_web_server runs infinitely)
    let _ = ipp_handle.await;

    Ok(())
}
