import { aruaruApi, type FeatureItem } from '../api';
import { alertBox, escapeHtml, layout } from '../components/layout';

export async function renderFeatures(root: HTMLElement): Promise<void> {
  const data = await aruaruApi.features();
  const features = data.features ?? [];
  const byCat = groupBy(features, (f) => f.category);

  const rows = Object.entries(byCat)
    .map(
      ([cat, items]) => `
<section class="mb-4">
  <h2 class="h5 text-muted">${escapeHtml(catLabel(cat))}</h2>
  <div class="list-group">
    ${items
      .map(
        (f) => `
    <div class="list-group-item" data-feature-id="${escapeHtml(f.id)}">
      <div class="d-flex justify-content-between align-items-start gap-3">
        <div>
          <strong>${escapeHtml(f.name)}</strong>
          <span class="badge text-bg-secondary ms-2">${escapeHtml(f.platform)}</span>
          <p class="mb-0 small text-muted">${escapeHtml(f.description)}</p>
        </div>
        ${
          f.runnable
            ? `<button type="button" class="btn btn-sm btn-primary run-feature" data-id="${escapeHtml(f.id)}">実行</button>`
            : ''
        }
      </div>
      <pre class="feature-result small mt-2 mb-0 d-none bg-dark text-light p-2 rounded"></pre>
    </div>`
      )
      .join('')}
  </div>
</section>`
    )
    .join('');

  root.innerHTML = layout(
    '機能一覧',
    `
<p class="text-muted">第二の KUSANAGI — ドメイン・HTTPS・vhost を選択して実行（VPS Linux 版）</p>
<div id="features-global-msg"></div>
${rows || alertBox('info', '機能がありません')}
<div class="mt-3">
  <label class="form-label small">certbot 用 domain（https.certbot）</label>
  <input id="certbot-domain" class="form-control form-control-sm" placeholder="example.com" />
</div>`
  );

  root.querySelectorAll('.run-feature').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const id = (btn as HTMLButtonElement).dataset.id!;
      const item = btn.closest('.list-group-item')!;
      const pre = item.querySelector('.feature-result') as HTMLElement;
      const global = root.querySelector('#features-global-msg') as HTMLElement;
      pre.classList.remove('d-none');
      pre.textContent = '実行中…';
      try {
        const params: Record<string, string> = {};
        if (id === 'https.certbot') {
          const d = (root.querySelector('#certbot-domain') as HTMLInputElement).value.trim();
          if (!d) throw new Error('ドメインを入力してください');
          params.domain = d;
        }
        if (id === 'dns.guide') {
          params.zone = (root.querySelector('#certbot-domain') as HTMLInputElement).value.trim() || 'example.com';
          params.vps_ip = window.location.hostname;
        }
        const res = await aruaruApi.runFeature(id, params);
        pre.textContent = JSON.stringify(res.result ?? res, null, 2);
        global.innerHTML = alertBox('success', `${id} 完了`);
      } catch (e) {
        pre.textContent = String(e);
        global.innerHTML = alertBox('danger', String(e));
      }
    });
  });
}

function groupBy<T>(arr: T[], key: (t: T) => string): Record<string, T[]> {
  return arr.reduce(
    (acc, x) => {
      const k = key(x);
      (acc[k] ??= []).push(x);
      return acc;
    },
    {} as Record<string, T[]>
  );
}

function catLabel(c: string): string {
  const m: Record<string, string> = {
    easyweb: 'easyweb',
    domain: 'ドメイン',
    https: 'HTTPS',
    vhost: 'Web サーバー',
    dns: 'DNS',
    recovery: 'OS 復元',
    plugin: 'プラグイン',
    drive: 'ドライブ',
    backup: 'バックアップ',
    vps: 'VPS',
  };
  return m[c] ?? c;
}
