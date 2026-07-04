//! 機能一覧（VPS Web / デスクトップ共通カタログ）と実行

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ai_providers;
use crate::handoff;
use crate::https_cert;
use crate::https_monitor;
use crate::state::SharedState;
use crate::vhost;

#[derive(Debug, Clone, Serialize)]
pub struct FeatureItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    /// vps | desktop | both
    pub platform: String,
    pub runnable: bool,
}

#[derive(Debug, Deserialize)]
pub struct RunFeatureRequest {
    pub id: String,
    #[serde(default)]
    pub params: Value,
}

pub fn list_vps_features() -> Vec<FeatureItem> {
    vec![
        FeatureItem {
            id: "handoff.export".into(),
            name: "プロジェクト引き継ぎ（エクスポート）".into(),
            description: "params: project, to=cursor|claude|gemini|deepseek|aruaru".into(),
            category: "handoff".into(),
            platform: "both".into(),
            runnable: true,
        },
        FeatureItem {
            id: "ai.recommend".into(),
            name: "AI おすすめ（用途別）".into(),
            description: "params: use_case=development|english_conversation|cheap".into(),
            category: "ai".into(),
            platform: "both".into(),
            runnable: true,
        },
        FeatureItem {
            id: "easyweb.install".into(),
            name: "easyweb バンドルインストール".into(),
            description: "domain / dns / vhost / https / redirect / proxy-reload".into(),
            category: "easyweb".into(),
            platform: "vps".into(),
            runnable: true,
        },
        FeatureItem {
            id: "domain.platform.list".into(),
            name: "運用ドメイン一覧".into(),
            description: "platform_domains を表示（実行で JSON 返却）".into(),
            category: "domain".into(),
            platform: "vps".into(),
            runnable: true,
        },
        FeatureItem {
            id: "https.scan".into(),
            name: "HTTPS 監視スキャン".into(),
            description: "全ドメインの HTTP/HTTPS を即時チェック".into(),
            category: "https".into(),
            platform: "vps".into(),
            runnable: true,
        },
        FeatureItem {
            id: "https.certbot".into(),
            name: "HTTPS 手動付与（certbot）".into(),
            description: "通常はドメイン登録で自動 HTTPS。手動再実行用".into(),
            category: "https".into(),
            platform: "vps".into(),
            runnable: true,
        },
        FeatureItem {
            id: "https.certbot_all".into(),
            name: "HTTPS 一括付与".into(),
            description: "auto_https 有効な全ドメインに certbot".into(),
            category: "https".into(),
            platform: "vps".into(),
            runnable: true,
        },
        FeatureItem {
            id: "vhost.regenerate".into(),
            name: "vhost 設定再生成".into(),
            description: "Nginx / Apache 設定ファイルを data/vhost/ に出力".into(),
            category: "vhost".into(),
            platform: "vps".into(),
            runnable: true,
        },
        FeatureItem {
            id: "vhost.install".into(),
            name: "vhost を Web サーバーへ配置".into(),
            description: "data/vhost/ を nginx/apache sites-enabled に symlink".into(),
            category: "vhost".into(),
            platform: "vps".into(),
            runnable: true,
        },
        FeatureItem {
            id: "vhost.reload".into(),
            name: "Web サーバーリロード".into(),
            description: "nginx または apache を reload".into(),
            category: "vhost".into(),
            platform: "vps".into(),
            runnable: true,
        },
        FeatureItem {
            id: "dns.guide".into(),
            name: "DNS 設定ガイド".into(),
            description: "params: zone, vps_ip".into(),
            category: "dns".into(),
            platform: "vps".into(),
            runnable: true,
        },
        FeatureItem {
            id: "recovery.status".into(),
            name: "OS-Recovery 状態".into(),
            description: "aruaru-os-recovery プラグイン状態".into(),
            category: "recovery".into(),
            platform: "both".into(),
            runnable: true,
        },
        FeatureItem {
            id: "plugins.list".into(),
            name: "インストール済みプラグイン".into(),
            description: "プラグイン一覧を返却".into(),
            category: "plugin".into(),
            platform: "vps".into(),
            runnable: true,
        },
    ]
}

