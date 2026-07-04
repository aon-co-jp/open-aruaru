//! プロジェクト引き継ぎ — aruaru ↔ Cursor / Claude / Gemini / DeepSeek 双方向

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::ai_providers;
use crate::paths;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffPacket {
    pub version: String,
    pub project: String,
    pub from: String,
    pub to: String,
    pub created_at: String,
    pub stack: serde_json::Value,
    pub data_compat: DataCompat,
    pub summary: String,
    pub goals: Vec<String>,
    pub file_tree: Vec<String>,
    pub rules_excerpt: String,
    pub body_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCompat {
    pub footprint: bool,
    pub data_dir: String,
    pub root_dir: String,
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffTarget {
    Aruaru,
    Cursor,
    Claude,
    Gemini,
    Deepseek,
    Generic,
}

impl HandoffTarget {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Aruaru => "aruaru",
            Self::Cursor => "cursor",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
            Self::Deepseek => "deepseek",
            Self::Generic => "generic",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub project: String,
    #[serde(default = "default_from")]
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub goals: Vec<String>,
    #[serde(default)]
    pub summary: String,
}

fn default_from() -> String {
    "aruaru".into()
}

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub text: String,
    #[serde(default)]
    pub source: String,
}

pub fn export_packet(
    state: &AppState,
    req: &ExportRequest,
    root_override: Option<&Path>,
) -> anyhow::Result<HandoffPacket> {
    let root = root_override
        .map(PathBuf::from)
        .unwrap_or_else(|| state.root_dir.clone());
    let data_dir = paths::display_path(&state.data_dir);
    let root_s = paths::display_path_canonical(&root);

    let to = parse_target(&req.to);
    let file_tree = list_tree(&root, 2);
    let rules = read_rules_excerpt(&root);
    let footprint = root.join(".aruaru-footprint.json").exists()
        || state.data_dir.join(".aruaru-footprint.json").exists();

    let stack = ai_providers::stack_for_development();
    let goals = if req.goals.is_empty() {
        vec![
            "第二の KUSANAGI — VPS 運用".into(),
            "第二の Cursor — Tauri+Poem+Cosmo 開発".into(),
        ]
    } else {
        req.goals.clone()
    };

    let summary = if req.summary.is_empty() {
        format!(
            "aruaru プロジェクト `{}` — desktop / web / smartphone ハイブリッド",
            req.project
        )
    } else {
        req.summary.clone()
    };

    let body = format_body(&to, &req.project, &summary, &goals, &stack, &file_tree, &rules);

    let packet = HandoffPacket {
        version: "1".into(),
        project: req.project.clone(),
        from: req.from.clone(),
        to: to.as_str().into(),
        created_at: chrono::Utc::now().to_rfc3339(),
        stack,
        data_compat: DataCompat {
            footprint,
            data_dir: data_dir.clone(),
            root_dir: root_s,
            platforms: vec![
                "aruaru-desktop".into(),
                "aruaru-web".into(),
                "aruaru-android".into(),
                "aruaru-iPhone".into(),
            ],
        },
        summary,
        goals,
        file_tree,
        rules_excerpt: rules,
        body_markdown: body,
    };

    save_packet(state, &packet)?;
    Ok(packet)
}

pub fn import_text(state: &AppState, req: &ImportRequest) -> anyhow::Result<HandoffPacket> {
    let parsed = parse_import(&req.text)?;
    let dir = state.data_dir.join("handoff").join("imports");
    std::fs::create_dir_all(&dir)?;
    let name = format!(
        "{}-{}.json",
        parsed.project.replace(['/', '\\', ':'], "_"),
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );
    std::fs::write(
        dir.join(&name),
        serde_json::to_string_pretty(&parsed)?,
    )?;
    Ok(parsed)
}

pub fn to_markdown(p: &HandoffPacket) -> String {
    format!(
        r#"---
aruaru-handoff: {}
project: {}
from: {}
to: {}
created_at: {}
data_compat_footprint: {}
---

# aruaru プロジェクト引き継ぎ — {project}

> 第二の KUSANAGI + 第二の Cursor — DATA 互換（footprint）

## 概要
{summary}

## ゴール
{goals}

## スタック
```json
{stack}
```

## DATA 互換
- root: `{root}`
- data: `{data}`
- footprint: {footprint}
- platforms: {platforms}

## ファイル構成（抜粋）
{tree}

## ルール抜粋
{rules}

---

{body}
"#,
        p.version,
        p.project,
        p.from,
        p.to,
        p.created_at,
        p.data_compat.footprint,
        project = p.project,
        summary = p.summary,
        goals = p
            .goals
            .iter()
            .map(|g| format!("- {g}"))
            .collect::<Vec<_>>()
            .join("\n"),
        stack = serde_json::to_string_pretty(&p.stack).unwrap_or_default(),
        root = p.data_compat.root_dir,
        data = p.data_compat.data_dir,
        footprint = p.data_compat.footprint,
        platforms = p.data_compat.platforms.join(", "),
        tree = p
            .file_tree
            .iter()
            .map(|f| format!("- `{f}`"))
            .collect::<Vec<_>>()
            .join("\n"),
        rules = if p.rules_excerpt.is_empty() {
            "（なし）".into()
        } else {
            p.rules_excerpt.clone()
        },
        body = p.body_markdown,
    )
}

fn parse_target(s: &str) -> HandoffTarget {
    match s.to_lowercase().as_str() {
        "aruaru" => HandoffTarget::Aruaru,
        "cursor" => HandoffTarget::Cursor,
        "claude" => HandoffTarget::Claude,
        "gemini" => HandoffTarget::Gemini,
        "deepseek" | "deep seek" => HandoffTarget::Deepseek,
        _ => HandoffTarget::Generic,
    }
}

fn format_body(
    to: &HandoffTarget,
    project: &str,
    summary: &str,
    goals: &[String],
    stack: &serde_json::Value,
    tree: &[String],
    rules: &str,
) -> String {
    let goals_list = goals
        .iter()
        .map(|g| format!("- {g}"))
        .collect::<Vec<_>>()
        .join("\n");

    let tree_list = tree
        .iter()
        .map(|f| format!("- `{f}`"))
        .collect::<Vec<_>>()
        .join("\n");

    match to {
        HandoffTarget::Cursor => format!(
            r#"## Cursor への引き継ぎ指示

このプロジェクトを Cursor で継続開発してください。

### プロジェクト: {project}
{summary}

### 必須スタック
- Tauri (Rust + TypeScript) — aruaru-desktop / aruaru-smartphone
- Poem — aruaru-web (VPS)
- PostgreSQL + aruaru-db DUAL
- WunderGraph Cosmo (OSS)

### ゴール
{goals_list}

### 参照ルール
{rules}

### ディレクトリ
{tree_list}

### 注意
- DATA は C: 以外（E:/F:）に footprint で互換
- VPS は F:/aruaru/deploy/upload-vps.ps1
- ドメイン登録後は自動 HTTPS
"#,
            tree_list = tree_list,
        ),
        HandoffTarget::Claude => format!(
            r#"## Claude プロジェクト引き継ぎ

**プロジェクト名:** {project}

**背景:** {summary}

**技術スタック:** Tauri, Poem, PostgreSQL, aruaru-db, WunderGraph Cosmo

**タスク:**
{goals_list}

**制約:** 第二の KUSANAGI として VPS 運用と DATA ポータビリティを維持すること。

**ファイル構成:**
{tree_list}
"#,
            tree_list = tree_list,
        ),
        HandoffTarget::Gemini => format!(
            r#"## Gemini 引き継ぎ

Project: {project}
Summary: {summary}
Stack: {stack}
Goals:
{goals_list}

Continue development with low-cost API preference where applicable.
"#
        ),
        HandoffTarget::Deepseek => format!(
            r#"## DeepSeek 引き継ぎ（低単価コーディング向け）

Project: {project}
{summary}

Stack: Rust/Poem/TypeScript/Tauri/PostgreSQL/Cosmo

Tasks:
{goals_list}

Files:
{tree_list}
"#,
            tree_list = tree_list,
        ),
        HandoffTarget::Aruaru | HandoffTarget::Generic => format!(
            r#"## aruaru 引き継ぎ

{summary}

### ゴール
{goals_list}

### スタック
{stack}
"#
        ),
    }
}

fn list_tree(root: &Path, depth: u32) -> Vec<String> {
    let mut out = Vec::new();
    if depth == 0 || !root.is_dir() {
        return out;
    }
    let root_display = paths::display_path_canonical(root);
    let Ok(entries) = std::fs::read_dir(root) else {
        return out;
    };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if ["node_modules", "target", ".git", "dist"].contains(&name.as_str()) {
            continue;
        }
        out.push(format!("{root_display}/{name}"));
        if e.path().is_dir() && depth > 1 {
            for sub in list_tree(&e.path(), depth - 1) {
                out.push(sub);
            }
        }
    }
    out.sort();
    out.truncate(40);
    out
}

