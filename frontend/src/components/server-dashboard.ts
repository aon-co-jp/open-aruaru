import { escapeHtml } from './layout';

export type ServerStatusPayload = {
  ok?: boolean;
  hostname?: string;
  cpu_percent?: number | null;
  memory_percent?: number | null;
  disk_percent?: number | null;
  ssl?: { ok: boolean; label: string };
  domain?: { ok: boolean; primary?: string; count?: number };
  backup?: { ok: boolean; label: string };
  services?: {
    aruaru?: string;
    nginx?: boolean;
    port_3000?: boolean;
  };
  uptime_secs?: number;
  public_url?: string;
  checked_at?: string;
};

function pctBar(label: string, value: number | null | undefined): string {
  const v = value ?? null;
  const display = v != null ? `${Math.round(v)}%` : '—';
  const width = v != null ? Math.min(100, Math.max(0, v)) : 0;
  const color = v != null && v >= 85 ? 'danger' : v != null && v >= 70 ? 'warning' : 'success';
  return `
<div class="mb-3">
  <div class="d-flex justify-content-between small mb-1">
    <span>${escapeHtml(label)}</span><strong>${display}</strong>
  </div>
  <div class="progress" style="height:8px">
    <div class="progress-bar bg-${color}" style="width:${width}%"></div>
  </div>
</div>`;
}

function statusBadge(ok: boolean, label: string): string {
  const icon = ok ? '🟢' : '🟡';
  const cls = ok ? 'text-success' : 'text-warning';
  return `<div class="${cls}"><span>${icon}</span> ${escapeHtml(label)}</div>`;
}

export function renderServerDashboard(status: ServerStatusPayload, opts?: { showActions?: boolean }): string {
  const overall =
    status.ssl?.ok && status.domain?.ok && status.services?.port_3000 !== false;
  const actions = opts?.showActions !== false ? `
<div class="row g-2 mt-3">
  <div class="col-6 col-md-4"><a class="btn btn-outline-primary w-100 btn-sm" href="#/settings/domain">② ドメイン追加</a></div>
  <div class="col-6 col-md-4"><a class="btn btn-outline-primary w-100 btn-sm" href="#/settings/https">③ SSL追加</a></div>
  <div class="col-6 col-md-4"><a class="btn btn-outline-primary w-100 btn-sm" href="#/features">⑤ 機能一覧</a></div>
</div>` : '';

  const uptime =
    status.uptime_secs != null
      ? `${Math.floor(status.uptime_secs / 86400)}日 ${Math.floor((status.uptime_secs % 86400) / 3600)}時間`
      : '—';

  return `
<div class="card border-secondary mb-3">
  <div class="card-header d-flex justify-content-between align-items-center">
    <span>AI Server Manager</span>
    <span class="badge ${overall ? 'text-bg-success' : 'text-bg-warning'}">${overall ? '正常' : '要確認'}</span>
  </div>
  <div class="card-body">
    <p class="small text-muted mb-3">ホスト: <code>${escapeHtml(status.hostname || '—')}</code>
      ${status.public_url ? ` · <a href="${escapeHtml(status.public_url)}">${escapeHtml(status.public_url)}</a>` : ''}</p>
    ${pctBar('CPU', status.cpu_percent)}
    ${pctBar('メモリ', status.memory_percent)}
    ${pctBar('ディスク', status.disk_percent)}
    <div class="row g-2 small mt-2">
      <div class="col-md-4">${statusBadge(!!status.ssl?.ok, `SSL: ${status.ssl?.label || '—'}`)}</div>
      <div class="col-md-4">${statusBadge(!!status.domain?.ok, `ドメイン: ${status.domain?.primary || '未設定'}`)}</div>
      <div class="col-md-4">${statusBadge(!!status.backup?.ok, `バックアップ: ${status.backup?.label || '—'}`)}</div>
    </div>
    <hr class="border-secondary" />
    <div class="small text-muted">
      aruaru: <code>${escapeHtml(status.services?.aruaru || '—')}</code>
      · nginx: ${status.services?.nginx ? '稼働' : '—'}
      · :3000 ${status.services?.port_3000 ? 'LISTEN' : 'closed'}
      · 稼働 ${uptime}
    </div>
    ${actions}
  </div>
</div>`;
}
