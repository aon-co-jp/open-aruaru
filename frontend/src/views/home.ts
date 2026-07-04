import { aruaruApi } from '../api';
import { layout } from '../components/layout';

export async function renderHome(root: HTMLElement): Promise<void> {
  const health = await aruaruApi.health();
  const db = health.database;
  root.innerHTML = layout(
    'ホーム',
    `
<div class="row g-3">
  <div class="col-md-8">
    <div class="card border-secondary">
      <div class="card-body">
        <p class="lead">第二の KUSANAGI — 自分の VPS で Web 運用を完結。</p>
        <p class="mb-0 text-muted">メールはあなたのサーバー内のみ。作者は預かりません。</p>
      </div>
    </div>
  </div>
  <div class="col-md-4">
    <div class="card border-secondary h-100">
      <div class="card-body">
        <h2 class="h6 text-muted">ステータス</h2>
        <ul class="list-unstyled small mb-0">
          <li>バージョン: <code>${health.version}</code></li>
          <li>DB: <code>${db.backend}</code> (${db.primary})</li>
          <li>公開 URL: <code>${health.public_url}</code></li>
          <li>管理者: ${health.registered ? '登録済み' : '未登録'}</li>
        </ul>
      </div>
    </div>
  </div>
</div>
<div class="mt-4 d-flex gap-2 flex-wrap">
  <a class="btn btn-primary" href="#/${health.registered ? 'login' : 'register'}">
    ${health.registered ? 'ログイン' : '初回登録'}
  </a>
  <a class="btn btn-outline-secondary" href="#/dashboard">ダッシュボード</a>
</div>`
  );
}
