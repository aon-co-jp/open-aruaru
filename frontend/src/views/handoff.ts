import { aruaruApi } from '../api';
import { alertBox, escapeHtml, layout } from '../components/layout';

const TARGETS = [
  { id: 'cursor', name: 'Cursor' },
  { id: 'claude', name: 'Claude' },
  { id: 'gemini', name: 'Gemini' },
  { id: 'deepseek', name: 'DeepSeek' },
  { id: 'aruaru', name: 'aruaru（戻す）' },
];

const USE_CASES = [
  { id: 'development', label: 'アプリ/Web 開発（Tauri+Poem+Cosmo）' },
  { id: 'english_conversation', label: '英会話（音声・翻訳・低単価）' },
  { id: 'cheap', label: '低単価 LLM' },
  { id: 'learning', label: '学習・AI 先生' },
  { id: 'server_ops', label: 'VPS 運用' },
];

export async function renderHandoff(root: HTMLElement): Promise<void> {
  const [targets, providers] = await Promise.all([
    aruaruApi.handoffTargets(),
    aruaruApi.aiProviders(),
  ]);

  root.innerHTML = layout(
    '第二の Cursor — 引き継ぎ & AI 選択',
    `
<p class="text-muted">aruaru ↔ Cursor / Claude / Gemini / DeepSeek を<strong>文章（Markdown）</strong>で双方向引き継ぎ。DATA footprint 互換。</p>

<div class="row g-3">
  <div class="col-lg-6">
    <div class="card border-secondary h-100">
      <div class="card-header">📤 エクスポート（aruaru → 他ツール）</div>
      <div class="card-body">
        <div class="mb-2">
          <label class="form-label">プロジェクト名</label>
          <input id="ho-project" class="form-control form-control-sm" value="aruaru" />
        </div>
        <div class="mb-2">
          <label class="form-label">引き継ぎ先</label>
          <select id="ho-to" class="form-select form-select-sm">
            ${TARGETS.map((t) => `<option value="${t.id}">${escapeHtml(t.name)}</option>`).join('')}
          </select>
        </div>
        <div class="mb-2">
          <label class="form-label">概要（任意）</label>
          <textarea id="ho-summary" class="form-control form-control-sm" rows="2" placeholder="第二の KUSANAGI + 第二の Cursor"></textarea>
        </div>
        <button type="button" class="btn btn-primary btn-sm" id="ho-export">Markdown を生成</button>
        <button type="button" class="btn btn-outline-secondary btn-sm" id="ho-copy">コピー</button>
        <pre id="ho-export-out" class="small mt-2 bg-dark text-light p-2 rounded" style="max-height:240px;overflow:auto"></pre>
      </div>
    </div>
  </div>
  <div class="col-lg-6">
    <div class="card border-secondary h-100">
      <div class="card-header">📥 インポート（他ツール → aruaru）</div>
      <div class="card-body">
        <p class="small text-muted">Cursor / Claude 等からコピーした Markdown を貼り付け</p>
        <textarea id="ho-import-in" class="form-control form-control-sm mb-2" rows="8" placeholder="---&#10;project: myapp&#10;---"></textarea>
        <button type="button" class="btn btn-primary btn-sm" id="ho-import">aruaru に取り込む</button>
        <div id="ho-import-msg" class="mt-2"></div>
      </div>
    </div>
  </div>
</div>

<div class="card border-secondary mt-3">
  <div class="card-header">🤖 AI おすすめ（用途別・選択可能）</div>
  <div class="card-body">
    <div class="mb-2">
      <label class="form-label">用途</label>
      <select id="ai-use-case" class="form-select form-select-sm">
        ${USE_CASES.map((u) => `<option value="${u.id}">${escapeHtml(u.label)}</option>`).join('')}
      </select>
    </div>
    <button type="button" class="btn btn-outline-primary btn-sm" id="ai-recommend">おすすめを表示</button>
    <div id="ai-list" class="mt-3"></div>
    <p class="small text-muted mt-2 mb-0">スタック: Tauri + Poem + PostgreSQL + aruaru-db + WunderGraph Cosmo</p>
  </div>
</div>

<div class="card border-secondary mt-3">
  <div class="card-header">📱 プラットフォーム</div>
  <div class="card-body small">
    <ul class="mb-0">
      <li><strong>aruaru-desktop</strong> — Windows（インストーラー付き）</li>
      <li><strong>aruaru-web</strong> — VPS ブラウザ版</li>
      <li><strong>aruaru-smartphone</strong> — aruaru-android / aruaru-iPhone（ハイブリッド）</li>
    </ul>
  </div>
</div>`
  );

  const exportOut = root.querySelector('#ho-export-out') as HTMLElement;

  root.querySelector('#ho-export')?.addEventListener('click', async () => {
    const project = (root.querySelector('#ho-project') as HTMLInputElement).value.trim();
    const to = (root.querySelector('#ho-to') as HTMLSelectElement).value;
    const summary = (root.querySelector('#ho-summary') as HTMLTextAreaElement).value.trim();
    exportOut.textContent = '生成中…';
    try {
      const res = await aruaruApi.handoffExport({ project, to, from: 'aruaru-web', summary, goals: [] });
      if (res.ok && res.markdown) {
        exportOut.textContent = res.markdown;
      } else {
        exportOut.textContent = res.error || 'failed';
      }
    } catch (e) {
      exportOut.textContent = String(e);
    }
  });

  root.querySelector('#ho-copy')?.addEventListener('click', async () => {
    const text = exportOut.textContent || '';
    if (text) await navigator.clipboard.writeText(text);
  });

  root.querySelector('#ho-import')?.addEventListener('click', async () => {
    const text = (root.querySelector('#ho-import-in') as HTMLTextAreaElement).value;
    const msg = root.querySelector('#ho-import-msg')!;
    try {
      const res = await aruaruApi.handoffImport({ text, source: 'paste' });
      msg.innerHTML = res.ok
        ? alertBox('success', `取り込み完了: ${res.packet?.project ?? ''}`)
        : alertBox('danger', res.error || 'error');
    } catch (e) {
      msg.innerHTML = alertBox('danger', String(e));
    }
  });

  root.querySelector('#ai-recommend')?.addEventListener('click', async () => {
    const useCase = (root.querySelector('#ai-use-case') as HTMLSelectElement).value;
    const list = root.querySelector('#ai-list')!;
    list.innerHTML = '<p class="text-muted">読み込み中…</p>';
    const res = await aruaruApi.aiRecommend(useCase);
    if (!res.recommended?.length) {
      list.innerHTML = alertBox('info', '該当なし');
      return;
    }
    list.innerHTML = res.recommended
      .map(
        (p) => `
<div class="border border-secondary rounded p-2 mb-2 ${localStorage.getItem('aruaru_ai_' + useCase) === p.id ? 'border-primary' : ''}">
  <strong>${escapeHtml(p.name)}</strong>
  ${p.score != null ? `<span class="badge text-bg-primary ms-1">${p.score}pt</span>` : ''}
  <span class="badge text-bg-secondary ms-1">${escapeHtml(p.cost_tier)}</span>
  <span class="badge text-bg-dark ms-1">${escapeHtml(p.vendor)}</span>
  <p class="small mb-1">${escapeHtml(p.strengths.join(' · '))}</p>
  ${p.reason ? `<p class="small text-muted mb-1">${escapeHtml(p.reason)}</p>` : ''}
  ${p.speech_quality ? `<span class="small text-muted">音声: ${p.speech_quality}</span> ` : ''}
  ${p.translation_quality ? `<span class="small text-muted">翻訳: ${p.translation_quality}</span>` : ''}
  <button type="button" class="btn btn-sm btn-outline-primary mt-1 select-ai-web" data-id="${escapeHtml(p.id)}" data-uc="${escapeHtml(useCase)}">選択</button>
</div>`
      )
      .join('');
    list.querySelectorAll('.select-ai-web').forEach((btn) => {
      btn.addEventListener('click', () => {
        const id = (btn as HTMLButtonElement).dataset.id!;
        const uc = (btn as HTMLButtonElement).dataset.uc!;
        localStorage.setItem('aruaru_ai_' + uc, id);
        (btn as HTMLButtonElement).textContent = '選択済み';
      });
    });
  });

  void targets;
  void providers;
}
