export function layout(title: string, body: string): string {
  return `
<header class="navbar navbar-expand-lg navbar-dark bg-dark border-bottom border-secondary">
  <div class="container">
    <a class="navbar-brand fw-semibold" href="#/">aruaru <span class="badge text-bg-secondary ms-1">v0.1</span></a>
    <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#nav">
      <span class="navbar-toggler-icon"></span>
    </button>
    <div class="collapse navbar-collapse" id="nav">
      <ul class="navbar-nav ms-auto">
        <li class="nav-item"><a class="nav-link" href="#/features">機能一覧</a></li>
        <li class="nav-item"><a class="nav-link" href="#/handoff">引き継ぎ / AI</a></li>
        <li class="nav-item"><a class="nav-link" href="#/wizard">接続ウィザード</a></li>
        <li class="nav-item"><a class="nav-link" href="#/dashboard">ダッシュボード</a></li>
        <li class="nav-item"><a class="nav-link" href="#/settings/domain">運用ドメイン</a></li>
        <li class="nav-item"><a class="nav-link" href="#/settings/easyweb">easyweb</a></li>
        <li class="nav-item"><a class="nav-link" href="#/settings/https">HTTPS</a></li>
      </ul>
    </div>
  </div>
</header>
<main class="container py-4">
  <h1 class="h3 mb-3">${escapeHtml(title)}</h1>
  ${body}
</main>
<footer class="container pb-4 text-muted small">
  aruaru — 自己ホスト型 Web 運用プラットフォーム（第二の KUSANAGI）
</footer>`;
}

export function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function alertBox(kind: 'success' | 'danger' | 'info', text: string): string {
  return `<div class="alert alert-${kind}">${escapeHtml(text)}</div>`;
}

export function magicLinkBlock(url: string): string {
  return `
<div class="alert alert-success">
  <p class="mb-2">ログインリンク（SMTP 未設定時はここに表示）</p>
  <a class="btn btn-sm btn-outline-success" href="${escapeHtml(url)}">リンクを開く</a>
</div>`;
}
