import { aruaruApi } from '../api';
import { alertBox, escapeHtml, layout } from '../components/layout';

export async function renderDomainSettings(root: HTMLElement): Promise<void> {
  const health = await aruaruApi.health();
  const list = await aruaruApi.platformDomains();
  const domains = list.domains || [];

  const rows =
    domains.length > 0
      ? domains
          .map((d) => {
            const badge = d.is_primary ? ' <span class="badge text-bg-primary">主</span>' : '';
            return `<tr>
  <td><code>${escapeHtml(d.name)}</code>${badge}</td>
  <td>${escapeHtml(d.last_checked || '-')}</td>
      <td>${d.https_enabled ? '<span class="text-success">ON</span>' : '<span class="text-warning">設定中…</span>'}</td>
  <td class="text-nowrap">
    <button class="btn btn-sm btn-outline-light set-primary" data-name="${escapeHtml(d.name)}">主にする</button>
    <button class="btn btn-sm btn-outline-danger delete-domain" data-name="${escapeHtml(d.name)}">削除</button>
  </td>
</tr>`;
          })
          .join('')
      : '<tr><td colspan="4" class="text-muted">未登録 — IP (<code>http://IP:3000</code>) で運用中</td></tr>';

  root.innerHTML = layout(
    'aruaru 運用ドメイン',
    `
<p class="text-muted">aruaru 本体を公開するドメイン。DNS で VPS の IP を向けてから登録してください。<strong>登録後は自動で HTTPS 化し、証明書の更新も自動メンテナンスします。</strong></p>
<p>現在の URL: <code>${escapeHtml(health.public_url)}</code></p>
<form id="platform-domain-form" class="card border-secondary mb-3">
  <div class="card-body">
    <label class="form-label">ドメイン</label>
    <div class="input-group">
      <input class="form-control" name="name" placeholder="aruaru.example.com" required />
      <button class="btn btn-primary" type="submit">登録</button>
    </div>
    <div id="platform-domain-result" class="mt-2"></div>
  </div>
</form>
<div class="card border-secondary mb-3">
  <div class="card-header">登録済み</div>
  <div class="table-responsive">
    <table class="table table-dark table-striped mb-0">
      <thead><tr><th>ドメイン</th><th>最終確認</th><th>HTTPS</th><th>操作</th></tr></thead>
      <tbody>${rows}</tbody>
    </table>
  </div>
</div>
<form id="swap-domain-form" class="card border-secondary">
  <div class="card-header">ドメイン入れ替え</div>
  <div class="card-body row g-2">
    <div class="col-md-5">
      <label class="form-label">現在の主</label>
      <input class="form-control" name="from" placeholder="old.example.com" />
    </div>
    <div class="col-md-5">
      <label class="form-label">新しい主</label>
      <input class="form-control" name="to" placeholder="new.example.com" />
    </div>
    <div class="col-md-2 d-flex align-items-end">
      <button class="btn btn-primary w-100" type="submit">入れ替え</button>
    </div>
  </div>
</form>`
  );

  bindDomainForms(root);
}

function bindDomainForms(root: HTMLElement): void {
  const pf = root.querySelector('#platform-domain-form') as HTMLFormElement;
  pf?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const name = new FormData(pf).get('name') as string;
    const box = root.querySelector('#platform-domain-result')!;
    const data = await aruaruApi.addPlatformDomain(name);
    if (data.ok) {
      box.innerHTML = alertBox(
        'success',
        (data as { https?: { message?: string } }).https?.message ||
          '登録しました。HTTPS を自動設定中です…'
      );
      setTimeout(() => void renderDomainSettings(root), 2000);
    } else box.innerHTML = alertBox('danger', data.error || 'error');
  });

  root.querySelectorAll<HTMLButtonElement>('.set-primary').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const data = await aruaruApi.setPrimaryDomain(btn.dataset.name!);
      if (data.ok) await renderDomainSettings(root);
      else alert(data.error || 'error');
    });
  });

  root.querySelectorAll<HTMLButtonElement>('.delete-domain').forEach((btn) => {
    btn.addEventListener('click', async () => {
      if (!confirm(`ドメイン「${btn.dataset.name}」を削除しますか？`)) return;
      const data = await aruaruApi.deletePlatformDomain(btn.dataset.name!);
      if (data.ok) await renderDomainSettings(root);
      else alert(data.error || 'error');
    });
  });

  const sf = root.querySelector('#swap-domain-form') as HTMLFormElement;
  sf?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const fd = new FormData(sf);
    const data = await aruaruApi.swapDomains(
      fd.get('from') as string,
      fd.get('to') as string
    );
    if (data.ok) await renderDomainSettings(root);
    else alert(data.error || 'error');
  });
}
