use poem::{
    handler,
    http::StatusCode,
    web::{Data, Query},
    Result,
};
use poem::session::Session;

use crate::auth;
use crate::session::{email as session_email, set_email as set_session};
use crate::state::SharedState;

fn layout(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ja"><head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} — aruaru-easyweb</title>
<link rel="stylesheet" href="/static/css/aruaru.css">
</head><body>
<header class="header"><div class="header-inner">
<a href="/" class="logo">aruaru <span class="ver">easyweb v0.1</span></a>
<nav><a href="/">ホーム</a><a href="/settings/domain">aruaruドメイン</a><a href="/settings/easyweb">easyweb</a><a href="/settings/https">HTTPS</a><a href="/settings/server">サーバー</a><a href="/settings/webserver">Webサーバー選択</a><a href="/settings/github">📦 GitHub</a><a href="/tools/webserver">🔧 設定生成（公開）</a></nav>
</div></header>
<main class="main">{body}</main>
<script src="/static/js/aruaru.js"></script>
</body></html>"#
    )
}

#[handler]
pub async fn index(state: Data<&SharedState>, session: &Session) -> Result<impl poem::IntoResponse> {
    if session_email(session).is_some() {
        return Ok(poem::Response::builder().status(StatusCode::FOUND).header("Location", "/dashboard").finish());
    }
    if !auth::has_admin(&state).await {
        return Ok(poem::Response::builder().status(StatusCode::FOUND).header("Location", "/register").finish());
    }
    Ok(poem::Response::builder().status(StatusCode::FOUND).header("Location", "/login").finish())
}

#[handler]
pub async fn register_page() -> Result<impl poem::IntoResponse> {
    let body = r#"
<section class="hero"><h1>メール登録</h1>
<p>メールアドレスを ID として登録します。パスワードは不要 — ログインリンクをメール（または画面）でお届けします。</p></section>
<section class="card">
<form id="register-form" class="form-grid">
<label>メールアドレス<input type="email" name="email" required placeholder="you@example.com"></label>
<button type="submit" class="btn btn-primary">登録してログインリンクを受け取る</button>
</form>
<div id="register-result" class="result"></div>
<p class="hint">SMTP 未設定時は、ログインリンクが画面に表示されます。ブックマークしてください。</p>
</section>"#;
    html("メール登録", body)
}

#[handler]
pub async fn login_page() -> Result<impl poem::IntoResponse> {
    let body = r#"
<section class="hero"><h1>ログイン</h1>
<p>登録メール宛にログインリンクを送信します（パスワード不要）</p></section>
<section class="card">
<form id="login-form" class="form-grid">
<label>メール<input type="email" name="email" required></label>
<button type="submit" class="btn btn-primary">ログインリンクを送る</button>
</form>
<div id="login-result" class="result"></div>
</section>"#;
    html("ログイン", body)
}

#[derive(serde::Deserialize)]
struct TokenQuery {
    token: String,
}

