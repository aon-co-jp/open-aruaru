mod api_key_handoff;
mod business_top100;
mod config;
mod copy_script_generator;
mod dashboard;
mod development_menu;
mod generator;
mod hybrid_theory;
mod language_top100;
mod language_update_job;
mod llm_folding;
mod manual_script_generator;
mod markdown;
mod programming_language_info;
mod quasi_quantum;
mod readme_output_menu;
mod watcher;

use anyhow::Result;
use poem::{get, listener::TcpListener, EndpointExt, Route, Server};

use crate::config::AppConfig;
use crate::dashboard::dashboard as dashboard_handler;
use crate::watcher::{new_status, run_initial_scan, run_scanner};

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::from_env_and_args()?;
    let status = new_status(&config);

    // Quality-gate integration smoke check: keeps the API key handoff planner connected
    // to the binary so clippy -D warnings does not treat the standard feature as dead code.
    let _ = api_key_handoff::quality_gate_smoke_check();
    let _ = readme_output_menu::quality_gate_smoke_check();
    let _ = development_menu::quality_gate_smoke_check();
    let _ = programming_language_info::quality_gate_smoke_check();
    let _ = language_update_job::quality_gate_smoke_check();
    let _ = language_top100::quality_gate_smoke_check();
    let _ = business_top100::quality_gate_smoke_check();

    // Generate README.rs / README.html for README.md files that already exist at startup.
    // FTP uploads may complete before the daemon starts, so relying only on later changes is unsafe.
    if let Err(err) = run_initial_scan(&config, &status).await {
        eprintln!("initial README scan failed: {err:#}");
    }

    let scanner_config = config.clone();
    let scanner_status = status.clone();
    tokio::spawn(async move {
        run_scanner(scanner_config, scanner_status).await;
    });

    let app = Route::new().at("/", get(dashboard_handler)).data(status);

    println!("aruaru-readme-auto-rs watching {}", config.root.display());
    println!("dashboard: http://{}", config.listen);
    println!("output mode: {}", config.output_mode.as_str());

    Server::new(TcpListener::bind(config.listen))
        .run(app)
        .await?;
    Ok(())
}
