// aruaru-easyweb — VPSサーバー エントリポイント

mod ai_providers;
mod auth;
mod cli;
mod config;
mod dns;
mod features;
mod generator;
mod handoff;
mod https_auto;
mod https_cert;
mod https_monitor;
mod linux_os;
mod markdown;
mod paths;
mod platform_domain;
mod plugins;
mod recovery;
mod reminders;
mod routes;
mod server_status;
mod session;
mod state;
mod storage;
mod github_crawler;
mod github_projects;
mod pptx_gen;
mod vhost;
mod watcher;
mod webserver_advisor;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = cli::Args::parse();
    cli::run(args).await
}