#[handler]
pub async fn verify(
    state: Data<&SharedState>,
    session: &Session,
    Query(q): Query<TokenQuery>,
) -> Result<impl poem::IntoResponse> {
    match auth::verify_token(&state, &q.token).await {
        Ok(email) => {
            set_session(session, &email);
            Ok(poem::Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/settings/domain")
                .finish())
        }
        Err(e) => html("エラー", &format!(r#"<section class="card"><p class="result err">{e}</p></section>"#)),
    }
}

#[handler]
pub async fn dashboard(state: Data<&SharedState>, session: &Session) -> Result<impl poem::IntoResponse> {
    let Some(email) = session_email(session) else {
        return Ok(poem::Response::builder().status(StatusCode::FOUND).header("Location", "/login").finish());
    };
    let primary = state.primary_platform_domain().await.ok().flatten();
    let primary_name = primary.as_ref().map(|d| d.name.as_str()).unwrap_or("（未設定 — IP:3000）");
    let plugins = state.list_plugins().await.unwrap_or_default();
    let rows: String = plugins.iter().map(|p| format!("<tr><td>{}</td><td>{}</td></tr>", p.id, p.version)).collect();
    let body = format!(
        r#"<section class="hero"><h1>ダッシュボード</h1><p>ログイン: {email}</p>
<p>運用URL: <code>{public_url}</code> · 主ドメイン: <strong>{primary_name}</strong></p></section>
<div id="reminder-box" class="reminder-banner hidden"></div>

<section class="card">
  <h2>サーバー制御</h2>
  <div class="status-row">
    <span id="sv-aruaru" class="badge">aruaru: 確認中…</span>
    <span id="sv-nginx"  class="badge badge-secondary" style="margin-left:8px"></span>
    <span id="sv-apache" class="badge badge-secondary" style="margin-left:8px"></span>
  </div>
  <div class="btn-group" style="margin-top:10px">
    <button class="btn btn-primary"      onclick="sctl('aruaru','start')">▶ 起動</button>
    <button class="btn btn-warning"      onclick="sctl('aruaru','stop')">■ 停止</button>
    <button class="btn btn-secondary"    onclick="sctl('aruaru','restart')">↺ 再起動</button>
    <button class="btn btn-sm"           onclick="sctl('aruaru','enable')">常駐 ON</button>
    <button class="btn btn-sm btn-danger" onclick="sctl('aruaru','disable')">常駐 OFF</button>
  </div>
  <div class="btn-group" style="margin-top:6px">
    <small>Nginx: </small>
    <button class="btn btn-sm" onclick="sctl('nginx','start')">▶</button>
    <button class="btn btn-sm" onclick="sctl('nginx','stop')">■</button>
    <button class="btn btn-sm" onclick="sctl('nginx','reload')">↺ リロード</button>
    <small style="margin-left:8px">Apache: </small>
    <button class="btn btn-sm" onclick="sctl('apache2','start')">▶</button>
    <button class="btn btn-sm" onclick="sctl('apache2','stop')">■</button>
    <button class="btn btn-sm" onclick="sctl('apache2','reload')">↺ リロード</button>
  </div>
  <p class="hint"><a href="/settings/server">詳細なサーバー状態 →</a></p>
  <div id="sctl-result" class="result" style="margin-top:8px"></div>
</section>

<section class="card"><h2>プラグイン</h2>
<table class="table"><thead><tr><th>ID</th><th>Ver</th></tr></thead>
<tbody>{rows}</tbody></table>
<p><a href="/settings/domain">aruaru ドメイン設定 →</a></p></section>

<script>
async function loadSvStatus() {{
  try {{
    const d = await fetch('/api/server/status').then(r => r.json());
    const sv = d.services || {{}};
    const a = sv.aruaru || '不明';
    const el = document.getElementById('sv-aruaru');
    el.textContent = 'aruaru: ' + a;
    el.className = 'badge ' + (a === 'active' ? 'badge-ok' : 'badge-warn');
    const ng = document.getElementById('sv-nginx');
    if (ng && sv.nginx !== undefined && sv.nginx !== null) {{
      ng.textContent = 'Nginx: ' + (sv.nginx ? 'active' : '停止');
      ng.className = 'badge ' + (sv.nginx ? 'badge-ok' : 'badge-warn');
    }}
    const ap = document.getElementById('sv-apache');
    if (ap && sv.apache !== undefined && sv.apache !== null) {{
      ap.textContent = 'Apache: ' + (sv.apache ? 'active' : '停止');
      ap.className = 'badge ' + (sv.apache ? 'badge-ok' : 'badge-warn');
    }}
  }} catch(e) {{ document.getElementById('sv-aruaru').textContent = '取得失敗'; }}
}}
async function sctl(unit, action) {{
  const res = document.getElementById('sctl-result');
  res.textContent = unit + ' ' + action + ' 実行中…';
  res.className = 'result';
  try {{
    const d = await fetch('/api/server/control', {{
      method: 'POST',
      headers: {{'Content-Type': 'application/json'}},
      body: JSON.stringify({{ unit, action }})
    }}).then(r => r.json());
    res.textContent = d.ok ? '✓ ' + (d.message || 'OK') : '✗ ' + (d.error || 'エラー');
    res.className = 'result ' + (d.ok ? 'result-ok' : 'result-error');
    setTimeout(loadSvStatus, 1500);
  }} catch(e) {{ res.textContent = '通信エラー: ' + e; res.className = 'result result-error'; }}
}}
loadSvStatus();
setInterval(loadSvStatus, 15000);
</script>"#,
        public_url = state.effective_public_url(),
        rows = if rows.is_empty() { "<tr><td colspan=2>未インストール</td></tr>".into() } else { rows }
    );
    html("ダッシュボード", &body)
}

#[handler]
pub async fn domain_settings(state: Data<&SharedState>, session: &Session) -> Result<impl poem::IntoResponse> {
    if session_email(session).is_none() {
        return Ok(poem::Response::builder().status(StatusCode::FOUND).header("Location", "/login").finish());
    }
    let domains = state.list_platform_domains().await.unwrap_or_default();
    let rows: String = domains
        .iter()
        .map(|d| {
            let badge = if d.is_primary != 0 { " <span class='badge'>主</span>" } else { "" };
            format!(
                "<tr data-name=\"{name}\"><td>{name}{badge}</td><td>{}</td><td>HTTPS:{}</td><td class=\"actions\">
<button class=\"btn btn-sm set-primary\" data-name=\"{name}\">主にする</button>
<button class=\"btn btn-sm delete-domain\" data-name=\"{name}\">削除</button></td></tr>",
                d.last_checked.as_deref().unwrap_or("-"),
                if d.https_enabled != 0 { "ON" } else { "OFF" },
                name = d.name,
                badge = badge
            )
        })
        .collect();

    let body = format!(
        r#"<section class="hero"><h1>aruaru 運用ドメイン</h1>
<p>aruaru 本体（easyweb 含む）を公開するドメイン。登録・削除・入れ替えが可能です。</p>
<p>現在の URL: <code>{public_url}</code></p></section>
<section class="card"><h2>ドメイン登録</h2>
<form id="platform-domain-form" class="form-grid">
<label>ドメイン<input name="name" placeholder="aruaru.example.com" required></label>
<button type="submit" class="btn btn-primary">登録</button>
</form>
<div id="platform-domain-result" class="result"></div>
<p class="hint">DNS で VPS の IP を向けてから登録。初回登録が自動的に「主ドメイン」になります。</p>
</section>
<section class="card"><h2>登録済みドメイン</h2>
<table class="table" id="platform-domains-table">
<thead><tr><th>ドメイン</th><th>最終確認</th><th>HTTPS</th><th>操作</th></tr></thead>
<tbody>{rows}</tbody></table>
</section>
<section class="card"><h2>ドメイン入れ替え</h2>
<form id="swap-domain-form" class="form-grid">
<label>現在の主<input name="from" placeholder="old.example.com"></label>
<label>新しい主<input name="to" placeholder="new.example.com"></label>
<button type="submit" class="btn btn-primary">入れ替え</button>
</form>
<p class="hint">主ドメインを切り替えると public_url とログインリンクのベース URL が更新されます。</p>
</section>"#,
        public_url = state.effective_public_url(),
        rows = if rows.is_empty() {
            "<tr><td colspan=4>未登録 — IP (<code>http://IP:3000</code>) で運用中</td></tr>".into()
        } else {
            rows
        }
    );
    html("aruaruドメイン", &body)
}

#[handler]
pub async fn easyweb_settings(state: Data<&SharedState>, session: &Session) -> Result<impl poem::IntoResponse> {
    let Some(_email) = session_email(session) else {
        return Ok(poem::Response::builder().status(StatusCode::FOUND).header("Location", "/login").finish());
    };
    let domains = state.list_domains().await.unwrap_or_default();
    let rows: String = domains.iter().map(|d| format!(
        "<tr><td>{}</td><td>{}</td><td>HTTPS:{}</td></tr>",
        d.name, d.last_checked.as_deref().unwrap_or("-"), if d.https_enabled != 0 { "ON" } else { "OFF" }
    )).collect();
    let body = format!(
        r#"<section class="hero"><h1>aruaru-easyweb 設定</h1>
<p>ホストするサイト用ドメイン（aruaru 本体の運用ドメインは <a href="/settings/domain">こちら</a>）</p></section>
<section class="card"><h2>ドメイン追加</h2>
<form id="domain-form" class="form-grid">
<label>ドメイン<input name="name" placeholder="example.com" required>
<button type="submit" class="btn btn-primary">追加</button></label>
</form></section>
<section class="card"><h2>ドメイン一覧</h2>
<table class="table"><thead><tr><th>ドメイン</th><th>最終確認</th><th>HTTPS</th></tr></thead>
<tbody>{rows}</tbody></table></section>"#,
        rows = if rows.is_empty() { "<tr><td colspan=3>ドメイン未登録</td></tr>".into() } else { rows }
    );
    html("easyweb設定", &body)
}

#[handler]
pub async fn https_settings(session: &Session) -> Result<impl poem::IntoResponse> {
    if session_email(session).is_none() {
        return Ok(poem::Response::builder().status(StatusCode::FOUND).header("Location", "/login").finish());
    }
    let body = r#"
<section class="hero"><h1>aruaru-https</h1>
<p>HTTP（sなし）を自動監視し、HTTPS + Let's Encrypt を自動設定（v0.1: キュー登録まで）</p></section>
<section class="card">
<ul>
<li>監視間隔: config の scan_interval_secs（デフォルト 5分）</li>
<li>初心者もベテランも「https の s」を意識不要 — 自動管理</li>
<li>証明書更新も aruaru-https が担当（v0.2 で certbot 連携）</li>
</ul>
</section>"#;
    html("HTTPS設定", body)
}

#[handler]
pub async fn server_status_page(state: Data<&SharedState>, session: &Session) -> Result<impl poem::IntoResponse> {
    if session_email(session).is_none() {
        return Ok(poem::Response::builder().status(StatusCode::FOUND).header("Location", "/login").finish());
    }
    let body = format!(
        r#"<section class="hero">
  <h1>サーバー状態</h1>
  <p>Webサーバー（Nginx / Apache）と aruaru サービスの状態をリアルタイムで確認・操作できます。</p>
  <p>運用URL: <code>{public_url}</code></p>
</section>

<section class="card">
  <h2>サービス状態 <button class="btn btn-sm" onclick="loadStatus()" style="margin-left:8px">↺ 更新</button></h2>
  <table class="table" id="status-table">
    <thead><tr><th>サービス</th><th>状態</th><th>操作</th></tr></thead>
    <tbody id="status-tbody">
      <tr><td colspan="3">読み込み中…</td></tr>
    </tbody>
  </table>
  <div id="sctl-result" class="result" style="margin-top:8px"></div>
</section>

<section class="card">
  <h2>リソース</h2>
  <div id="resource-info">読み込み中…</div>
</section>

<script>
const UNITS = [
  {{ id: 'aruaru', label: 'aruaru',  key: 'aruaru',  type: 'string' }},
  {{ id: 'nginx',  label: 'Nginx',   key: 'nginx',   type: 'bool' }},
  {{ id: 'apache2',label: 'Apache',  key: 'apache',  type: 'bool' }},
];

async function loadStatus() {{
  try {{
    const d = await fetch('/api/server/status').then(r => r.json());
    const sv = d.services || {{}};
    const tbody = document.getElementById('status-tbody');
    tbody.innerHTML = UNITS.map(u => {{
      let raw = sv[u.key];
      let isActive = u.type === 'bool' ? raw === true : raw === 'active';
      let label = raw === undefined || raw === null ? '対象外' : (u.type === 'bool' ? (raw ? 'active' : '停止') : raw);
      let badge = raw === undefined || raw === null ? 'badge-secondary'
                  : isActive ? 'badge-ok' : 'badge-warn';
      let btns = raw !== undefined && raw !== null ? `
        <button class="btn btn-sm btn-primary" onclick="sctl('${{u.id}}','start')">▶ 起動</button>
        <button class="btn btn-sm btn-warning" onclick="sctl('${{u.id}}','stop')">■ 停止</button>
        <button class="btn btn-sm" onclick="sctl('${{u.id}}','restart')">↺ 再起動</button>
        <button class="btn btn-sm" onclick="sctl('${{u.id}}','reload')">⟳ リロード</button>
        ${{u.id === 'aruaru' ? `<button class="btn btn-sm" onclick="sctl('aruaru','enable')">常駐ON</button>
        <button class="btn btn-sm btn-danger" onclick="sctl('aruaru','disable')">常駐OFF</button>` : ''}}
      ` : '<span class="hint">Linux VPS 上でのみ操作可</span>';
      return `<tr>
        <td><strong>${{u.label}}</strong></td>
        <td><span class="badge ${{badge}}">${{label}}</span></td>
        <td>${{btns}}</td>
      </tr>`;
    }}).join('');

    const ri = document.getElementById('resource-info');
    const pct = v => v !== undefined && v !== null ? v + ' %' : '—';
    const uptime = d.uptime_secs ? Math.floor(d.uptime_secs / 3600) + ' 時間' : '—';
    ri.innerHTML = `<table class="table">
      <tr><th>ホスト名</th><td>${{d.hostname || '—'}}</td></tr>
      <tr><th>CPU</th><td>${{pct(d.cpu_percent)}}</td></tr>
      <tr><th>メモリ</th><td>${{pct(d.memory_percent)}}</td></tr>
      <tr><th>ディスク</th><td>${{pct(d.disk_percent)}}</td></tr>
      <tr><th>稼働時間</th><td>${{uptime}}</td></tr>
      <tr><th>確認日時</th><td>${{d.checked_at || '—'}}</td></tr>
    </table>`;
  }} catch(e) {{
    document.getElementById('status-tbody').innerHTML = `<tr><td colspan="3" class="result-error">取得失敗: ${{e}}</td></tr>`;
  }}
}}

async function sctl(unit, action) {{
  const res = document.getElementById('sctl-result');
  res.textContent = unit + ' ' + action + ' 実行中…';
  res.className = 'result';
  try {{
    const d = await fetch('/api/server/control', {{
      method: 'POST',
      headers: {{'Content-Type': 'application/json'}},
      body: JSON.stringify({{ unit, action }})
    }}).then(r => r.json());
    res.textContent = d.ok ? '✓ ' + (d.message || 'OK') : '✗ ' + (d.error || 'エラー');
    res.className = 'result ' + (d.ok ? 'result-ok' : 'result-error');
    setTimeout(loadStatus, 1500);
  }} catch(e) {{
    res.textContent = '通信エラー: ' + e;
    res.className = 'result result-error';
  }}
}}

loadStatus();
setInterval(loadStatus, 15000);
</script>"#,
        public_url = state.effective_public_url(),
    );
    html("サーバー状態", &body)
}

#[handler]
pub async fn webserver_settings(session: &Session) -> Result<impl poem::IntoResponse> {
    if session_email(session).is_none() {
        return Ok(poem::Response::builder().status(StatusCode::FOUND).header("Location", "/login").finish());
    }
    let body = r#"
<section class="hero">
  <h2 id="page-title">Webサーバー選択 &amp; AI自動設定</h2>
  <p id="page-desc">プログラム言語・フレームワークを選ぶと、AI が Nginx / Apache のどちらが最適かを推薦します。そのまま設定ファイルを自動生成できます。</p>
  <div style="text-align:right;margin-top:4px">
    <button class="btn btn-sm" onclick="setLang('ja')" id="btn-ja">🇯🇵 日本語</button>
    <button class="btn btn-sm" onclick="setLang('en')" id="btn-en">🇺🇸 English</button>
  </div>
</section>

<!-- ① 言語 / FW 選択 -->
<section class="card">
  <h2 id="h-select">① 言語・フレームワークを選択</h2>
  <div id="lang-grid" class="lang-grid"></div>
</section>

<!-- ② AI 推薦結果 -->
<section class="card" id="advisor-section" style="display:none">
  <h2 id="h-advisor">② AI 推薦</h2>
  <div id="advisor-result"></div>
</section>

<!-- ③ Webサーバー選択 & 設定生成 -->
<section class="card" id="config-section" style="display:none">
  <h2 id="h-config">③ 設定ファイル自動生成</h2>
  <form id="config-form" class="form-grid">
    <label id="lbl-server">Webサーバー
      <select name="server" id="sel-server">
        <option value="nginx">Nginx（推薦）</option>
        <option value="apache">Apache</option>
      </select>
    </label>
    <label id="lbl-domain">ドメイン<input name="domain" placeholder="example.com" required></label>
    <label id="lbl-port">アプリポート<input name="port" type="number" value="3000" min="1" max="65535" required></label>
    <button type="submit" class="btn btn-primary" id="btn-gen">設定ファイルを生成</button>
  </form>
  <div id="config-result" class="result"></div>
</section>

<!-- ④ Nginx / Apache 詳細比較 -->
<section class="card">
  <h2 id="h-compare">Nginx vs Apache 比較</h2>
  <div id="compare-content">読み込み中…</div>
</section>

<style>
.lang-grid { display:grid; grid-template-columns:repeat(auto-fill,minmax(180px,1fr)); gap:8px; margin-top:8px; }
.lang-btn { padding:10px 12px; border:1px solid var(--border,#ddd); border-radius:6px; background:var(--card-bg,#fff);
  cursor:pointer; text-align:left; transition:border-color .15s,box-shadow .15s; }
.lang-btn:hover, .lang-btn.active { border-color:var(--accent,#2563eb); box-shadow:0 0 0 2px rgba(37,99,235,.2); }
.rec-box { display:grid; grid-template-columns:1fr 1fr; gap:12px; margin-top:12px; }
.rec-card { border:1px solid var(--border,#ddd); border-radius:8px; padding:14px; }
.rec-card.winner { border-color:#22c55e; background:rgba(34,197,94,.06); }
.compat-badge { display:inline-block; padding:2px 8px; border-radius:4px; font-size:.8em; font-weight:bold; }
.compat-excellent { background:#dcfce7; color:#15803d; }
.compat-good      { background:#dbeafe; color:#1d4ed8; }
.compat-fair      { background:#fef9c3; color:#854d0e; }
.compat-limited   { background:#fee2e2; color:#b91c1c; }
.compare-grid { display:grid; grid-template-columns:1fr 1fr; gap:16px; margin-top:12px; }
.compare-col { border:1px solid var(--border,#ddd); border-radius:8px; padding:16px; }
.compare-col h3 { margin:0 0 8px; }
.compare-col ul { margin:4px 0; padding-left:18px; }
.code-block { background:var(--code-bg,#f4f4f4); border-radius:6px; padding:12px;
  font-family:monospace; font-size:.8em; overflow:auto; white-space:pre; max-height:260px; }
</style>

<script>
// ─── 多言語 ───────────────────────────────────────────────
const T = {
  ja: {
    pageTitle: 'Webサーバー選択 &amp; AI自動設定',
    pageDesc:  'プログラム言語・フレームワークを選ぶと、AIがNginx / Apacheの最適な方を推薦します。',
    hSelect:  '① 言語・フレームワークを選択',
    hAdvisor: '② AI 推薦',
    hConfig:  '③ 設定ファイル自動生成',
    hCompare: 'Nginx vs Apache 比較',
    lblServer:'Webサーバー', lblDomain:'ドメイン', lblPort:'アプリポート',
    btnGen:   '設定ファイルを生成',
    recommended: '推薦', reason: '推薦理由', compat: '相性', tips: 'ヒント',
    strengths:'強み', weaknesses:'弱点', configOps:'設定ファイル運用',
    difficulty:'設定難易度', configEx:'設定例',
    loading: '読み込み中…',
  },
  en: {
    pageTitle: 'Web Server Selection &amp; AI Auto-Config',
    pageDesc:  'Choose your language / framework and AI will recommend Nginx or Apache.',
    hSelect:  '① Select Language / Framework',
    hAdvisor: '② AI Recommendation',
    hConfig:  '③ Auto-generate Config File',
    hCompare: 'Nginx vs Apache Comparison',
    lblServer:'Web Server', lblDomain:'Domain', lblPort:'App Port',
    btnGen:   'Generate Config',
    recommended: 'Recommended', reason: 'Reason', compat: 'Compatibility', tips: 'Tips',
    strengths:'Strengths', weaknesses:'Weaknesses', configOps:'Config File Operations',
    difficulty:'Config Difficulty', configEx:'Config Example',
    loading: 'Loading…',
  }
};
let LANG = 'ja';
let compareData = null;
let currentLang = null;

function t(k) { return T[LANG][k] || k; }

function setLang(l) {
  LANG = l;
  document.getElementById('btn-ja').style.fontWeight = l==='ja'?'bold':'normal';
  document.getElementById('btn-en').style.fontWeight = l==='en'?'bold':'normal';
  applyTranslations();
  if (compareData) renderCompare(compareData);
  if (currentLang) showAdvisor(currentLang, false);
}

function applyTranslations() {
  ['pageTitle','pageDesc','hSelect','hAdvisor','hConfig','hCompare',
   'lblServer','lblDomain','lblPort','btnGen'].forEach(k => {
    const el = document.getElementById(k.replace(/^(lbl|btn|h|page)/, m => ({
      lbl:'lbl-',btn:'btn-',h:'h-',page:'page-'
    })[m] || m));
    if (el) { if (el.tagName==='LABEL') el.childNodes[0].textContent = t(k)+'\n';
              else el.innerHTML = t(k); }
  });
  renderLangGrid(compareData);
}

// ─── ① 言語グリッド ──────────────────────────────────────
function renderLangGrid(data) {
  const g = document.getElementById('lang-grid');
  if (!data) { g.innerHTML = '<p>' + t('loading') + '</p>'; return; }
  g.innerHTML = data.lang_frameworks.map(e =>
    `<button class="lang-btn${currentLang===e.id?' active':''}" onclick="showAdvisor('${e.id}',true)">
      <strong>${LANG==='ja' ? e.label_ja : e.label_en}</strong><br>
      <small>${e.recommended_server==='nginx'?'→ Nginx推薦':'→ Apache推薦'}</small>
    </button>`
  ).join('');
}

// ─── ② AI 推薦 ──────────────────────────────────────────
async function showAdvisor(langId, scroll) {
  currentLang = langId;
  document.querySelectorAll('.lang-btn').forEach(b => b.classList.remove('active'));
  document.querySelectorAll('.lang-btn').forEach(b => {
    if (b.getAttribute('onclick') && b.getAttribute('onclick').includes("'"+langId+"'"))
      b.classList.add('active');
  });

  const port = document.querySelector('[name=port]')?.value || 3000;
  const res = await fetch(`/api/webserver/advisor?lang=${langId}&port=${port}`).then(r=>r.json());
  if (!res.ok) return;
  const r = res.recommendation;

  const recName   = r.recommended === 'nginx' ? 'Nginx' : 'Apache';
  const otherName = r.recommended === 'nginx' ? 'Apache' : 'Nginx';
  const reason    = LANG==='ja' ? r.reason_ja   : r.reason_en;
  const tips      = LANG==='ja' ? r.tips_ja     : r.tips_en;
  const ngCompat  = LANG==='ja' ? r.nginx_compat_ja  : r.nginx_compat_en;
  const apCompat  = LANG==='ja' ? r.apache_compat_ja : r.apache_compat_en;

  function compatClass(s) {
    if (s.includes('最適')||s==='Excellent') return 'compat-excellent';
    if (s.includes('良好')||s==='Good')      return 'compat-good';
    if (s.includes('可')||s==='Fair')        return 'compat-fair';
    return 'compat-limited';
  }

  document.getElementById('advisor-result').innerHTML = `
    <div class="rec-box">
      <div class="rec-card winner">
        <h3>✅ ${recName} <span style="font-size:.75em;color:#555">(${t('recommended')})</span></h3>
        <p><strong>${t('reason')}:</strong> ${reason}</p>
        <p><span class="compat-badge ${compatClass(r.recommended==='nginx'?ngCompat:apCompat)}">${r.recommended==='nginx'?ngCompat:apCompat}</span></p>
      </div>
      <div class="rec-card">
        <h3>${otherName}</h3>
        <p><span class="compat-badge ${compatClass(r.recommended==='nginx'?apCompat:ngCompat)}">${r.recommended==='nginx'?apCompat:ngCompat}</span></p>
      </div>
    </div>
    <p style="margin-top:12px"><strong>${t('tips')}:</strong> ${tips}</p>
    <details style="margin-top:8px"><summary>${LANG==='ja'?'設定プレビュー':'Config Preview'}</summary>
      <div class="code-block">${escHtml(r.config_preview)}</div>
    </details>`;

  // 推薦サーバーをセレクトに反映
  document.getElementById('sel-server').value = r.recommended;
  document.getElementById('advisor-section').style.display = '';
  document.getElementById('config-section').style.display = '';
  if (scroll) document.getElementById('advisor-section').scrollIntoView({behavior:'smooth'});
}

// ─── ③ 設定生成 ──────────────────────────────────────────
document.getElementById('config-form').addEventListener('submit', async e => {
  e.preventDefault();
  const fd = new FormData(e.target);
  const body = { server: fd.get('server'), domain: fd.get('domain'),
                 port: Number(fd.get('port')), lang: currentLang || 'generic' };
  const res = document.getElementById('config-result');
  res.textContent = LANG==='ja'?'生成中…':'Generating…';
  try {
    const d = await fetch('/api/webserver/select', {
      method:'POST', headers:{'Content-Type':'application/json'},
      body: JSON.stringify(body)
    }).then(r=>r.json());
    if (d.ok) {
      const msg   = LANG==='ja' ? d.message_ja   : d.message_en;
      const next  = LANG==='ja' ? d.next_step_ja : d.next_step_en;
      res.innerHTML = `<span class="result-ok">✓ ${escHtml(msg)}</span><br><small>${escHtml(next)}</small>`;
    } else {
      res.innerHTML = `<span class="result-error">✗ ${escHtml(d.error)}</span>`;
    }
    res.className = 'result';
  } catch(err) {
    res.textContent = (LANG==='ja'?'通信エラー: ':'Network error: ') + err;
    res.className = 'result result-error';
  }
});

// ─── ④ 比較 ─────────────────────────────────────────────
function compatBadge(s) {
  const cls = s.includes('最適')||s==='Excellent'?'compat-excellent'
            : s.includes('良好')||s==='Good'?'compat-good'
            : s.includes('可')||s==='Fair'?'compat-fair':'compat-limited';
  return `<span class="compat-badge ${cls}">${s}</span>`;
}

function renderCompare(data) {
  const n = data.nginx;
  const a = data.apache;
  function col(info, isNginx) {
    const sl = LANG==='ja';
    const strengths  = (sl ? info.strengths_ja  : info.strengths_en ).map(s=>`<li>${s}</li>`).join('');
    const weaknesses = (sl ? info.weaknesses_ja : info.weaknesses_en).map(s=>`<li>${s}</li>`).join('');
    return `<div class="compare-col">
      <h3>${info.name}</h3>
      <p>${sl ? info.summary_ja : info.summary_en}</p>
      <p><strong>${t('difficulty')}:</strong> ${sl ? info.config_difficulty_ja : info.config_difficulty_en}</p>
      <p><strong>${t('configOps')}:</strong><br>${sl ? info.config_file_ops_ja : info.config_file_ops_en}</p>
      <details><summary>${t('strengths')}</summary><ul>${strengths}</ul></details>
      <details><summary>${t('weaknesses')}</summary><ul>${weaknesses}</ul></details>
      <details><summary>${t('configEx')}</summary><div class="code-block">${escHtml(info.config_example)}</div></details>
    </div>`;
  }

  const langRows = data.lang_frameworks.map(e => {
    const label  = LANG==='ja' ? e.label_ja : e.label_en;
    const ngC    = LANG==='ja' ? e.compat_nginx_ja  : e.compat_nginx_en;
    const apC    = LANG==='ja' ? e.compat_apache_ja : e.compat_apache_en;
    const reason = LANG==='ja' ? e.reason_ja : e.reason_en;
    return `<tr>
      <td><button class="btn btn-sm" onclick="showAdvisor('${e.id}',true)">${label}</button></td>
      <td>${compatBadge(ngC || e.compat_nginx || '')}</td>
      <td>${compatBadge(apC || e.compat_apache || '')}</td>
      <td>${e.recommended_server === 'nginx' ? '🔵 Nginx' : '🟠 Apache'}</td>
      <td style="font-size:.8em">${reason}</td>
    </tr>`;
  }).join('');

  document.getElementById('compare-content').innerHTML = `
    <div class="compare-grid">${col(n,true)}${col(a,false)}</div>
    <h3 style="margin-top:20px">${LANG==='ja'?'言語・FW別 相性表':'Compatibility by Language / Framework'}</h3>
    <div style="overflow:auto">
    <table class="table">
      <thead><tr>
        <th>${LANG==='ja'?'言語/FW':'Language/FW'}</th>
        <th>Nginx</th><th>Apache</th>
        <th>${LANG==='ja'?'推薦':'Recommended'}</th>
        <th>${LANG==='ja'?'理由':'Reason'}</th>
      </tr></thead>
      <tbody>${langRows}</tbody>
    </table></div>`;
}

function escHtml(s) {
  return String(s||'').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}

// ─── 初期化 ──────────────────────────────────────────────
async function init() {
  const res = await fetch('/api/webserver/compare').then(r=>r.json()).catch(()=>null);
  if (res && res.ok) {
    compareData = res;
    // compat ラベルを前処理
    compareData.lang_frameworks.forEach(e => {
      e.compat_nginx_ja  = compatLevelToJa(e.compat_nginx);
      e.compat_nginx_en  = compatLevelToEn(e.compat_nginx);
      e.compat_apache_ja = compatLevelToJa(e.compat_apache);
      e.compat_apache_en = compatLevelToEn(e.compat_apache);
    });
    renderLangGrid(compareData);
    renderCompare(compareData);
  } else {
    document.getElementById('compare-content').textContent = 'データ取得失敗';
  }
}

function compatLevelToJa(v) {
  const m = { Excellent:'◎ 最適', Good:'○ 良好', Fair:'△ 可', Limited:'✕ 不向き' };
  return m[v] || v;
}
function compatLevelToEn(v) {
  const m = { Excellent:'Excellent', Good:'Good', Fair:'Fair', Limited:'Limited' };
  return m[v] || v;
}

init();
</script>"#;
    html("Webサーバー設定", body)
}

/// 公開ページ — ログイン不要、誰でも利用可能
#[handler]
pub async fn webserver_wizard() -> Result<impl poem::IntoResponse> {
    let body = r#"
<section class="hero" style="text-align:center;padding:40px 20px">
  <h1 id="wiz-title">🌐 Webサーバー自動選択 &amp; 設定ファイル生成</h1>
  <p id="wiz-desc">言語・フレームワーク・データベース・サービスを選ぶだけで、<br>
  Nginx / Apache のどちらが最適かをAIが判定し、設定ファイルを無料でダウンロードできます。</p>
  <div style="margin-top:8px">
    <button class="btn btn-sm" onclick="setLang('ja')" id="btn-ja" style="font-weight:bold">🇯🇵 日本語</button>
    <button class="btn btn-sm" onclick="setLang('en')" id="btn-en">🇺🇸 English</button>
  </div>
</section>

<!-- STEP 1: 言語/FW -->
<section class="card wiz-step" id="step1">
  <div class="step-header"><span class="step-num">1</span>
    <h2 id="h-lang">言語・フレームワークを選択</h2></div>
  <div id="lang-grid" class="choice-grid"></div>
</section>

<!-- STEP 2: DB -->
<section class="card wiz-step" id="step2" style="display:none">
  <div class="step-header"><span class="step-num">2</span>
    <h2 id="h-db">データベースを選択（複数可）</h2></div>
  <div id="db-grid" class="choice-grid"></div>
  <button class="btn btn-primary" style="margin-top:12px" onclick="goStep(3)" id="btn-next2">次へ →</button>
</section>

<!-- STEP 3: サービス -->
<section class="card wiz-step" id="step3" style="display:none">
  <div class="step-header"><span class="step-num">3</span>
    <h2 id="h-svc">追加機能・サービスを選択（複数可）</h2></div>
  <div id="svc-grid" class="choice-grid"></div>

  <div class="form-row" style="margin-top:16px;gap:12px;display:flex;flex-wrap:wrap">
    <label style="flex:1;min-width:180px" id="lbl-domain">ドメイン
      <input id="inp-domain" placeholder="example.com" value="example.com">
    </label>
    <label style="flex:1;min-width:120px" id="lbl-port">アプリポート
      <input id="inp-port" type="number" value="3000" min="1" max="65535">
    </label>
  </div>
  <button class="btn btn-primary" style="margin-top:12px" onclick="getRecommendation()" id="btn-recommend">
    🤖 AI に推薦してもらう
  </button>
</section>

<!-- STEP 4: 結果 -->
<section class="card" id="step4" style="display:none">
  <div class="step-header"><span class="step-num">4</span>
    <h2 id="h-result">AI 推薦結果</h2></div>
  <div id="result-box"></div>

  <div style="margin-top:20px;padding:16px;border:1px solid var(--accent,#2563eb);border-radius:8px;background:rgba(37,99,235,.04)">
    <h3 id="h-download" style="margin:0 0 8px">📥 設定ファイルをダウンロード</h3>
    <p id="p-download-desc">推薦された設定ファイルを無料でダウンロードできます。</p>
    <div style="display:flex;gap:10px;flex-wrap:wrap">
      <button class="btn btn-primary" id="btn-dl-recommended" onclick="downloadConfig()">
        ⬇ 推薦設定をダウンロード
      </button>
      <button class="btn btn-secondary" id="btn-dl-nginx" onclick="downloadConfig('nginx')">
        Nginx 設定
      </button>
      <button class="btn btn-secondary" id="btn-dl-apache" onclick="downloadConfig('apache')">
        Apache 設定
      </button>
    </div>
  </div>
  <button class="btn btn-sm" style="margin-top:16px" onclick="restart()" id="btn-restart">
    ↺ もう一度選び直す
  </button>
</section>

<style>
.wiz-step { transition: opacity .2s; }
.step-header { display:flex; align-items:center; gap:10px; margin-bottom:12px; }
.step-num { width:32px; height:32px; border-radius:50%; background:var(--accent,#2563eb);
  color:#fff; font-weight:bold; display:flex; align-items:center; justify-content:center; flex-shrink:0; }
.choice-grid { display:grid; grid-template-columns:repeat(auto-fill,minmax(200px,1fr)); gap:8px; }
.choice-btn { padding:10px 14px; border:2px solid var(--border,#ddd); border-radius:8px;
  background:var(--card-bg,#fff); cursor:pointer; text-align:left; transition:all .15s; }
.choice-btn:hover { border-color:var(--accent,#2563eb); }
.choice-btn.selected { border-color:var(--accent,#2563eb); background:rgba(37,99,235,.08);
  box-shadow:0 0 0 2px rgba(37,99,235,.2); }
.score-bar { display:flex; gap:8px; align-items:center; margin:6px 0; }
.score-fill { height:10px; border-radius:5px; transition:width .4s; }
.score-nginx  { background:#2563eb; }
.score-apache { background:#ea580c; }
.rec-winner { border:2px solid #22c55e; border-radius:10px; padding:16px; background:rgba(34,197,94,.06); }
.rec-other  { border:2px solid var(--border,#ddd); border-radius:10px; padding:16px; }
.note-list  { font-size:.85em; margin:8px 0; padding-left:16px; }
.code-preview { background:var(--code-bg,#f4f4f4); border-radius:6px; padding:12px;
  font-family:monospace; font-size:.78em; overflow:auto; white-space:pre; max-height:220px; margin-top:8px; }
</style>

<script>
let LANG = 'ja';
let compareData = null;
let state = { lang: null, dbs: new Set(), svcs: new Set() };
let lastRec = null;

const TXT = {
  ja: {
    title: '🌐 Webサーバー自動選択 &amp; 設定ファイル生成',
    desc:  '言語・フレームワーク・データベース・サービスを選ぶだけで、<br>Nginx / Apacheのどちらが最適かをAIが判定し、設定ファイルを無料でダウンロードできます。',
    hLang:'言語・フレームワークを選択', hDb:'データベースを選択（複数可）',
    hSvc:'追加機能・サービスを選択（複数可）', hResult:'AI 推薦結果',
    hDl:'📥 設定ファイルをダウンロード',
    dlDesc:'推薦された設定ファイルを無料でダウンロードできます。',
    dlRec:'⬇ 推薦設定をダウンロード', dlNginx:'Nginx 設定', dlApache:'Apache 設定',
    next:'次へ →', recommend:'🤖 AI に推薦してもらう', restart:'↺ もう一度選び直す',
    domain:'ドメイン', port:'アプリポート',
    score:'スコア', reason:'推薦理由', tips:'ヒント',
    dbNotes:'DB 別ポイント', svcNotes:'サービス別ポイント',
    preview:'設定プレビュー',
  },
  en: {
    title: '🌐 Web Server Auto-Select &amp; Config Generator',
    desc:  'Select language, framework, database, and services.<br>AI picks Nginx or Apache and generates your config file for free download.',
    hLang:'Select Language / Framework', hDb:'Select Databases (multi-select)',
    hSvc:'Select Additional Services (multi-select)', hResult:'AI Recommendation',
    hDl:'📥 Download Config File',
    dlDesc:'Download the recommended config file for free.',
    dlRec:'⬇ Download Recommended', dlNginx:'Nginx Config', dlApache:'Apache Config',
    next:'Next →', recommend:'🤖 Get AI Recommendation', restart:'↺ Start Over',
    domain:'Domain', port:'App Port',
    score:'Score', reason:'Reason', tips:'Tips',
    dbNotes:'DB Notes', svcNotes:'Service Notes',
    preview:'Config Preview',
  }
};
function t(k) { return TXT[LANG][k] || k; }

function setLang(l) {
  LANG = l;
  document.getElementById('btn-ja').style.fontWeight = l==='ja'?'bold':'normal';
  document.getElementById('btn-en').style.fontWeight = l==='en'?'bold':'normal';
  applyTxt();
  if (compareData) { renderGrids(); }
  if (lastRec) renderResult(lastRec);
}

function applyTxt() {
  const map = { 'wiz-title':'title','wiz-desc':'desc','h-lang':'hLang',
    'h-db':'hDb','h-svc':'hSvc','h-result':'hResult','h-download':'hDl',
    'p-download-desc':'dlDesc','btn-next2':'next','btn-recommend':'recommend',
    'btn-restart':'restart','btn-dl-recommended':'dlRec','btn-dl-nginx':'dlNginx',
    'btn-dl-apache':'dlApache','lbl-domain':'domain','lbl-port':'port' };
  Object.entries(map).forEach(([id,k]) => {
    const el = document.getElementById(id);
    if (!el) return;
    if (el.tagName==='LABEL') el.childNodes[0].textContent = t(k)+'\n';
    else el.innerHTML = t(k);
  });
}

// ─── グリッド描画 ────────────────────────────────────────
function renderGrids() {
  if (!compareData) return;

  // 言語グリッド
  document.getElementById('lang-grid').innerHTML =
    compareData.lang_frameworks.map(e => {
      const label = LANG==='ja' ? e.label_ja : e.label_en;
      const rec   = e.recommended_server === 'nginx' ? '🔵 Nginx' : '🟠 Apache';
      return `<button class="choice-btn${state.lang===e.id?' selected':''}"
        onclick="selectLang('${e.id}')" data-id="${e.id}">
        <strong>${label}</strong><br><small>${rec}</small></button>`;
    }).join('');

  // DB グリッド
  document.getElementById('db-grid').innerHTML =
    compareData.databases.map(d => {
      const label = LANG==='ja' ? d.label_ja : d.label_en;
      const cat   = LANG==='ja' ? catJa(d.category) : d.category.toUpperCase();
      return `<button class="choice-btn${state.dbs.has(d.id)?' selected':''}"
        onclick="toggleDb('${d.id}')" data-id="${d.id}">
        <strong>${label}</strong><br><small>${cat} :${d.default_port||'N/A'}</small></button>`;
    }).join('');

  // サービスグリッド
  document.getElementById('svc-grid').innerHTML =
    compareData.additional_services.map(s => {
      const label = LANG==='ja' ? s.label_ja : s.label_en;
      return `<button class="choice-btn${state.svcs.has(s.id)?' selected':''}"
        onclick="toggleSvc('${s.id}')" data-id="${s.id}">
        ${label}</button>`;
    }).join('');
}

function catJa(c) {
  return { rdbms:'RDBMS', nosql:'NoSQL', cache:'キャッシュ', search:'検索', queue:'キュー' }[c] || c;
}

// ─── ステップ制御 ────────────────────────────────────────
function selectLang(id) {
  state.lang = id;
  renderGrids();
  goStep(2);
}

function toggleDb(id) {
  if (state.dbs.has(id)) state.dbs.delete(id); else state.dbs.add(id);
  renderGrids();
}

function toggleSvc(id) {
  if (state.svcs.has(id)) state.svcs.delete(id); else state.svcs.add(id);
  renderGrids();
}

function goStep(n) {
  [1,2,3,4].forEach(i => {
    const el = document.getElementById('step'+i);
    if (el) el.style.display = i<=n ? '' : 'none';
  });
  document.getElementById('step'+n)?.scrollIntoView({behavior:'smooth'});
}

function restart() {
  state = { lang: null, dbs: new Set(), svcs: new Set() };
  lastRec = null;
  renderGrids();
  [2,3,4].forEach(i => {
    const el = document.getElementById('step'+i);
    if (el) el.style.display = 'none';
  });
  document.getElementById('step1').scrollIntoView({behavior:'smooth'});
}

// ─── AI 推薦 ─────────────────────────────────────────────
async function getRecommendation() {
  const domain = document.getElementById('inp-domain').value || 'example.com';
  const port   = Number(document.getElementById('inp-port').value) || 3000;
  const body   = {
    lang: state.lang || 'static',
    databases: [...state.dbs],
    services:  [...state.svcs],
    port, domain,
  };

  const btn = document.getElementById('btn-recommend');
  btn.disabled = true;
  btn.textContent = LANG==='ja' ? '判定中…' : 'Analyzing…';

  try {
    const d = await fetch('/api/webserver/recommend', {
      method:'POST', headers:{'Content-Type':'application/json'},
      body: JSON.stringify(body)
    }).then(r=>r.json());

    if (d.ok) {
      lastRec = { ...d.recommendation, domain, port };
      renderResult(lastRec);
      goStep(4);
    }
  } finally {
    btn.disabled = false;
    btn.innerHTML = t('recommend');
  }
}

function renderResult(rec) {
  const isNginx = rec.recommended === 'nginx';
  const winner  = isNginx ? 'Nginx' : 'Apache';
  const loser   = isNginx ? 'Apache' : 'Nginx';
  const wScore  = isNginx ? rec.nginx_score : rec.apache_score;
  const lScore  = isNginx ? rec.apache_score : rec.nginx_score;
  const maxScore = Math.max(rec.nginx_score, rec.apache_score, 1);
  const reason   = LANG==='ja' ? rec.reason_ja : rec.reason_en;
  const tips     = LANG==='ja' ? rec.tips_ja   : rec.tips_en;
  const dbNotes  = LANG==='ja' ? rec.db_notes_ja   : rec.db_notes_en;
  const svcNotes = LANG==='ja' ? rec.service_notes_ja : rec.service_notes_en;

  const dbList  = dbNotes.length  ? `<p><strong>${t('dbNotes')}:</strong></p><ul class="note-list">${dbNotes.map(n=>`<li>${escHtml(n)}</li>`).join('')}</ul>` : '';
  const svcList = svcNotes.length ? `<p><strong>${t('svcNotes')}:</strong></p><ul class="note-list">${svcNotes.map(n=>`<li>${escHtml(n)}</li>`).join('')}</ul>` : '';

  const previewShort = rec.config_content
    ? rec.config_content.split('\n').slice(0, 30).join('\n') + '\n...'
    : '';

  document.getElementById('result-box').innerHTML = `
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:14px;margin-bottom:16px">
      <div class="rec-winner">
        <h3>✅ ${winner} <small>(推薦)</small></h3>
        <div class="score-bar">
          <span style="min-width:60px">${t('score')}: ${wScore}</span>
          <div class="score-fill ${isNginx?'score-nginx':'score-apache'}" style="width:${Math.round(wScore/maxScore*100)}%"></div>
        </div>
      </div>
      <div class="rec-other">
        <h3>${loser}</h3>
        <div class="score-bar">
          <span style="min-width:60px">${t('score')}: ${lScore}</span>
          <div class="score-fill ${isNginx?'score-apache':'score-nginx'}" style="width:${Math.round(lScore/maxScore*100)}%"></div>
        </div>
      </div>
    </div>
    <p><strong>${t('reason')}:</strong> ${escHtml(reason)}</p>
    ${tips ? `<p><strong>${t('tips')}:</strong> ${escHtml(tips)}</p>` : ''}
    ${dbList}${svcList}
    <details style="margin-top:12px">
      <summary>${t('preview')}</summary>
      <div class="code-preview">${escHtml(previewShort)}</div>
    </details>`;

  // ダウンロードボタンの推薦サーバー表示更新
  document.getElementById('btn-dl-recommended').innerHTML =
    `⬇ ${winner} ${LANG==='ja'?'設定（推薦）':'Config (Recommended)'}`;
}

// ─── ダウンロード ─────────────────────────────────────────
function downloadConfig(forceServer) {
  if (!lastRec) return;
  const server = forceServer || lastRec.recommended;
  const domain = lastRec.domain || 'example.com';
  const port   = lastRec.port   || 3000;
  const dbs    = [...state.dbs].join(',');
  const svcs   = [...state.svcs].join(',');
  const lang   = state.lang || 'generic';
  const url    = `/api/webserver/download?server=${server}&lang=${encodeURIComponent(lang)}`
               + `&domain=${encodeURIComponent(domain)}&port=${port}`
               + `&databases=${encodeURIComponent(dbs)}&services=${encodeURIComponent(svcs)}`;
  window.location.href = url;
}

function escHtml(s) {
  return String(s||'').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}

// ─── 初期化 ──────────────────────────────────────────────
async function init() {
  const d = await fetch('/api/webserver/compare').then(r=>r.json()).catch(()=>null);
  if (d && d.ok) {
    compareData = d;
    renderGrids();
  } else {
    document.getElementById('lang-grid').textContent = LANG==='ja'?'データ取得失敗':'Failed to load data';
  }
}

init();
</script>"#;
    html("Webサーバー自動選択", body)
}

// ─── GitHub プロジェクト管理ページ ───────────────────────

#[handler]
pub async fn github_settings(
    state: Data<&SharedState>,
    session: &Session,
) -> Result<impl poem::IntoResponse> {
    if session_email(session).is_none() {
        return Ok(poem::Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", "/login")
            .finish());
    }

    let token_set = state.github_token().is_some();
    let projects  = state.list_github_projects().await.unwrap_or_default();
    let selected_count = projects.iter().filter(|p| p.selected != 0).count();

    let project_rows: String = projects.iter().map(|p| {
        let checked  = if p.selected != 0 { "checked" } else { "" };
        let lang     = p.language.as_deref().unwrap_or("-");
        let updated  = &p.updated_at_gh[..10];
        let has_readme = if p.readme_content.is_some() { "✓" } else { "—" };
        format!(
            r#"<tr>
              <td><input type="checkbox" class="proj-check" data-id="{ghid}" {checked}></td>
              <td><a href="{url}" target="_blank"><strong>{name}</strong></a><br>
                  <small>{desc}</small></td>
              <td>{lang}</td>
              <td>⭐ {stars}</td>
              <td>{updated}</td>
              <td>{readme}</td>
              <td><button class="btn btn-sm" onclick="fetchReadme({ghid})">README取得</button></td>
            </tr>"#,
            ghid  = p.gh_id,
            url   = p.html_url,
            name  = p.name,
            desc  = p.description.as_deref().unwrap_or(""),
            lang  = lang,
            stars = p.stargazers_count,
            updated = updated,
            readme  = has_readme,
            checked = checked,
        )
    }).collect();

    let token_status = if token_set {
        r#"<span class="badge badge-ok">✓ トークン設定済み</span>"#
    } else {
        r#"<span class="badge badge-warn">⚠ 未設定</span>"#
    };

    let body = format!(
        r#"
<section class="hero">
  <h1>GitHub プロジェクト管理</h1>
  <p>表示するリポジトリを選択し、README から <strong>PowerPoint 資料を自動生成</strong>できます。<br>
  データは <strong>10分ごとに自動取得</strong>されます。</p>
</section>

<!-- トークン設定 -->
<section class="card">
  <h2>GitHub 設定 {token_status}</h2>
  <form id="gh-config-form" class="form-grid">
    <label>Personal Access Token（repo スコープ）
      <input name="token" type="password" placeholder="ghp_..." autocomplete="off">
    </label>
    <label>組織名（オプション）
      <input name="org" placeholder="my-org">
    </label>
    <button type="submit" class="btn btn-primary">保存</button>
  </form>
  <div id="config-result" class="result"></div>
</section>

<!-- リポジトリ一覧 -->
<section class="card">
  <h2>リポジトリ一覧 <small>（選択中: <span id="sel-count">{selected_count}</span> 件）</small></h2>
  <div style="display:flex;gap:8px;flex-wrap:wrap;margin-bottom:12px">
    <button class="btn btn-primary" onclick="syncRepos()">🔄 GitHubから最新取得</button>
    <button class="btn btn-primary" onclick="saveSelection()">✓ 選択を保存</button>
    <a class="btn btn-primary" href="/api/github/pptx?lang=ja" download>
      ⬇ 選択中を PPTX 生成（日本語）
    </a>
    <a class="btn btn-secondary" href="/api/github/pptx?lang=en" download>
      ⬇ Generate PPTX (English)
    </a>
  </div>
  <div id="sync-result" class="result" style="margin-bottom:8px"></div>
  <div style="overflow:auto">
  <table class="table" id="proj-table">
    <thead><tr>
      <th><input type="checkbox" id="check-all" onchange="toggleAll(this)"></th>
      <th>リポジトリ</th><th>言語</th><th>Stars</th><th>更新日</th><th>README</th><th>操作</th>
    </tr></thead>
    <tbody>
      {project_rows}
    </tbody>
  </table>
  </div>
  {empty_msg}
</section>

<!-- PPTX 個別生成 -->
<section class="card">
  <h2>個別 PPTX 生成</h2>
  <p>特定のリポジトリのみを対象に PPTX を生成します。</p>
  <div class="form-grid">
    <label>リポジトリ ID（カンマ区切り）
      <input id="custom-ids" placeholder="例: 123456,789012">
    </label>
    <label>言語
      <select id="custom-lang">
        <option value="ja">日本語</option>
        <option value="en">English</option>
      </select>
    </label>
  </div>
  <button class="btn btn-primary" style="margin-top:8px" onclick="downloadCustom()">
    ⬇ PPTX ダウンロード
  </button>
</section>

<script>
// ─── 設定保存 ───────────────────────────────────────────
document.getElementById('gh-config-form').addEventListener('submit', async e => {{
  e.preventDefault();
  const fd = new FormData(e.target);
  const res = document.getElementById('config-result');
  const d = await fetch('/api/github/config', {{
    method: 'POST',
    headers: {{'Content-Type': 'application/json'}},
    body: JSON.stringify({{ token: fd.get('token'), org: fd.get('org') || null }})
  }}).then(r => r.json()).catch(err => ({{ ok: false, error: err.toString() }}));
  res.textContent = d.ok ? '✓ 保存しました' : '✗ ' + d.error;
  res.className = 'result ' + (d.ok ? 'result-ok' : 'result-error');
}});

// ─── GitHub 同期 ────────────────────────────────────────
async function syncRepos() {{
  const res = document.getElementById('sync-result');
  res.textContent = '同期中…';
  res.className = 'result';
  const d = await fetch('/api/github/sync', {{ method: 'POST' }})
    .then(r => r.json()).catch(e => ({{ ok: false, error: e.toString() }}));
  if (d.ok) {{
    res.textContent = '✓ ' + d.count + ' 件同期完了 — ページを再読み込みします';
    res.className = 'result result-ok';
    setTimeout(() => location.reload(), 1200);
  }} else {{
    res.textContent = '✗ ' + d.error;
    res.className = 'result result-error';
  }}
}}

// ─── 選択保存 ────────────────────────────────────────────
async function saveSelection() {{
  const ids = [...document.querySelectorAll('.proj-check:checked')]
    .map(el => parseInt(el.dataset.id));
  document.getElementById('sel-count').textContent = ids.length;
  const d = await fetch('/api/github/select', {{
    method: 'POST',
    headers: {{'Content-Type': 'application/json'}},
    body: JSON.stringify({{ gh_ids: ids }})
  }}).then(r => r.json()).catch(e => ({{ ok: false, error: e.toString() }}));
  const res = document.getElementById('sync-result');
  res.textContent = d.ok ? '✓ 選択を保存しました' : '✗ ' + d.error;
  res.className = 'result ' + (d.ok ? 'result-ok' : 'result-error');
}}

// ─── 全選択 ──────────────────────────────────────────────
function toggleAll(el) {{
  document.querySelectorAll('.proj-check').forEach(c => c.checked = el.checked);
}}

// ─── README 即時取得 ─────────────────────────────────────
async function fetchReadme(ghId) {{
  const d = await fetch('/api/github/readme', {{
    method: 'POST',
    headers: {{'Content-Type': 'application/json'}},
    body: JSON.stringify({{ gh_id: ghId }})
  }}).then(r => r.json()).catch(e => ({{ ok: false, error: e.toString() }}));
  const res = document.getElementById('sync-result');
  res.textContent = d.ok ? '✓ README 取得完了' : '✗ ' + d.error;
  res.className = 'result ' + (d.ok ? 'result-ok' : 'result-error');
}}

// ─── 個別 PPTX ──────────────────────────────────────────
function downloadCustom() {{
  const ids  = document.getElementById('custom-ids').value.trim();
  const lang = document.getElementById('custom-lang').value;
  const url  = '/api/github/pptx?lang=' + lang + (ids ? '&ids=' + encodeURIComponent(ids) : '');
  window.location.href = url;
}}
</script>"#,
        token_status   = token_status,
        selected_count = selected_count,
        project_rows   = project_rows,
        empty_msg = if projects.is_empty() {
            r#"<p class="hint">「GitHubから最新取得」ボタンでリポジトリを同期してください。</p>"#
        } else { "" },
    );

    html("GitHub プロジェクト管理", &body)
}

fn html(title: &str, body: &str) -> Result<poem::Response> {
    Ok(poem::Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .body(layout(title, body)))
}

pub fn page_routes() -> poem::Route {
    poem::Route::new()
        .at("/", poem::get(index))
        .at("/register", poem::get(register_page))
        .at("/login", poem::get(login_page))
        .at("/auth/verify", poem::get(verify))
        .at("/dashboard", poem::get(dashboard))
        .at("/settings/domain", poem::get(domain_settings))
        .at("/settings/easyweb", poem::get(easyweb_settings))
        .at("/settings/https", poem::get(https_settings))
        .at("/settings/server", poem::get(server_status_page))
        .at("/settings/webserver", poem::get(webserver_settings))
        .at("/settings/github",    poem::get(github_settings))
        .at("/tools/webserver",    poem::get(webserver_wizard))
}
