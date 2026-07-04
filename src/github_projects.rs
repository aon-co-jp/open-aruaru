//! GitHub プロジェクト連携
//!
//! - トークン設定（config.toml に github_token を追加）
//! - リポジトリ一覧取得（GitHub REST API v3）
//! - 表示するプロジェクトの選択・保存（SQLite）
//! - README.md 取得（base64 デコード）
//! - コードファイル拡張子サマリー取得

use serde::{Deserialize, Serialize};

use crate::state::AppState;

// ─────────────────────────────────────────────────────────
// 構造体
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GithubProject {
    pub id: i64,
    pub gh_id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: i64,
    pub updated_at_gh: String,
    pub selected: i64, // 0 or 1
    pub readme_content: Option<String>,
    pub ext_summary: Option<String>,      // JSON: [["rs",42],...]
    pub saved_at: String,
    pub readme_fetched_at: Option<String>,
}

impl GithubProject {
    /// readme_fetched_at を Option<String> から返す
    pub fn readme_fetched_at_opt(&self) -> Option<&str> {
        self.readme_fetched_at.as_deref()
    }
}

/// GitHub API から返るリポジトリ（軽量）
#[derive(Debug, Deserialize)]
pub struct GhRepo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: i64,
    pub updated_at: String,
}

/// GitHub API Content レスポンス
#[derive(Debug, Deserialize)]
struct GhContent {
    encoding: String,
    content: String,
}

// ─────────────────────────────────────────────────────────
// GitHub API ヘルパー
// ─────────────────────────────────────────────────────────

fn gh_client(token: &str) -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("aruaru/0.1")
        .default_headers({
            let mut h = reqwest::header::HeaderMap::new();
            h.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            );
            h.insert(
                reqwest::header::ACCEPT,
                "application/vnd.github+json".parse().unwrap(),
            );
            h
        })
        .build()
        .expect("reqwest client")
}

/// 認証ユーザーのリポジトリ一覧を取得（最大 100 件、スター順）
pub async fn fetch_user_repos(token: &str) -> anyhow::Result<Vec<GhRepo>> {
    let client = gh_client(token);
    let url = "https://api.github.com/user/repos?per_page=100&sort=updated&direction=desc&type=all";
    let resp: Vec<GhRepo> = client.get(url).send().await?.json().await?;
    Ok(resp)
}

/// 組織のリポジトリ一覧を取得
pub async fn fetch_org_repos(token: &str, org: &str) -> anyhow::Result<Vec<GhRepo>> {
    let client = gh_client(token);
    let url = format!("https://api.github.com/orgs/{org}/repos?per_page=100&sort=updated&direction=desc");
    let resp: Vec<GhRepo> = client.get(&url).send().await?.json().await?;
    Ok(resp)
}

/// README.md を取得（Markdown テキスト）
pub async fn fetch_readme(token: &str, full_name: &str) -> anyhow::Result<String> {
    let client = gh_client(token);
    let url = format!("https://api.github.com/repos/{full_name}/contents/README.md");
    let resp: GhContent = client.get(&url).send().await?.json().await?;
    if resp.encoding == "base64" {
        let clean: String = resp.content.chars().filter(|c| *c != '\n' && *c != '\r').collect();
        let bytes = base64_decode(&clean)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    } else {
        Ok(resp.content)
    }
}

/// ルートディレクトリのファイル一覧を取得し、拡張子をカウント
pub async fn fetch_extension_summary(
    token: &str,
    full_name: &str,
) -> anyhow::Result<Vec<(String, usize)>> {
    let client = gh_client(token);
    let url = format!("https://api.github.com/repos/{full_name}/git/trees/HEAD?recursive=1");

    #[derive(Deserialize)]
    struct Tree {
        tree: Vec<TreeItem>,
    }
    #[derive(Deserialize)]
    struct TreeItem {
        path: String,
        #[serde(rename = "type")]
        kind: String,
    }

    let resp: Tree = client.get(&url).send().await?.json().await?;
    let mut counts: std::collections::HashMap<String, usize> = Default::default();
    for item in &resp.tree {
        if item.kind == "blob" {
            if let Some(ext) = std::path::Path::new(&item.path)
                .extension()
                .and_then(|e| e.to_str())
            {
                if KNOWN_EXTENSIONS.contains(&ext) {
                    *counts.entry(ext.to_string()).or_default() += 1;
                }
            }
        }
    }
    let mut list: Vec<(String, usize)> = counts.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(list)
}

/// 表示対象として既知の拡張子
const KNOWN_EXTENSIONS: &[&str] = &[
    // Rust
    "rs",
    // TypeScript / JavaScript
    "ts", "tsx", "js", "jsx", "mjs", "cjs",
    // Python
    "py", "pyi",
    // Mojo
    "mojo", "🔥",
    // Go
    "go",
    // Java / Kotlin / Scala
    "java", "kt", "scala",
    // C / C++ / C#
    "c", "cpp", "cc", "h", "hpp", "cs",
    // Ruby
    "rb",
    // PHP
    "php",
    // Swift / Objective-C
    "swift", "m",
    // HTML / CSS / SCSS
    "html", "htm", "css", "scss", "sass",
    // Shell
    "sh", "bash", "zsh",
    // SQL
    "sql",
    // YAML / TOML / JSON
    "yml", "yaml", "toml", "json",
    // Docker / config
    "dockerfile",
    // Markdown
    "md", "mdx",
    // WebAssembly
    "wasm", "wat",
];

