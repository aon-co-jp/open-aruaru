use poem::{
    handler, post,
    web::{Data, Json, Query},
    IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

use crate::ai_providers;
use crate::auth;
use crate::features;
use crate::github_projects;
use crate::handoff;
use crate::https_auto;
use crate::https_cert;
use crate::pptx_gen;
use crate::reminders;
use crate::server_status;
use crate::state::SharedState;
use crate::webserver_advisor;

#[derive(Deserialize)]
pub struct EmailBody {
    email: String,
}

#[handler]
pub async fn health(state: Data<&SharedState>) -> impl IntoResponse {
    let primary = state.primary_platform_domain().await.ok().flatten();
    Json(json!({
        "ok": true,
        "version": "0.1.0",
        "easyweb": "0.1.0",
        "registered": auth::has_admin(&state).await,
        "public_url": state.effective_public_url(),
        "primary_domain": primary.map(|d| d.name),
        "database": {
            "primary": state.config.database.primary,
            "backend": state.db_backend.as_str()
        }
    }))
}

#[handler]
pub async fn register_api(state: Data<&SharedState>, Json(body): Json<EmailBody>) -> impl IntoResponse {
    match auth::register_email(&state, &body.email).await {
        Ok(url) => Json(json!({ "ok": true, "magic_link": url, "message": "メールのリンクからログインしてください（SMTP未設定時はリンクを画面表示）" })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn login_api(state: Data<&SharedState>, Json(body): Json<EmailBody>) -> impl IntoResponse {
    match auth::request_login_link(&state, &body.email).await {
        Ok(url) => Json(json!({ "ok": true, "magic_link": url })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub struct DomainBody {
    name: String,
}

#[handler]
pub async fn add_domain(state: Data<&SharedState>, Json(body): Json<DomainBody>) -> impl IntoResponse {
    match state.add_domain(&body.name).await {
        Ok(()) => {
            if state.config.https_monitor.auto_configure {
                https_auto::schedule_configure(state.clone(), body.name.trim());
            }
            Json(json!({
                "ok": true,
                "https": {
                    "status": "configuring",
                    "message": "サイトドメイン登録後、自動で HTTPS を設定します"
                }
            }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn domains_api(state: Data<&SharedState>) -> impl IntoResponse {
    let domains = state.list_domains().await.unwrap_or_default();
    // 各ドメインに www バリアントを付与して返す（表示用）
    let with_www: Vec<serde_json::Value> = domains
        .iter()
        .map(|d| {
            let www = crate::vhost::www_variant(&d.name);
            json!({
                "name": d.name,
                "www": www,
                "https_enabled": d.https_enabled,
                "auto_https": d.auto_https,
                "last_checked": d.last_checked,
            })
        })
        .collect();
    Json(json!({ "ok": true, "domains": with_www }))
}

#[handler]
pub async fn delete_domain(
    state: Data<&SharedState>,
    Json(body): Json<DomainBody>,
) -> impl IntoResponse {
    match state.delete_domain(&body.name).await {
        Ok(()) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn reregister_domain(
    state: Data<&SharedState>,
    Json(body): Json<DomainBody>,
) -> impl IntoResponse {
    match state.reregister_domain(&body.name).await {
        Ok(()) => {
            if state.config.https_monitor.auto_configure {
                https_auto::schedule_configure(state.clone(), body.name.trim());
            }
            Json(json!({
                "ok": true,
                "message": "HTTPS 状態をリセットしました。自動再取得を開始します。"
            }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn plugins_api(state: Data<&SharedState>) -> impl IntoResponse {
    Json(json!({ "ok": true, "plugins": state.list_plugins().await.unwrap_or_default() }))
}

#[handler]
pub async fn install_api(state: Data<&SharedState>, Json(body): Json<InstallBody>) -> impl IntoResponse {
    match state.install(&body.target).await {
        Ok(ids) => Json(json!({ "ok": true, "installed": ids })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub struct InstallBody {
    target: String,
}

#[handler]
pub async fn platform_domains_list(state: Data<&SharedState>) -> impl IntoResponse {
    let domains = state.list_platform_domains().await.unwrap_or_default();
    let with_www: Vec<serde_json::Value> = domains
        .iter()
        .map(|d| {
            let www = crate::vhost::www_variant(&d.name);
            json!({
                "name": d.name,
                "www": www,
                "is_primary": d.is_primary,
                "https_enabled": d.https_enabled,
                "auto_https": d.auto_https,
                "last_checked": d.last_checked,
                "created_at": d.created_at,
            })
        })
        .collect();
    Json(json!({
        "ok": true,
        "domains": with_www,
        "public_url": state.effective_public_url()
    }))
}

#[handler]
pub async fn platform_domain_add(
    state: Data<&SharedState>,
    Json(body): Json<DomainBody>,
) -> impl IntoResponse {
    match state.add_platform_domain(&body.name).await {
        Ok(d) => {
            if d.auto_https != 0 && state.config.https_monitor.auto_configure {
                https_auto::schedule_configure(state.clone(), &d.name);
            }
            Json(json!({
                "ok": true,
                "domain": d,
                "public_url": state.effective_public_url(),
                "https": {
                    "status": "configuring",
                    "message": "ドメイン登録後、自動で HTTPS を設定します（DNS 反映後に certbot）。更新も自動メンテナンスします。"
                }
            }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub struct DomainNameBody {
    name: String,
}

#[derive(Deserialize)]
pub struct DomainSwapBody {
    from: String,
    to: String,
}

#[handler]
pub async fn platform_domain_delete(
    state: Data<&SharedState>,
    Json(body): Json<DomainNameBody>,
) -> impl IntoResponse {
    match state.delete_platform_domain(&body.name).await {
        Ok(()) => Json(json!({ "ok": true, "public_url": state.effective_public_url() })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn platform_domain_primary(
    state: Data<&SharedState>,
    Json(body): Json<DomainNameBody>,
) -> impl IntoResponse {
    match state.set_primary_platform_domain(&body.name).await {
        Ok(()) => Json(json!({ "ok": true, "public_url": state.effective_public_url() })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn platform_domain_swap(
    state: Data<&SharedState>,
    Json(body): Json<DomainSwapBody>,
) -> impl IntoResponse {
    match state.swap_platform_domain(&body.from, &body.to).await {
        Ok(()) => Json(json!({ "ok": true, "public_url": state.effective_public_url() })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn reminders_api(state: Data<&SharedState>) -> impl IntoResponse {
    let due = reminders::due(&state.pool, &state.config.reminders).await.unwrap_or(false);
    Json(json!({
        "ok": true,
        "due": due,
        "items": if due { reminders::items() } else { vec![] }
    }))
}

#[derive(Deserialize)]
pub struct DnsGuideQuery {
    zone: String,
    vps_ip: String,
}

#[derive(Deserialize)]
pub struct DnsRecordBody {
    zone: String,
    name: String,
    record_type: String,
    value: String,
    #[serde(default = "default_ttl")]
    ttl: i64,
}

fn default_ttl() -> i64 {
    3600
}

#[derive(Deserialize)]
pub struct DnsRecordIdBody {
    id: i64,
}

#[handler]
pub async fn dns_records_list(
    state: Data<&SharedState>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let zone = q.get("zone").map(|s| s.as_str());
    match state.list_dns_records(zone).await {
        Ok(records) => Json(json!({ "ok": true, "records": records })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn dns_record_add(
    state: Data<&SharedState>,
    Json(body): Json<DnsRecordBody>,
) -> impl IntoResponse {
    match state
        .add_dns_record(&body.zone, &body.name, &body.record_type, &body.value, body.ttl)
        .await
    {
        Ok(record) => Json(json!({ "ok": true, "record": record })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn dns_record_delete(
    state: Data<&SharedState>,
    Json(body): Json<DnsRecordIdBody>,
) -> impl IntoResponse {
    match state.delete_dns_record(body.id).await {
        Ok(()) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn dns_guide_api(
    state: Data<&SharedState>,
    Query(q): Query<DnsGuideQuery>,
) -> impl IntoResponse {
    match state.dns_guide(&q.zone, &q.vps_ip) {
        Ok(guide) => Json(json!({ "ok": true, "guide": guide })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn https_status_api(state: Data<&SharedState>) -> impl IntoResponse {
    Json(json!({
        "ok": true,
        "https": https_cert::monitor_status(&state)
    }))
}

#[handler]
pub async fn server_status_api(state: Data<&SharedState>) -> impl IntoResponse {
    let status = server_status::collect(&state).await;
    Json(server_status::status_json(&state, &status))
}

#[handler]
pub async fn handoff_targets_api() -> impl IntoResponse {
    Json(json!({ "ok": true, "data": handoff::handoff_targets() }))
}

#[handler]
pub async fn handoff_export(
    state: Data<&SharedState>,
    Json(body): Json<handoff::ExportRequest>,
) -> impl IntoResponse {
    match handoff::export_packet(&state, &body, None) {
        Ok(p) => {
            let md = handoff::to_markdown(&p);
            Json(json!({
                "ok": true,
                "packet": p,
                "markdown": md
            }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn handoff_import(
    state: Data<&SharedState>,
    Json(body): Json<handoff::ImportRequest>,
) -> impl IntoResponse {
    match handoff::import_text(&state, &body) {
        Ok(p) => Json(json!({ "ok": true, "packet": p })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub struct AiRecommendQuery {
    #[serde(default = "default_use_case")]
    use_case: String,
}

fn default_use_case() -> String {
    "development".into()
}

#[handler]
pub async fn ai_providers_list() -> impl IntoResponse {
    Json(json!({
        "ok": true,
        "providers": ai_providers::list_providers(),
        "stack": ai_providers::stack_for_development()
    }))
}

#[handler]
pub async fn ai_recommend(Query(q): Query<AiRecommendQuery>) -> impl IntoResponse {
    let uc = match q.use_case.as_str() {
        "english_conversation" => ai_providers::RecommendUseCase::EnglishConversation,
        "server_ops" => ai_providers::RecommendUseCase::ServerOps,
        "learning" => ai_providers::RecommendUseCase::Learning,
        "cheap" => ai_providers::RecommendUseCase::Cheap,
        "design" => ai_providers::RecommendUseCase::Design,
        _ => ai_providers::RecommendUseCase::Development,
    };
    let scored = ai_providers::recommend_scored(uc);
    let recommended: Vec<serde_json::Value> = scored
        .iter()
        .map(|s| {
            json!({
                "id": s.provider.id,
                "name": s.provider.name,
                "vendor": s.provider.vendor,
                "kind": s.provider.kind,
                "models": s.provider.models,
                "cost_tier": s.provider.cost_tier,
                "strengths": s.provider.strengths,
                "speech_quality": s.provider.speech_quality,
                "translation_quality": s.provider.translation_quality,
                "recommended_for": s.provider.recommended_for,
                "score": s.score,
                "reason": s.reason,
            })
        })
        .collect();
    Json(json!({
        "ok": true,
        "use_case": q.use_case,
        "recommended": recommended
    }))
}

#[handler]
pub async fn features_list(_state: Data<&SharedState>) -> impl IntoResponse {
    Json(json!({
        "ok": true,
        "features": features::list_vps_features()
    }))
}

#[derive(Deserialize)]
pub struct RunFeatureBody {
    id: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[handler]
pub async fn features_run(
    state: Data<&SharedState>,
    Json(body): Json<RunFeatureBody>,
) -> impl IntoResponse {
    match features::run_vps_feature(state.clone(), &body.id, &body.params).await {
        Ok(result) => Json(json!({ "ok": true, "id": body.id, "result": result })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ─── サーバー制御 ─────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct ServerControlBody {
    /// "start" | "stop" | "restart" | "enable" | "disable"
    action: String,
    /// "aruaru" | "nginx" | "apache2" など（省略時は "aruaru"）
    #[serde(default = "default_unit")]
    unit: String,
}
fn default_unit() -> String { "aruaru".to_string() }

#[handler]
async fn server_control(
    _state: Data<&SharedState>,
    Json(body): Json<ServerControlBody>,
) -> impl IntoResponse {
    let allowed_actions = ["start", "stop", "restart", "enable", "disable", "reload"];
    let allowed_units = ["aruaru", "nginx", "apache2", "httpd"];

    if !allowed_actions.contains(&body.action.as_str()) {
        return Json(json!({ "ok": false, "error": format!("不正な action: {}", body.action) }));
    }
    if !allowed_units.contains(&body.unit.as_str()) {
        return Json(json!({ "ok": false, "error": format!("不正な unit: {}", body.unit) }));
    }

    #[cfg(target_os = "linux")]
    {
        match std::process::Command::new("sudo")
            .args(["systemctl", &body.action, &body.unit])
            .output()
        {
            Ok(o) if o.status.success() => {
                Json(json!({ "ok": true, "message": format!("systemctl {} {} 完了", body.action, body.unit) }))
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                Json(json!({ "ok": false, "error": err.to_string() }))
            }
            Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        Json(json!({ "ok": false, "error": "サーバー制御は Linux VPS 上でのみ使用できます" }))
    }
}

// ─── Webサーバーアドバイザー ────────────────────────────────────

#[derive(Deserialize)]
struct WebserverAdvisorQuery {
    lang: String,
    #[serde(default = "default_app_port")]
    port: u16,
}
fn default_app_port() -> u16 { 3000 }

#[handler]
async fn webserver_advisor_api(Query(q): Query<WebserverAdvisorQuery>) -> impl IntoResponse {
    let rec = webserver_advisor::recommend(&q.lang, q.port);
    Json(json!({ "ok": true, "recommendation": rec }))
}

#[handler]
async fn webserver_compare_api() -> impl IntoResponse {
    let nginx = webserver_advisor::nginx_info();
    let apache = webserver_advisor::apache_info();
    let langs = webserver_advisor::lang_framework_entries();
    let databases = webserver_advisor::list_databases();
    let services  = webserver_advisor::list_additional_services();
    Json(json!({
        "ok": true,
        "nginx": nginx,
        "apache": apache,
        "lang_frameworks": langs,
        "databases": databases,
        "additional_services": services,
    }))
}

// ─── 拡張推薦（DB・サービス考慮） ────────────────────────────

#[derive(Deserialize)]
struct RecommendFullBody {
    lang: String,
    #[serde(default)]
    databases: Vec<String>,
    #[serde(default)]
    services: Vec<String>,
    #[serde(default = "default_app_port")]
    port: u16,
    #[serde(default = "default_domain")]
    domain: String,
}
fn default_domain() -> String { "example.com".to_string() }

#[handler]
async fn webserver_recommend_full(Json(body): Json<RecommendFullBody>) -> impl IntoResponse {
    let rec = webserver_advisor::recommend_full(
        &body.lang,
        &body.databases,
        &body.services,
        body.port,
        &body.domain,
    );
    Json(json!({ "ok": true, "recommendation": rec }))
}

// ─── 設定ファイルダウンロード ────────────────────────────────

#[derive(Deserialize)]
struct DownloadQuery {
    server: String,
    lang: String,
    #[serde(default = "default_domain")]
    domain: String,
    #[serde(default = "default_app_port")]
    port: u16,
    #[serde(default)]
    databases: String,   // カンマ区切り
    #[serde(default)]
    services: String,    // カンマ区切り
}

#[handler]
async fn webserver_config_download(Query(q): Query<DownloadQuery>) -> impl IntoResponse {
    let db_ids: Vec<String> = q.databases.split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    let svc_ids: Vec<String> = q.services.split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let content = webserver_advisor::generate_config_content(
        &q.server, &q.domain, q.port, &q.lang, &db_ids, &svc_ids,
    );

    let filename = format!(
        "{}-{}-{}.conf",
        q.server,
        q.domain.replace('.', "_"),
        q.lang.replace('-', "_"),
    );

    poem::Response::builder()
        .status(poem::http::StatusCode::OK)
        .header("Content-Type", "text/plain; charset=utf-8")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(content)
}

#[derive(Deserialize)]
struct WebserverSelectBody {
    server: String,   // "nginx" | "apache"
    domain: String,
    port: u16,
    lang: String,
}

#[handler]
async fn webserver_select_api(
    state: Data<&SharedState>,
    Json(body): Json<WebserverSelectBody>,
) -> impl IntoResponse {
    let server = body.server.trim().to_lowercase();
    if server != "nginx" && server != "apache" {
        return Json(json!({ "ok": false, "error": "server は nginx か apache を指定してください" }));
    }
    match webserver_advisor::generate_config(
        &state.data_dir,
        &server,
        &body.domain,
        body.port,
        &body.lang,
    ) {
        Ok(path) => {
            let msg_ja = format!(
                "{} 向け {} 設定ファイルを生成しました → {}",
                body.lang, server.to_uppercase(), path
            );
            let msg_en = format!(
                "{} config for {} generated → {}",
                server.to_uppercase(), body.lang, path
            );
            // vhost 再生成も実行
            let _ = crate::vhost::regenerate_all(&state).await;
            Json(json!({
                "ok": true,
                "path": path,
                "message_ja": msg_ja,
                "message_en": msg_en,
                "next_step_ja": format!(
                    "生成ファイルを /etc/{}/sites-available/ にコピーして有効化してください。",
                    if server == "nginx" { "nginx" } else { "apache2" }
                ),
                "next_step_en": format!(
                    "Copy the generated file to /etc/{}/sites-available/ and enable it.",
                    if server == "nginx" { "nginx" } else { "apache2" }
                )
            }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ─── GitHub プロジェクト ──────────────────────────────────

/// GitHub トークン・組織名の設定
#[derive(Deserialize)]
struct GithubConfigBody {
    token: String,
    #[serde(default)]
    org: Option<String>,
}

#[handler]
async fn github_config_save(
    state: Data<&SharedState>,
    Json(body): Json<GithubConfigBody>,
) -> impl IntoResponse {
    match state.save_github_config(body.token, body.org).await {
        Ok(_) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// リポジトリ一覧同期（GitHub API → DB）
#[handler]
async fn github_sync(state: Data<&SharedState>) -> impl IntoResponse {
    match state.sync_github_repos().await {
        Ok(projects) => Json(json!({ "ok": true, "count": projects.len(), "projects": projects })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// DB キャッシュ済みプロジェクト一覧
#[handler]
async fn github_list(state: Data<&SharedState>) -> impl IntoResponse {
    match state.list_github_projects().await {
        Ok(projects) => Json(json!({ "ok": true, "projects": projects })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// プロジェクト選択切り替え
#[derive(Deserialize)]
struct GithubToggleBody { gh_id: i64 }

#[handler]
async fn github_toggle(
    state: Data<&SharedState>,
    Json(body): Json<GithubToggleBody>,
) -> impl IntoResponse {
    match state.toggle_github_project(body.gh_id).await {
        Ok(selected) => Json(json!({ "ok": true, "selected": selected })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// 複数プロジェクトを一括選択
#[derive(Deserialize)]
struct GithubSelectBody { gh_ids: Vec<i64> }

#[handler]
async fn github_select_batch(
    state: Data<&SharedState>,
    Json(body): Json<GithubSelectBody>,
) -> impl IntoResponse {
    match state.set_github_projects_selected(&body.gh_ids).await {
        Ok(_) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// README を即時取得して DB に保存
#[derive(Deserialize)]
struct GithubReadmeBody { gh_id: i64 }

#[handler]
async fn github_fetch_readme(
    state: Data<&SharedState>,
    Json(body): Json<GithubReadmeBody>,
) -> impl IntoResponse {
    match state.fetch_and_store_readme(body.gh_id).await {
        Ok(_) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ─── PPTX 生成・ダウンロード ──────────────────────────────

/// 選択プロジェクトの PPTX を生成してダウンロード（純 Rust）
#[derive(Deserialize)]
struct PptxGenQuery {
    #[serde(default = "default_pptx_lang")]
    lang: String,
    /// カンマ区切りで gh_id を指定（省略時は選択中全件）
    #[serde(default)]
    ids: String,
}
fn default_pptx_lang() -> String { "ja".to_string() }

#[handler]
async fn github_pptx_download(
    state: Data<&SharedState>,
    Query(q): Query<PptxGenQuery>,
) -> impl IntoResponse {
    // 対象プロジェクト取得
    let projects = if q.ids.is_empty() {
        state.list_selected_github_projects().await.unwrap_or_default()
    } else {
        let ids: Vec<i64> = q.ids.split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        let all = state.list_github_projects().await.unwrap_or_default();
        all.into_iter().filter(|p| ids.contains(&p.gh_id)).collect()
    };

    if projects.is_empty() {
        return poem::Response::builder()
            .status(poem::http::StatusCode::BAD_REQUEST)
            .body("プロジェクトが選択されていません / No projects selected");
    }

    // スペック生成
    let spec = match pptx_gen::build_spec_from_projects(&projects, &q.lang).await {
        Ok(s) => s,
        Err(e) => return poem::Response::builder()
            .status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(e.to_string()),
    };

    // 純 Rust で PPTX バイト列生成
    let bytes = match pptx_gen::render_pptx(&spec) {
        Ok(b) => b,
        Err(e) => return poem::Response::builder()
            .status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(e.to_string()),
    };

    let filename = format!(
        "aruaru-{}-{}.pptx",
        spec.title.replace([' ', '/'], "_"),
        chrono::Utc::now().format("%Y%m%d"),
    );

    poem::Response::builder()
        .status(poem::http::StatusCode::OK)
        .header("Content-Type", "application/vnd.openxmlformats-officedocument.presentationml.presentation")
        .header("Content-Disposition", format!("attachment; filename=\"{filename}\""))
        .body(bytes)
}

pub fn api_routes() -> poem::Route {
    poem::Route::new()
        .at("/health", poem::get(health))
        .at("/server/status", poem::get(server_status_api))
        .at("/server/control", post(server_control))
        .at("/https/status", poem::get(https_status_api))
        .at("/register", post(register_api))
        .at("/login", post(login_api))
        .at("/domains", poem::get(domains_api).post(add_domain))
        .at("/domains/delete", post(delete_domain))
        .at("/domains/reregister", post(reregister_domain))
        .at("/platform/domains", poem::get(platform_domains_list).post(platform_domain_add))
        .at("/platform/domains/delete", post(platform_domain_delete))
        .at("/platform/domains/primary", post(platform_domain_primary))
        .at("/platform/domains/swap", post(platform_domain_swap))
        .at("/plugins", poem::get(plugins_api))
        .at("/install", post(install_api))
        .at("/reminders", poem::get(reminders_api))
        .at("/dns/records", poem::get(dns_records_list).post(dns_record_add))
        .at("/dns/records/delete", post(dns_record_delete))
        .at("/dns/guide", poem::get(dns_guide_api))
        .at("/handoff/targets", poem::get(handoff_targets_api))
        .at("/handoff/export", post(handoff_export))
        .at("/handoff/import", post(handoff_import))
        .at("/ai/providers", poem::get(ai_providers_list))
        .at("/ai/recommend", poem::get(ai_recommend))
        .at("/features", poem::get(features_list).post(features_run))
        .at("/webserver/advisor", poem::get(webserver_advisor_api))
        .at("/webserver/compare", poem::get(webserver_compare_api))
        .at("/webserver/select", post(webserver_select_api))
        .at("/webserver/recommend", post(webserver_recommend_full))
        .at("/webserver/download", poem::get(webserver_config_download))
        // GitHub 連携
        .at("/github/config",        post(github_config_save))
        .at("/github/sync",          post(github_sync))
        .at("/github/projects",      poem::get(github_list))
        .at("/github/toggle",        post(github_toggle))
        .at("/github/select",        post(github_select_batch))
        .at("/github/readme",        post(github_fetch_readme))
        .at("/github/pptx",          poem::get(github_pptx_download))
}
