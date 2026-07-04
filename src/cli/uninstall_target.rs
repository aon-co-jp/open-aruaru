use crate::plugins::scan_conflicts;
use crate::state::AppState;

pub async fn run(app: &AppState, id: &str, purge_data: bool) -> anyhow::Result<()> {
    app.uninstall(id, purge_data).await?;
    println!("Uninstalled: {id}");
    for c in scan_conflicts(&app.pool, &app.root_dir).await? {
        println!("[conflict] {} — {}", c.plugin_id, c.message);
    }
    Ok(())
}