fn read_rules_excerpt(root: &Path) -> String {
    let rules_dir = root.join(".cursor").join("rules");
    if !rules_dir.is_dir() {
        return String::new();
    }
    let mut parts = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&rules_dir) {
        for e in entries.flatten().take(5) {
            if e.path().extension().and_then(|x| x.to_str()) == Some("mdc") {
                if let Ok(raw) = std::fs::read_to_string(e.path()) {
                    let excerpt: String = raw.chars().take(800).collect();
                    parts.push(format!("### {}\n{}", e.file_name().to_string_lossy(), excerpt));
                }
            }
        }
    }
    parts.join("\n\n")
}

fn parse_import(text: &str) -> anyhow::Result<HandoffPacket> {
    if text.trim_start().starts_with('{') {
        return Ok(serde_json::from_str(text)?);
    }

    // YAML front matter 簡易パース
    let mut project = "imported".into();
    let mut from = "external".into();
    let mut to = "aruaru".into();
    if text.starts_with("---") {
        if let Some(end) = text[3..].find("---") {
            let fm = &text[3..3 + end];
            for line in fm.lines() {
                if let Some((k, v)) = line.split_once(':') {
                    match k.trim() {
                        "project" => project = v.trim().into(),
                        "from" => from = v.trim().into(),
                        "to" => to = v.trim().into(),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(HandoffPacket {
        version: "1".into(),
        project,
        from,
        to,
        created_at: chrono::Utc::now().to_rfc3339(),
        stack: ai_providers::stack_for_development(),
        data_compat: DataCompat {
            footprint: false,
            data_dir: String::new(),
            root_dir: String::new(),
            platforms: vec!["aruaru-web".into()],
        },
        summary: "Imported handoff document".into(),
        goals: vec!["継続開発".into()],
        file_tree: vec![],
        rules_excerpt: String::new(),
        body_markdown: text.to_string(),
    })
}

fn save_packet(state: &AppState, packet: &HandoffPacket) -> anyhow::Result<()> {
    let dir = state.data_dir.join("handoff").join("exports");
    std::fs::create_dir_all(&dir)?;
    let stem = packet.project.replace(['/', '\\', ':'], "_");
    std::fs::write(
        dir.join(format!("{stem}.json")),
        serde_json::to_string_pretty(packet)?,
    )?;
    std::fs::write(dir.join(format!("{stem}.md")), to_markdown(packet))?;
    Ok(())
}

pub fn handoff_targets() -> serde_json::Value {
    json!({
        "targets": [
            { "id": "aruaru", "name": "aruaru（戻す）", "direction": "both" },
            { "id": "cursor", "name": "Cursor", "direction": "both" },
            { "id": "claude", "name": "Claude", "direction": "both" },
            { "id": "gemini", "name": "Gemini", "direction": "both" },
            { "id": "deepseek", "name": "DeepSeek", "direction": "both" }
        ],
        "note": "Markdown または JSON でコピー＆ペースト引き継ぎ。DATA footprint 互換。パスは F:/aruaru/... 形式（ドライブ名から）で表示。"
    })
}
