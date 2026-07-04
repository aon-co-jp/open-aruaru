// ===== VPS 管理 =====
async function loadVps() {
  const res  = await fetch('/api/vps/');
  const list = await res.json();
  const el   = document.getElementById('vpsList');
  el.innerHTML = list.length ? '' : '<div class="loading">VPS が登録されていません</div>';
  list.forEach(v => {
    const dot  = `<span class="status-dot status-${v.status}"></span>`;
    const stat = { online:'🟢 オンライン', offline:'🔴 オフライン', unknown:'⚪ 未確認' }[v.status] || v.status;
    el.innerHTML += `
    <div class="vps-item">
      <div class="vps-info">
        <div class="vps-name">${dot}${v.name}</div>
        <div class="vps-ip">${v.ip_address}:${v.port} &nbsp;/&nbsp; ${v.username} &nbsp;/&nbsp; ${v.auth_method}</div>
        <div style="font-size:.78rem;color:var(--muted);margin-top:.15rem">${stat}</div>
      </div>
      <div class="vps-actions">
        <a class="btn-term"  href="/terminal?vps=${v.id}">💻 ターミナル</a>
        <a class="btn-files" href="/files?vps=${v.id}">📁 ファイル</a>
        <button class="btn-secondary" style="font-size:.8rem" onclick="pingVps('${v.id}',this)">Ping</button>
        <button onclick="deleteVps('${v.id}')"
          style="background:transparent;border:1px solid var(--danger);color:var(--danger);
                 padding:.3rem .65rem;border-radius:6px;cursor:pointer;font-size:.8rem">削除</button>
      </div>
    </div>`;
  });
}

async function pingVps(id, btn) {
  btn.disabled = true; btn.textContent = '確認中…';
  const res  = await fetch(`/api/vps/${id}/ping`, { method: 'POST' });
  const data = await res.json();
  btn.disabled = false; btn.textContent = 'Ping';
  loadVps();
}

async function deleteVps(id) {
  if (!confirm('このVPSを削除しますか？')) return;
  await fetch(`/api/vps/${id}`, { method: 'DELETE' });
  loadVps();
}

function openVpsModal() {
  loadKeyOptions();
  document.getElementById('vpsModal').classList.remove('hidden');
}
function closeVpsModal() {
  document.getElementById('vpsModal').classList.add('hidden');
  document.getElementById('detectResult').textContent = '';
}

// ===== IP アドレス自動検出 =====
async function autoDetectIp() {
  const btn = document.getElementById('detectBtn');
  const res_el = document.getElementById('detectResult');
  btn.disabled = true; btn.textContent = '検出中…';
  res_el.textContent = '';

  try {
    const res  = await fetch('/api/vps/detect-ip');
    const data = await res.json();
    if (data.ip) {
      document.getElementById('vpsIp').value = data.ip;
      res_el.style.color = 'var(--success)';
      res_el.textContent = `✅ 自動検出: ${data.ip}`;
    } else {
      res_el.style.color = 'var(--warning)';
      res_el.textContent = '⚠️ 自動検出できませんでした。手動で入力してください。';
    }
  } catch (e) {
    res_el.style.color = 'var(--danger)';
    res_el.textContent = '❌ 検出失敗。手動で入力してください。';
  }
  btn.disabled = false; btn.textContent = '🔍 自動検出';
}

async function saveVps() {
  const ip = document.getElementById('vpsIp').value.trim();
  if (!ip) { alert('IP アドレスを入力してください'); return; }

  const body = {
    name:        document.getElementById('vpsName').value.trim() || ip,
    ip_address:  ip,
    port:        parseInt(document.getElementById('vpsPort').value) || 22,
    username:    document.getElementById('vpsUser').value.trim() || 'root',
    auth_method: document.querySelector('input[name="vpsAuth"]:checked').value,
    key_id:      document.getElementById('vpsKeyId').value || null,
  };

  const btn = document.querySelector('#vpsModal .btn-primary');
  btn.disabled = true; btn.textContent = '登録中…';

  const res = await fetch('/api/vps/', {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body),
  });
  btn.disabled = false; btn.textContent = '登録 & 接続確認';

  if (res.ok) {
    const data = await res.json();
    closeVpsModal();
    // 登録後すぐ Ping で状態確認
    fetch(`/api/vps/${data.id}/ping`, { method: 'POST' }).then(() => loadVps());
  } else {
    alert('登録失敗: ' + await res.text());
  }
}

