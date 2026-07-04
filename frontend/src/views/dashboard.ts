import { aruaruApi, type ServerStatus } from '../api';
import { renderServerDashboard } from '../components/server-dashboard';
import { escapeHtml, layout } from '../components/layout';

export async function renderDashboard(root: HTMLElement): Promise<void> {
  const [health, status, plugins] = await Promise.all([
    aruaruApi.health(),
    aruaruApi.serverStatus(),
    aruaruApi.plugins(),
  ]);

  const pluginRows =
    plugins.plugins?.length > 0
      ? plugins.plugins
          .map(
            (p) =>
              `<tr><td><code>${escapeHtml(p.id)}</code></td><td>${escapeHtml(p.version)}</td></tr>`
          )
          .join('')
      : '<tr><td colspan="2" class="text-muted">未インストール</td></tr>';

  root.innerHTML = layout(
    'ダッシュボード',
    `
<p class="text-muted">Linux 初心者向け — サーバー状態を一目で確認（コマンド不要）</p>
<div class="d-flex gap-2 mb-3">
  <button type="button" class="btn btn-sm btn-outline-secondary" id="dash-refresh">再読み込み</button>
  <a class="btn btn-sm btn-outline-primary" href="#/wizard">接続ウィザード</a>
</div>
<div id="dash-status">${renderServerDashboard(status)}</div>
<div class="card border-secondary">
  <div class="card-header">インストール済みプラグイン</div>
  <div class="table-responsive">
    <table class="table table-dark table-striped mb-0">
      <thead><tr><th>ID</th><th>Ver</th></tr></thead>
      <tbody>${pluginRows}</tbody>
    </table>
  </div>
</div>
<p class="mt-2 small text-muted">aruaru ${escapeHtml(health.version)} · DB ${escapeHtml(health.database.backend)}</p>`
  );

  root.querySelector('#dash-refresh')?.addEventListener('click', async () => {
    const el = root.querySelector('#dash-status')!;
    el.innerHTML = '<p class="text-muted">読み込み中…</p>';
    const st = await aruaruApi.serverStatus();
    el.innerHTML = renderServerDashboard(st);
  });
}
