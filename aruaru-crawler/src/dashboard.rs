use poem::handler;
use poem::web::{Data, Html};

use crate::watcher::SharedStatus;

#[handler]
pub async fn dashboard(status: Data<&SharedStatus>) -> Html<String> {
    let locked = status.lock().await;
    let generated = list_items(&locked.last_generated);
    let errors = list_items(&locked.last_errors);
    let last_scan = locked.last_scan_at.as_deref().unwrap_or("not scanned yet");

    Html(format!(
        r#"<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>aruaru README Auto RS</title>
<style>
body {{ margin: 0; font-family: system-ui, sans-serif; background: #f6f7fb; color: #172033; line-height: 1.7; }}
main {{ width: min(calc(100% - 2rem), 1100px); margin: 2rem auto; }}
.card {{ background: white; border: 1px solid #dfe4ef; border-radius: 20px; padding: clamp(1rem, 3vw, 2rem); box-shadow: 0 18px 50px rgba(20,32,56,.10); }}
.grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem; }}
.stat {{ border: 1px solid #dfe4ef; border-radius: 16px; padding: 1rem; }}
.stat b {{ display: block; font-size: 1.8rem; }}
code {{ background: #eef2ff; padding: .1rem .35rem; border-radius: .4rem; }}
ul {{ padding-left: 1.2rem; }}
</style>
</head>
<body>
<main>
  <section class="card">
    <h1>aruaru README Auto RS</h1>
    <p>FTPなどで配置された <code>README.md</code> を監視して、同じフォルダに <code>README.rs</code> を生成します。</p>
    <p>Root: <code>{root}</code><br>Output: <code>{output}</code><br>Last scan: <code>{last_scan}</code></p>
    <div class="grid">
      <div class="stat"><span>Generated</span><b>{generated_count}</b></div>
      <div class="stat"><span>Skipped</span><b>{skipped_count}</b></div>
      <div class="stat"><span>Errors</span><b>{error_count}</b></div>
    </div>
    <h2>最近の生成</h2>
    <ul>{generated}</ul>
    <h2>最近のエラー</h2>
    <ul>{errors}</ul>
  </section>
</main>
</body>
</html>"#,
        root = escape_html(&locked.root),
        output = escape_html(&locked.output_mode),
        last_scan = escape_html(last_scan),
        generated_count = locked.generated_count,
        skipped_count = locked.skipped_count,
        error_count = locked.error_count,
    ))
}

fn list_items(items: &[String]) -> String {
    if items.is_empty() {
        return "<li>なし</li>".to_string();
    }
    items
        .iter()
        .map(|item| format!("<li><code>{}</code></li>", escape_html(item)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