// ===== SSH 鍵管理 =====
async function loadKeys() {
  const res  = await fetch('/api/keys/');
  const list = await res.json();
  const el   = document.getElementById('keyList');
  if (!el) return;
  el.innerHTML = list.length ? '' : '<div class="loading">鍵が登録されていません</div>';
  list.forEach(k => {
    el.innerHTML += `
    <div class="key-item">
      <div class="key-info">
        <div class="key-name">🔑 ${k.name} <span class="badge">${k.key_type.toUpperCase()}</span></div>
        <div class="key-fp">${k.fingerprint}</div>
      </div>
      <a href="/api/keys/${k.id}/pubkey"
        style="color:var(--primary);font-size:.82rem;margin-right:.6rem;text-decoration:none">
        📋 公開鍵 DL</a>
      <button onclick="deleteKey('${k.id}')"
        style="background:transparent;border:1px solid var(--danger);color:var(--danger);
               padding:.25rem .6rem;border-radius:5px;cursor:pointer;font-size:.78rem">削除</button>
    </div>`;
  });
}

async function loadKeyOptions() {
  const res  = await fetch('/api/keys/');
  const list = await res.json();
  const sel  = document.getElementById('vpsKeyId');
  sel.innerHTML = '<option value="">— 選択してください —</option>';
  list.forEach(k => { sel.innerHTML += `<option value="${k.id}">${k.name} (${k.key_type})</option>`; });
}

function openKeyModal()  { document.getElementById('keyModal').classList.remove('hidden'); }
function closeKeyModal() {
  document.getElementById('keyModal').classList.add('hidden');
  document.getElementById('keyResult').classList.add('hidden');
  document.getElementById('keyName').value = '';
}

async function doGenerateKey() {
  const name = document.getElementById('keyName').value.trim();
  const type = document.querySelector('input[name="keyType"]:checked').value;
  if (!name) { alert('鍵の名前を入力してください'); return; }

  const btn = document.getElementById('genKeyBtn');
  btn.disabled = true; btn.textContent = '生成中…';

  const res = await fetch('/api/keys/', {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, key_type: type }),
  });
  btn.disabled = false; btn.textContent = '自動生成';

  if (res.ok) {
    const kp = await res.json();
    const r  = document.getElementById('keyResult');
    r.classList.remove('hidden');
    r.innerHTML = `
      <div>✅ 生成完了 — <strong>${kp.name}</strong> (${kp.key_type.toUpperCase()})</div>
      <div style="font-size:.78rem;margin:.3rem 0">Fingerprint: ${kp.fingerprint}</div>
      <div style="font-size:.82rem;margin-top:.6rem;color:var(--muted)">
        公開鍵（VPS の <code>~/.ssh/authorized_keys</code> に追加してください）:
      </div>
      <textarea readonly style="width:100%;background:var(--bg);border:1px solid var(--border);
        border-radius:6px;color:var(--text);font-family:monospace;font-size:.75rem;
        padding:.5rem;margin-top:.3rem;resize:vertical;min-height:55px">${kp.public_key}</textarea>
      <a href="/api/keys/${kp.id}/pubkey" class="btn-primary"
        style="display:inline-block;margin-top:.5rem;text-decoration:none;font-size:.85rem">
        📥 公開鍵をダウンロード</a>`;
    loadKeys();
  } else {
    alert('生成失敗: ' + await res.text());
  }
}

async function deleteKey(id) {
  if (!confirm('この鍵を削除しますか？')) return;
  await fetch(`/api/keys/${id}`, { method: 'DELETE' });
  loadKeys();
}

// 初期化
loadVps();
loadKeys();
