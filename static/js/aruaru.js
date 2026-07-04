(function () {
  async function api(path, opts) {
    const r = await fetch('/api' + path, { headers: { 'Content-Type': 'application/json' }, ...opts });
    return r.json();
  }

  function bindForm(id, endpoint, resultId, okMsg) {
    const form = document.getElementById(id);
    if (!form) return;
    form.addEventListener('submit', async (e) => {
      e.preventDefault();
      const fd = new FormData(form);
      const body = Object.fromEntries(fd.entries());
      const el = document.getElementById(resultId);
      const data = await api(endpoint, { method: 'POST', body: JSON.stringify(body) });
      if (data.ok) {
        if (el) {
          el.className = 'result ok';
          el.textContent = okMsg + (data.public_url ? '\nURL: ' + data.public_url : '') + (data.magic_link ? '\n\n' + data.magic_link : '');
        }
        if (data.magic_link && el) {
          const a = document.createElement('a');
          a.href = data.magic_link;
          a.textContent = 'ログインリンクを開く';
          el.appendChild(document.createElement('br'));
          el.appendChild(a);
        } else if (id.indexOf('domain') >= 0 || id === 'swap-domain-form') {
          location.reload();
        }
      } else if (el) {
        el.className = 'result err';
        el.textContent = data.error || 'error';
      }
    });
  }

  bindForm('register-form', '/register', 'register-result', '登録しました。');
  bindForm('login-form', '/login', 'login-result', 'リンクを発行しました。');
  bindForm('platform-domain-form', '/platform/domains', 'platform-domain-result', 'ドメインを登録しました。');

  const swapForm = document.getElementById('swap-domain-form');
  if (swapForm) {
    swapForm.addEventListener('submit', async (e) => {
      e.preventDefault();
      const fd = new FormData(swapForm);
      const body = Object.fromEntries(fd.entries());
      const data = await api('/platform/domains/swap', { method: 'POST', body: JSON.stringify(body) });
      if (data.ok) location.reload();
      else alert(data.error || 'error');
    });
  }

  document.querySelectorAll('.set-primary').forEach(function (btn) {
    btn.addEventListener('click', async function () {
      const data = await api('/platform/domains/primary', {
        method: 'POST',
        body: JSON.stringify({ name: btn.dataset.name }),
      });
      if (data.ok) location.reload();
      else alert(data.error || 'error');
    });
  });

  document.querySelectorAll('.delete-domain').forEach(function (btn) {
    btn.addEventListener('click', async function () {
      if (!confirm('ドメイン「' + btn.dataset.name + '」を削除しますか？')) return;
      const data = await api('/platform/domains/delete', {
        method: 'POST',
        body: JSON.stringify({ name: btn.dataset.name }),
      });
      if (data.ok) location.reload();
      else alert(data.error || 'error');
    });
  });

  const df = document.getElementById('domain-form');
  if (df) {
    df.addEventListener('submit', async (e) => {
      e.preventDefault();
      const fd = new FormData(df);
      await api('/domains', { method: 'POST', body: JSON.stringify(Object.fromEntries(fd.entries())) });
      location.reload();
    });
  }

  async function loadReminders() {
    const box = document.getElementById('reminder-box');
    if (!box) return;
    const data = await api('/reminders');
    if (!data.due || !data.items.length) { box.classList.add('hidden'); return; }
    box.classList.remove('hidden');
    box.innerHTML = '<h3>開発リマインダー</h3>' + data.items.map(i => '<p>' + i.question + '</p>').join('');
  }
  loadReminders();
})();
