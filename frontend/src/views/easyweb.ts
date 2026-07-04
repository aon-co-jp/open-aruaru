import { aruaruApi } from '../api';
import { escapeHtml, layout } from '../components/layout';

export async function renderEasyweb(root: HTMLElement): Promise<void> {
  const list = await aruaruApi.domains();
  const domains = list.domains || [];

  const rows =
    domains.length > 0
      ? domains
          .map(
            (d) =>
              `<tr><td><code>${escapeHtml(d.name)}</code></td><td>${escapeHtml(d.last_checked || '-')}</td><td>${d.https_enabled ? 'ON' : 'OFF'}</td></tr>`
          )
          .join('')
      : '<tr><td colspan="3" class="text-muted">ドメイン未登録</td></tr>';

  root.innerHTML = layout(
    'aruaru-easyweb',
    `
<p class="text-muted">ホストするサイト用ドメイン（本体の運用ドメインは <a href="#/settings/domain">こちら</a>）</p>
<form id="domain-form" class="card border-secondary mb-3">
  <div class="card-body">
    <label class="form-label">ドメイン追加</label>
    <div class="input-group">
      <input class="form-control" name="name" placeholder="example.com" required />
      <button class="btn btn-primary" type="submit">追加</button>
    </div>
  </div>
</form>
<div class="card border-secondary">
  <div class="card-header">ドメイン一覧</div>
  <div class="table-responsive">
    <table class="table table-dark table-striped mb-0">
      <thead><tr><th>ドメイン</th><th>最終確認</th><th>HTTPS</th></tr></thead>
      <tbody>${rows}</tbody>
    </table>
  </div>
</div>`
  );

  const form = root.querySelector('#domain-form') as HTMLFormElement;
  form?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const name = new FormData(form).get('name') as string;
    const data = await aruaruApi.addDomain(name);
    if (data.ok) await renderEasyweb(root);
    else alert(data.error || 'error');
  });
}
