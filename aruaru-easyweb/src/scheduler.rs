//! 証明書自動更新スケジューラ（毎日 03:00 実行）

use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;
use crate::db::Pool;

pub async fn start(pool: Pool) -> Result<()> {
    let sched = JobScheduler::new().await?;

    sched.add(Job::new_async("0 0 3 * * *", move |_, _| {
        let p = pool.clone();
        Box::pin(async move {
            info!("証明書自動更新スケジューラ実行");
            if let Err(e) = crate::domain::ssl::auto_renew_all(&p).await {
                tracing::error!("証明書自動更新エラー: {}", e);
            }
        })
    })?).await?;

    sched.start().await?;
    info!("スケジューラ起動完了（毎日 03:00 に証明書チェック）");
    Ok(())
}
