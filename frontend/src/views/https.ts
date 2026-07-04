import { aruaruApi, type HttpsStatus } from '../api';
import { alertBox, escapeHtml, layout } from '../components/layout';

export async function renderHttps(root: HTMLElement): Promise<void> {
  const [httpsRes, domainsRes, health] = await Promise.all([
    aruaruApi.httpsStatus().catch(() => ({ ok: false, https: null as HttpsStatus | null })),
    aruaruApi.platformDomains().catch(() => ({ ok: false, domains: [] })),
    aruaruApi.health().catch(() => null),
  ]);

  const https = httpsRes.https;
  const domains = domainsRes.domains ?? [];
  const httpsCount = domains.filter((d) => d.https_enabled).length;
  const renewHours = https ? Math.round(https.renewal_interval_secs / 3600) : 24;
  const scanMin = https ? Math.round(https.scan_interval_secs / 60) : 5;

  const queueRows =
    https?.pending_queue?.length ?
      https.pending_queue
        .map(
          (q) =>
            `<tr><td><code>${escapeHtml(q.domain ?? '—')}</code></td><td class="small">${escapeHtml(q.note ?? '')}</td><td class="small text-muted">${escapeHtml(q.queued_at ?? '')}</td></tr>`
        )
        .join('')
    : '<tr><td colspan="3" class="text-muted">キューなし</td></tr>';

  root.innerHTML = layout(
    'aruaru-https',
    `
<p class="text-muted">ドメイン登録後は <strong>自動で Let's Encrypt</strong> を取得し、${renewHours} 時間ごとに更新します。手動 certbot は不要です。</p>

<div class="row g-3">
  <div class="col-lg-6">
    <div class="card border-secondary h-100">
      <div class="card-header">監視・自動設定</div>
      <div class="card-body small">
        <ul class="mb-0">
          <li>監視: ${https?.enabled ? '✅ 有効' : '❌ 無効'}</li>
          <li>登録直後の自動 HTTPS: ${https?.auto_configure ? '✅ ON' : '❌ OFF'}</li>
          <li>certbot メール: ${https?.certbot_email_set ? '✅ 設定済み' : '⚠️ 未設定（data/config.toml）'}</li>
          <li>Web サーバー: <code>${escapeHtml(https?.web_server ?? 'nginx')}</code></li>
          <li>ACME webroot: <code>${escapeHtml(https?.acme_webroot ?? 'acme-webroot')}</code></li>
          <li>キュー再試行: ${scanMin} 分ごと</li>
          <li>証明書更新: ${renewHours} 時間ごと（certbot renew）</li>
        </ul>
      </div>
    </div>
  </div>
  <div class="col-lg-6">
    <div class="card border-secondary h-100">
      <div class="card-header">ドメイン SSL 状態</div>
      <div class="card-body small">
        <p>登録 ${domains.length} 件 · HTTPS 有効 ${httpsCount} 件</p>
        ${
          domains.length ?
            `<ul class="mb-0">${domains
              .map(
                (d) =>
                  `<li><code>${escapeHtml(d.name)}</code> ${d.https_enabled ? '🟢 HTTPS' : '🟡 待機/取得中'}${d.is_primary ? ' <span class="badge text-bg-primary">主</span>' : ''}</li>`
              )
              .join('')}</ul>`
          : '<p class="text-muted mb-0">ドメイン未登録 — <a href="#/settings/domain">運用ドメイン</a> から追加</p>'
        }
        ${health?.public_url ? `<p class="mt-2 mb-0">公開 URL: <a href="${escapeHtml(health.public_url)}">${escapeHtml(health.public_url)}</a></p>` : ''}
      </div>
    </div>
  </div>
</div>

<div class="card border-secondary mt-3">
  <div class="card-header">HTTPS キュー（取得待ち・再試行）</div>
  <div class="card-body p-0">
    <table class="table table-sm table-dark mb-0">
      <thead><tr><th>ドメイン</th><th>メモ</th><th>キュー日時</th></tr></thead>
      <tbody>${queueRows}</tbody>
    </table>
  </div>
</div>

<div class="card border-secondary mt-3">
  <div class="card-body small mb-0">
    <p class="mb-2"><strong>手順（初心者向け）</strong></p>
    <ol class="mb-0">
      <li><a href="#/settings/domain">運用ドメイン</a> にドメインを登録 → 自動 HTTPS 開始</li>
      <li>DNS の A レコードを VPS に向ける（<a href="#/features">機能一覧</a> の DNS ガイド可）</li>
      <li>失敗時は上のキューに残り、数分ごとに再試行</li>
    </ol>
    <p class="text-muted mt-2 mb-0">設定ファイル: VPS 上の <code>data/config.toml</code> の <code>[https_monitor]</code></p>
  </div>
</div>`
  );

  if (!httpsRes.ok) {
    root.insertAdjacentHTML('afterbegin', alertBox('info', 'HTTPS 状態 API に接続できませんでした'));
  }
}