pub async fn run_vps_feature(state: SharedState, id: &str, params: &Value) -> anyhow::Result<Value> {
    match id {
        "handoff.export" => {
            let project = params
                .get("project")
                .and_then(|v| v.as_str())
                .unwrap_or("aruaru");
            let to = params
                .get("to")
                .and_then(|v| v.as_str())
                .unwrap_or("cursor");
            let req = handoff::ExportRequest {
                project: project.into(),
                from: "aruaru-web".into(),
                to: to.into(),
                goals: vec![],
                summary: String::new(),
            };
            let p = handoff::export_packet(state.as_ref(), &req, None)?;
            Ok(json!({
                "markdown": handoff::to_markdown(&p),
                "packet": p
            }))
        }
        "ai.recommend" => {
            let uc = params
                .get("use_case")
                .and_then(|v| v.as_str())
                .unwrap_or("development");
            let case = match uc {
                "english_conversation" => ai_providers::RecommendUseCase::EnglishConversation,
                "cheap" => ai_providers::RecommendUseCase::Cheap,
                "learning" => ai_providers::RecommendUseCase::Learning,
                "server_ops" => ai_providers::RecommendUseCase::ServerOps,
                "design" => ai_providers::RecommendUseCase::Design,
                _ => ai_providers::RecommendUseCase::Development,
            };
            Ok(json!({
                "recommended": ai_providers::recommend(case),
                "stack": ai_providers::stack_for_development()
            }))
        }
        "easyweb.install" => {
            let ids = state.install("aruaru-easyweb").await?;
            Ok(json!({ "installed": ids }))
        }
        "domain.platform.list" => {
            let domains = state.list_platform_domains().await?;
            Ok(json!({ "domains": domains, "public_url": state.effective_public_url() }))
        }
        "https.scan" => {
            https_monitor::scan_once(state.clone()).await?;
            Ok(json!({ "message": "HTTPS scan completed" }))
        }
        "https.certbot" => {
            let domain = params
                .get("domain")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("params.domain required"))?;
            let msg = https_cert::configure_domain(&state, domain).await?;
            Ok(json!({ "message": msg }))
        }
        "https.certbot_all" => {
            let results = https_cert::configure_all_pending(&state).await?;
            Ok(json!({ "results": results }))
        }
        "vhost.regenerate" => {
            let path = vhost::regenerate_all(&state).await?;
            Ok(json!({ "path": path.to_string_lossy() }))
        }
        "vhost.install" => {
            let path = vhost::regenerate_all(&state).await?;
            let msg = vhost::install_symlinks(&state)?;
            let reload = vhost::reload_proxy(&state.config.https_monitor.web_server)?;
            Ok(json!({
                "path": path.to_string_lossy(),
                "install": msg,
                "reload": reload
            }))
        }
        "vhost.reload" => {
            let ws = state.config.https_monitor.web_server.clone();
            let msg = vhost::reload_proxy(&ws)?;
            Ok(json!({ "message": msg }))
        }
        "dns.guide" => {
            let zone = params
                .get("zone")
                .and_then(|v| v.as_str())
                .unwrap_or("example.com");
            let ip = params
                .get("vps_ip")
                .and_then(|v| v.as_str())
                .unwrap_or("127.0.0.1");
            let guide = state.dns_guide(zone, ip)?;
            Ok(json!({ "guide": guide }))
        }
        "recovery.status" => {
            let installed = state
                .has_plugin(crate::recovery::PLUGIN_ID)
                .await
                .unwrap_or(false);
            Ok(json!({ "installed": installed, "plugin": crate::recovery::PLUGIN_ID }))
        }
        "plugins.list" => {
            let plugins = state.list_plugins().await?;
            Ok(json!({ "plugins": plugins }))
        }
        _ => anyhow::bail!("unknown feature: {id}"),
    }
}
