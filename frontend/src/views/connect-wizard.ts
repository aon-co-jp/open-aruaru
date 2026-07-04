import { aruaruApi } from '../api';
import { renderServerDashboard } from '../components/server-dashboard';
import { alertBox, escapeHtml, layout } from '../components/layout';

const PROVIDERS = [
  { id: 'conoha', label: 'ConoHa VPS' },
  { id: 'sakura', label: 'さくらのVPS' },
  { id: 'xserver', label: 'Xserver VPS' },
  { id: 'aws', label: 'AWS' },
  { id: 'azure', label: 'Azure' },
  { id: 'gcp', label: 'Google Cloud' },
  { id: 'oracle', label: 'Oracle Cloud' },
  { id: 'other', label: 'その他' },
];

export async function renderConnectWizard(root: HTMLElement): Promise<void> {
  const health = await aruaruApi.health();
  const status = await aruaruApi.serverStatus();

  root.innerHTML = layout(
    'セットアップウィザード',
    `
<p class="text-muted">VPS 上の aruaru はすでに動いています。ブラウザからドメイン・HTTPS を設定しましょう。</p>
<p class="small">SSH 接続ウィザードは <strong>aruaru-desktop</strong>（Windows アプリ）で利用できます。</p>

<div class="card border-secondary mb-3">
  <div class="card-header">ステップ 1 — 管理者登録</div>
  <div class="card-body">
    <p class="mb-2">${health.registered ? alertBox('success', '管理者登録済み') : alertBox('info', '未登録 — 初回登録してください')}</p>
    <a class="btn btn-sm btn-primary" href="#/${health.registered ? 'login' : 'register'}">${health.registered ? 'ログイン' : '初回登録'}</a>
  </div>
</div>

<div class="card border-secondary mb-3">
  <div class="card-header">ステップ 2 — サーバー状態</div>
  <div class="card-body" id="wizard-status">${renderServerDashboard(status, { showActions: false })}</div>
</div>

<div class="card border-secondary mb-3">
  <div class="card-header">ステップ 3 — ドメイン・SSL</div>
  <div class="card-body">
    <ol class="mb-3">
      <li><a href="#/settings/domain">運用ドメインを登録</a></li>
      <li>レジストラで A レコードを VPS の IP に向ける</li>
      <li><a href="#/features">機能一覧</a> → <code>https.certbot</code> で SSL 取得</li>
      <li><a href="#/features">vhost.install</a> で Nginx に反映</li>
    </ol>
    <a class="btn btn-primary" href="#/settings/domain">ドメイン設定へ</a>
    <a class="btn btn-outline-secondary ms-2" href="#/features">機能一覧</a>
  </div>
</div>

<div class="card border-secondary">
  <div class="card-header">プロバイダー参考（デスクトップ版）</div>
  <div class="card-body">
    <p class="small text-muted mb-2">Windows から VPS へ初回接続する際のプロバイダー一覧:</p>
    <ul class="list-unstyled small mb-0">
      ${PROVIDERS.map((p) => `<li><code>${escapeHtml(p.id)}</code> — ${escapeHtml(p.label)}</li>`).join('')}
    </ul>
  </div>
</div>`
  );
}
