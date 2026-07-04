use crate::plugins::scan_conflicts;
use crate::state::AppState;

pub async fn run(app: &AppState, target: &str) -> anyhow::Result<()> {
    let ids = app.install(target).await?;
    println!("Installed: {}", ids.join(", "));
    for c in scan_conflicts(&app.pool, &app.root_dir).await? {
        println!("[conflict] {} — {}", c.plugin_id, c.message);
    }
    Ok(())
}
