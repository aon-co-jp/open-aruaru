//! GitHub 自動クロール — 10分ごとに選択リポジトリの最新データを取得
//!
//! 取得内容:
//!   - リポジトリ一覧のリフレッシュ（stars / description / updated_at）
//!   - 選択中プロジェクトの README.md（前回取得から変更があった場合のみ）
//!   - コードファイル拡張子サマリー（ツリー API）

use std::sync::Arc;

use crate::github_projects::{fetch_extension_summary, fetch_readme, fetch_user_repos};
use crate::state::AppState;

/// クロール間隔（10分）
const CRAWL_INTERVAL_SECS: u64 = 10 * 60;

/// バックグラウンドで定期クロールを開始する
pub fn spawn_github_crawler(state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            // 起動直後は少し待ってから実行（サーバー起動完了後に実施）
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

            if let Err(e) = run_crawl(&state).await {
                tracing::warn!("GitHub auto-crawl error: {e}");
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(CRAWL_INTERVAL_SECS)).await;
        }
    });
}

async fn run_crawl(state: &AppState) -> anyhow::Result<()> {
    let token = match state.github_token() {
        Some(t) => t,
        None => {
            tracing::debug!("GitHub クロール: トークン未設定のためスキップ");
            return Ok(());
        }
    };

    tracing::info!("GitHub 自動クロール開始");

    // ① リポジトリ一覧をリフレッシュ（all repos → DB upsert）
    let repos = match fetch_user_repos(&token).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("GitHub repos 取得失敗: {e}");
            return Err(e);
        }
    };

    let now = chrono::Utc::now().to_rfc3339();

    // 組織リポジトリが設定されていれば追加取得
    let org_repos = if let Some(org) = state.config.github.as_ref().and_then(|g| g.org.clone()) {
        crate::github_projects::fetch_org_repos(&token, &org)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    for repo in repos.iter().chain(org_repos.iter()) {
        let _ = sqlx::query(
            "INSERT INTO github_projects
               (gh_id, name, full_name, description, html_url, language,
                stargazers_count, updated_at_gh, selected, readme_content, saved_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,
               COALESCE((SELECT selected FROM github_projects WHERE gh_id=$1), 0),
               (SELECT readme_content FROM github_projects WHERE gh_id=$1),
               $9)
             ON CONFLICT(gh_id) DO UPDATE SET
               name=excluded.name, full_name=excluded.full_name,
               description=excluded.description, html_url=excluded.html_url,
               language=excluded.language, stargazers_count=excluded.stargazers_count,
               updated_at_gh=excluded.updated_at_gh, saved_at=excluded.saved_at",
        )
        .bind(repo.id)
        .bind(&repo.name)
        .bind(&repo.full_name)
        .bind(&repo.description)
        .bind(&repo.html_url)
        .bind(&repo.language)
        .bind(repo.stargazers_count)
        .bind(&repo.updated_at)
        .bind(&now)
        .execute(&state.pool)
        .await;
    }

    tracing::info!("GitHub リポジトリ {} 件を同期", repos.len() + org_repos.len());

    // ② 選択中プロジェクトの README + 拡張子サマリーを更新
    let selected = state.list_selected_github_projects().await.unwrap_or_default();

    for proj in &selected {
        // README.md 取得（updated_at_gh が変わった場合 or 未取得）
        let needs_readme = proj.readme_content.is_none()
            || proj.readme_fetched_at_opt().map(|t| t < proj.updated_at_gh).unwrap_or(true);

        if needs_readme {
            match fetch_readme(&token, &proj.full_name).await {
                Ok(readme) => {
                    let _ = sqlx::query(
                        "UPDATE github_projects SET readme_content=$1, readme_fetched_at=$2 WHERE gh_id=$3",
                    )
                    .bind(&readme)
                    .bind(&now)
                    .bind(proj.gh_id)
                    .execute(&state.pool)
                    .await;
                    tracing::info!("README 更新: {}", proj.full_name);
                }
                Err(e) => {
                    tracing::warn!("README 取得失敗 {}: {e}", proj.full_name);
                }
            }
        }

        // 拡張子サマリー（未取得 or 1日以上経過）
        let needs_ext = needs_readme; // README と同タイミングで更新
        if needs_ext {
            match fetch_extension_summary(&token, &proj.full_name).await {
                Ok(exts) => {
                    if let Ok(json) = serde_json::to_string(&exts) {
                        let _ = sqlx::query(
                            "UPDATE github_projects SET ext_summary=$1 WHERE gh_id=$2",
                        )
                        .bind(&json)
                        .bind(proj.gh_id)
                        .execute(&state.pool)
                        .await;
                    }
                }
                Err(e) => {
                    tracing::warn!("拡張子サマリー取得失敗 {}: {e}", proj.full_name);
                }
            }
        }

        // GitHub API レート制限対策（1秒待機）
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    tracing::info!("GitHub 自動クロール完了（選択中 {} 件を更新）", selected.len());
    Ok(())
}
