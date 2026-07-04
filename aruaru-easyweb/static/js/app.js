// ===== ドメイン一覧の読み込み =====
async function loadDomains() {
  const res     = await fetch('/api/domain/');
  const domains = await res.json();
  const list    = document.getElementById('domainList');
  const alerts  = document.getElementById('ssl-alerts');
  list.innerHTML = ''; alerts.innerHTML = '';

  if (!domains.length) {
    list.innerHTML = '<div class="loading">ドメインが登録されていません</div>';
    return;
  }

  domains.forEach(d => {
    // SSL 期限アラート
    if (d.https_enabled && d.cert_expires_at) {
      const days = Math.floor((new Date(d.cert_expires_at) - Date.now()) / 86400000);
      if (days < 30) {
        alerts.innerHTML += `<div class="alert">⚠️ <b>${d.name}</b> の証明書が残り <b>${days} 日</b> で期限切れです</div>`;
      }
    }
    list.insertAdjacentHTML('beforeend', renderDomain(d));
  });

  // HTTPS トグルイベント
  list.querySelectorAll('.tog-https').forEach(el => {
    el.addEventListener('change', async () => {
      await fetch(`/api/domain/${el.dataset.id}/https`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled: el.checked }),
      });
      loadDomains();
    });
  });

  // Webサーバー切り替えボタン
  list.querySelectorAll('.btn-switch-ws').forEach(el => {
    el.addEventListener('click', async () => {
      const to = el.dataset.current === 'nginx' ? 'apache' : 'nginx';
      if (!confirm(`${el.dataset.domain} を ${to.toUpperCase()} に切り替えます。よろしいですか？`)) return;
      el.disabled = true; el.textContent = '切替中…';
      await fetch(`/api/domain/${el.dataset.id}/webserver`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ webserver: to }),
      });
      loadDomains();
    });
  });

  // 削除ボタン
  list.querySelectorAll('.btn-del').forEach(el => {
    el.addEventListener('click', async () => {
      if (!confirm(`${el.dataset.domain} を削除します。設定ファイルも自動削除されます。`)) return;
      await fetch(`/api/domain/${el.dataset.id}`, { method: 'DELETE' });
      loadDomains();
    });
  });
}

function renderDomain(d) {
  const bare = d.name.replace(/^www\./, '');
  let certHtml = '';
  if (d.https_enabled && d.cert_expires_at) {
    const days = Math.floor((new Date(d.cert_expires_at) - Date.now()) / 86400000);
    const cls  = days > 30 ? 'cert-ok' : days > 7 ? 'cert-warn' : 'cert-err';
    certHtml = `<span class="${cls}">🔒 残り ${days} 日</span>`;
  }
  const wsNext = d.webserver === 'nginx' ? 'Apache' : 'Nginx';
  return `
<div class="domain-item">
  <div>
    <div class="domain-name">${d.is_app_domain ? '⭐ ' : ''}${bare}</div>
    <div class="domain-meta">
      <span class="www-label">www.${bare} → 自動リダイレクト</span>
      <span class="ws-badge">${d.webserver.toUpperCase()}</span>
      ${certHtml}
    </div>
  </div>
  <div class="domain-controls">
    <label class="ctrl-label">HTTPS
      <label class="toggle">
        <input type="checkbox" class="tog-https" data-id="${d.id}" ${d.https_enabled ? 'checked' : ''}>
        <span class="slider"></span>
      </label>
    </label>
    <label class="ctrl-label">Gzip
      <label class="toggle">
        <input type="checkbox" ${d.gzip_enabled ? 'checked' : ''} disabled>
        <span class="slider"></span>
      </label>
    </label>
    <button class="btn-switch-ws btn-secondary"
      data-id="${d.id}" data-current="${d.webserver}" data-domain="${bare}">
      → ${wsNext} へ切替
    </button>
    <button class="btn-del" data-id="${d.id}" data-domain="${bare}"
      style="background:transparent;border:1px solid var(--danger);color:var(--danger);
             padding:.3rem .75rem;border-radius:6px;cursor:pointer;font-size:.8rem">
      削除
    </button>
  </div>
</div>`;
}

// ===== フレームワーク推奨 =====
document.querySelectorAll('.fw-btn').forEach(btn => {
  btn.addEventListener('click', async () => {
    document.querySelectorAll('.fw-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    const res  = await fetch('/api/webserver/recommend', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ framework: btn.dataset.fw }),
    });
    const data = await res.json();
    const rec  = document.getElementById('recommendation');
    rec.classList.remove('hidden');
    rec.innerHTML = `
      <div class="rec-server">推奨: ${data.webserver.toUpperCase()}</div>
      <div class="rec-reason">${data.reason}</div>
      <div class="rec-conf">確信度: ${data.confidence}%</div>`;
  });
});

// ===== モーダル =====
function openModal()  { document.getElementById('modal').classList.remove('hidden'); }
function closeModal() { document.getElementById('modal').classList.add('hidden'); }

async function saveDomain() {
  const name  = document.getElementById('mName').value.trim();
  if (!name) { alert('ドメイン名を入力してください'); return; }
  const ws    = document.querySelector('input[name="mWs"]:checked').value;
  const port  = document.getElementById('mPort').value || null;
  const https = document.getElementById('mHttps').checked;
  const isApp = document.getElementById('mApp').checked;

  const btn = document.querySelector('.modal-box .btn-primary');
  btn.disabled = true; btn.textContent = '登録・適用中…';

  const res = await fetch('/api/domain/', {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      name, webserver: ws,
      proxy_port: port ? parseInt(port) : null,
      https, is_app_domain: isApp,
    }),
  });

  btn.disabled = false; btn.textContent = '登録・自動適用';

  if (res.ok) {
    closeModal();
    document.getElementById('mName').value = '';
    document.getElementById('mPort').value = '';
    document.getElementById('mHttps').checked = false;
    document.getElementById('mApp').checked   = false;
    loadDomains();
  } else {
    const err = await res.text();
    alert('エラー: ' + err);
  }
}

// 起動時に読み込み
if (document.getElementById('domainList')) loadDomains();