/// 簡易 base64 デコード（padding 調整つき）
fn base64_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    use std::io::Read;
    // 標準ライブラリの base64 はないので、自前で簡易実装
    // ただし aruaru の依存に追加せず、シンプルに処理
    let alphabet =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut table = [0u8; 256];
    for (i, &b) in alphabet.iter().enumerate() {
        table[b as usize] = i as u8;
    }

    let s = s.trim_end_matches('=');
    let mut bits: u32 = 0;
    let mut nbits = 0u32;
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    for b in s.bytes() {
        if b.is_ascii_whitespace() { continue; }
        bits = (bits << 6) | table[b as usize] as u32;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            out.push((bits >> nbits) as u8);
        }
    }
    Ok(out)
}

// ─────────────────────────────────────────────────────────
// DB メソッド (AppState)
// ─────────────────────────────────────────────────────────

impl AppState {
    /// GitHub トークンを config から取得
    pub fn github_token(&self) -> Option<String> {
        self.config.github.as_ref()
            .and_then(|g| g.token.clone())
            .filter(|t| !t.is_empty())
    }

    /// GitHub からリポジトリ一覧を取得して DB に保存
    pub async fn sync_github_repos(&self) -> anyhow::Result<Vec<GithubProject>> {
        let token = self.github_token()
            .ok_or_else(|| anyhow::anyhow!("GitHub トークン未設定"))?;

        let repos = fetch_user_repos(&token).await?;
        let now = chrono::Utc::now().to_rfc3339();

        for repo in &repos {
            sqlx::query(
                "INSERT INTO github_projects (gh_id, name, full_name, description, html_url, language, stargazers_count, updated_at_gh, selected, readme_content, saved_at)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8, COALESCE((SELECT selected FROM github_projects WHERE gh_id=$1), 0), NULL, $9)
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
            .execute(&self.pool)
            .await?;
        }

        self.list_github_projects().await
    }

    /// DB のプロジェクト一覧を取得
    pub async fn list_github_projects(&self) -> anyhow::Result<Vec<GithubProject>> {
        Ok(sqlx::query_as::<_, GithubProject>(
            "SELECT id, gh_id, name, full_name, description, html_url, language,
                    stargazers_count, updated_at_gh, selected, readme_content,
                    ext_summary, saved_at, readme_fetched_at
             FROM github_projects ORDER BY selected DESC, stargazers_count DESC, updated_at_gh DESC",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    /// 選択中のプロジェクトのみ取得
    pub async fn list_selected_github_projects(&self) -> anyhow::Result<Vec<GithubProject>> {
        Ok(sqlx::query_as::<_, GithubProject>(
            "SELECT id, gh_id, name, full_name, description, html_url, language,
                    stargazers_count, updated_at_gh, selected, readme_content,
                    ext_summary, saved_at, readme_fetched_at
             FROM github_projects WHERE selected=1
             ORDER BY stargazers_count DESC, updated_at_gh DESC",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    /// プロジェクトの選択状態を切り替え
    pub async fn toggle_github_project(&self, gh_id: i64) -> anyhow::Result<bool> {
        let current: i64 = sqlx::query_scalar(
            "SELECT selected FROM github_projects WHERE gh_id=$1",
        )
        .bind(gh_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("project not found: {gh_id}"))?;

        let next = if current == 0 { 1i64 } else { 0i64 };
        sqlx::query("UPDATE github_projects SET selected=$1 WHERE gh_id=$2")
            .bind(next)
            .bind(gh_id)
            .execute(&self.pool)
            .await?;
        Ok(next == 1)
    }

    /// 複数プロジェクトの選択状態を一括設定
    pub async fn set_github_projects_selected(
        &self,
        gh_ids: &[i64],
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE github_projects SET selected=0")
            .execute(&self.pool)
            .await?;
        for id in gh_ids {
            sqlx::query("UPDATE github_projects SET selected=1 WHERE gh_id=$1")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    /// README を取得して DB に保存
    pub async fn fetch_and_store_readme(&self, gh_id: i64) -> anyhow::Result<String> {
        let token = self.github_token()
            .ok_or_else(|| anyhow::anyhow!("GitHub トークン未設定"))?;

        let full_name: String = sqlx::query_scalar(
            "SELECT full_name FROM github_projects WHERE gh_id=$1",
        )
        .bind(gh_id)
        .fetch_one(&self.pool)
        .await?;

        let readme = fetch_readme(&token, &full_name).await?;
        sqlx::query("UPDATE github_projects SET readme_content=$1 WHERE gh_id=$2")
            .bind(&readme)
            .bind(gh_id)
            .execute(&self.pool)
            .await?;
        Ok(readme)
    }

    /// GitHub 設定（トークン・組織名）を保存
    pub async fn save_github_config(
        &self,
        token: String,
        org: Option<String>,
    ) -> anyhow::Result<()> {
        let mut cfg = self.read_config_file()?;
        cfg.github = Some(crate::config::GithubConfig { token: Some(token), org });
        self.write_config_file_pub(&cfg).await
    }
}
