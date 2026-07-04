let currentVps  = '';
let currentPath = '/';

async function loadVpsList() {
  const res  = await fetch('/api/vps/');
  const list = await res.json();
  const sel  = document.getElementById('vpsSelect');
  list.forEach(v => { sel.innerHTML += `<option value="${v.id}">${v.name} (${v.ip_address})</option>`; });
  // URL パラメータで VPS が指定されていれば自動選択
  const params = new URLSearchParams(location.search);
  if (params.get('vps')) { sel.value = params.get('vps'); changeVps(); }
}

async function changeVps() {
  currentVps  = document.getElementById('vpsSelect').value;
  currentPath = '/';
  if (currentVps) await loadDir('/');
}
document.getElementById('vpsSelect').addEventListener('change', changeVps);

async function loadDir(path) {
  currentPath = path;
  setBreadcrumb(path);
  const res  = await fetch(`/api/sftp/${currentVps}/ls?path=${encodeURIComponent(path)}`);
  const list = await res.json();
  const el   = document.getElementById('fileList');
  el.innerHTML = '';
  if (path !== '/') {
    el.innerHTML += `<div class="file-row" onclick="navigate('${parent_path(path)}')">
      <span class="file-icon">↩️</span><span class="file-name">..</span></div>`;
  }
  list.forEach(f => {
    el.innerHTML += `
    <div class="file-row" onclick="${f.is_dir ? `navigate('${f.path}')` : `downloadFile('${f.path}')`}">
      <span class="file-icon">${f.is_dir ? '📁' : fileIcon(f.name)}</span>
      <span class="file-name">${f.name}</span>
      <span class="file-size">${f.is_dir ? '—' : fmt_size(f.size)}</span>
      <span class="file-date">${f.modified.slice(0,19).replace('T',' ')}</span>
      <span style="margin-left:.5rem">
        ${!f.is_dir ? `<button onclick="event.stopPropagation();downloadFile('${f.path}')"
          style="background:transparent;border:1px solid #334155;color:#94a3b8;
                 padding:.15rem .5rem;border-radius:4px;cursor:pointer;font-size:.75rem">DL</button>` : ''}
        <button onclick="event.stopPropagation();deleteFile('${f.path}',${f.is_dir})"
          style="background:transparent;border:1px solid #ef4444;color:#ef4444;
                 padding:.15rem .5rem;border-radius:4px;cursor:pointer;font-size:.75rem">削除</button>
      </span>
    </div>`;
  });
}

function navigate(path) { loadDir(path); }
function refresh()      { loadDir(currentPath); }

function setBreadcrumb(path) {
  const parts = path.split('/').filter(Boolean);
  let html = '<span onclick="navigate(\'/\')">/ </span>';
  let acc  = '';
  parts.forEach(p => { acc += '/' + p; html += `<span onclick="navigate('${acc}')">${p} / </span>`; });
  document.getElementById('breadcrumb').innerHTML = html;
}

function downloadFile(path) {
  window.location.href = `/api/sftp/${currentVps}/download?path=${encodeURIComponent(path)}`;
}

async function deleteFile(path, isDir) {
  if (!confirm(`${path} を削除しますか？`)) return;
  await fetch(`/api/sftp/${currentVps}/delete`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, is_dir: isDir }),
  });
  refresh();
}

function uploadFile() { document.getElementById('fileInput').click(); }

async function handleFileSelect(e) {
  const files = e.target.files;
  for (const file of files) {
    const fd = new FormData();
    fd.append('file', file);
    await fetch(`/api/sftp/${currentVps}/upload?path=${encodeURIComponent(currentPath)}`, {
      method: 'POST', body: fd,
    });
  }
  refresh();
}

async function newFolder() {
  const name = prompt('フォルダ名を入力してください');
  if (!name) return;
  await fetch(`/api/sftp/${currentVps}/mkdir`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path: currentPath + '/' + name }),
  });
  refresh();
}

// ドラッグ＆ドロップアップロード
const main = document.getElementById('mainArea');
const overlay = document.getElementById('dropOverlay');
main.addEventListener('dragover', e => { e.preventDefault(); overlay.classList.add('active'); });
main.addEventListener('dragleave', () => overlay.classList.remove('active'));
main.addEventListener('drop', async e => {
  e.preventDefault(); overlay.classList.remove('active');
  const files = e.dataTransfer.files;
  for (const file of files) {
    const fd = new FormData(); fd.append('file', file);
    await fetch(`/api/sftp/${currentVps}/upload?path=${encodeURIComponent(currentPath)}`, { method:'POST', body:fd });
  }
  refresh();
});

async function runAiFile() {
  const instr = document.getElementById('aiInstruction').value.trim();
  if (!instr || !currentVps) return;
  const res  = await fetch('/api/ai/file-operation', {
    method:'POST', headers:{'Content-Type':'application/json'},
    body: JSON.stringify({ vps_id: currentVps, instruction: instr, current_path: currentPath }),
  });
  const data = await res.json();
  alert('AI 実行結果: ' + (data.message || JSON.stringify(data)));
  refresh();
}

function parent_path(p) { return p.split('/').slice(0,-1).join('/') || '/'; }
function fmt_size(b) { return b>1048576 ? (b/1048576).toFixed(1)+'MB' : b>1024 ? (b/1024).toFixed(1)+'KB' : b+'B'; }
function fileIcon(name) {
  const ext = name.split('.').pop().toLowerCase();
  const icons = {rs:'🦀',toml:'⚙️',html:'🌐',css:'🎨',js:'📜',json:'📋',md:'📝',
    sh:'📜',py:'🐍',rb:'💎',go:'🐹',php:'🐘',sql:'🗄️',zip:'📦',tar:'📦',gz:'📦',jpg:'🖼️',png:'🖼️'};
  return icons[ext] || '📄';
}

loadVpsList();
