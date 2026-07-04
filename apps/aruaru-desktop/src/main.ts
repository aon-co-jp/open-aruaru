import { invoke } from "@tauri-apps/api/core";

type DesktopConfig = {
  data_dir: string;
  root_dir: string;
  server_url: string;
  os_drive: string;
  data_drive: string;
};

type FootprintScan = {
  marker_path: string;
  data_dir: string;
  app_dir: string;
  data_drive: string;
  updated_at: string;
};

type DriveInfo = { letter: string; label: string; free_gb: number };

type DesktopFeatureItem = {
  id: string;
  name: string;
  description: string;
  category: string;
  platform: string;
  runnable: boolean;
  source: string;
};

type ServerStatusPayload = {
  hostname?: string;
  cpu_percent?: number | null;
  memory_percent?: number | null;
  disk_percent?: number | null;
  ssl?: { ok: boolean; label: string };
  domain?: { ok: boolean; primary?: string };
  backup?: { ok: boolean; label: string };
  services?: { aruaru?: string; nginx?: boolean; port_3000?: boolean };
  uptime_secs?: number;
  public_url?: string;
};

type VpsWizardInput = {
  provider: string;
  host: string;
  user: string;
  key_path: string;
  remote_dir: string;
  save_key_copy: boolean;
};

type WizardCheck = { id: string; label: string; ok: boolean; message: string };

type WizardTestResult = {
  ok: boolean;
  checks: WizardCheck[];
  connection?: VpsConnectionStatus;
  public_key?: string;
  fix_hint?: string;
};

type ServerDashboard = {
  ssh_ok: boolean;
  host: string;
  status?: ServerStatusPayload;
  error?: string;
  checked_at: string;
};

const PROVIDERS = [
  ["conoha", "ConoHa VPS"],
  ["sakura", "さくらのVPS"],
  ["xserver", "Xserver VPS"],
  ["aws", "AWS"],
  ["azure", "Azure"],
  ["gcp", "Google Cloud"],
  ["oracle", "Oracle Cloud"],
  ["other", "その他"],
] as const;

type VpsConnectionStatus = {
  ssh_ok: boolean;
  host: string;
  user: string;
  key_path: string;
  key_exists: boolean;
  remote_dir: string;
  hostname?: string;
  port_3000?: boolean;
  systemd?: string;
  aruaru_health?: { version?: string; database?: { backend?: string } };
  public_url?: string;
  error?: string;
  checked_at: string;
};

const panels = document.querySelector("#panels")!;
const serverStatus = document.querySelector("#server-status")!;
const vpsStatus = document.querySelector("#vps-status")!;

let config: DesktopConfig = {
  data_dir: "F:\\aruaru-data",
  root_dir: "F:\\aruaru",
  server_url: "http://127.0.0.1:3000",
  os_drive: "C:",
  data_drive: "F:",
};

async function loadConfig() {
  config = await invoke<DesktopConfig>("get_desktop_config");
  await refreshServerStatus();
  await refreshVpsStatus();
}

async function refreshVpsStatus() {
  if (!vpsStatus) return;
  try {
    const st = await invoke<VpsConnectionStatus>("check_vps_connection", {
      rootDir: config.root_dir,
    });
    vpsStatus.textContent = formatVpsStatusLine(st);
    vpsStatus.className = st.ssh_ok ? "ok" : "ng";
  } catch (e) {
    vpsStatus.textContent = `VPS: 確認失敗 (${String(e)})`;
    vpsStatus.className = "ng";
  }
}

function formatVpsStatusLine(st: VpsConnectionStatus): string {
  if (!st.key_exists) {
    return `VPS: 鍵なし (${st.key_path})`;
  }
  if (!st.ssh_ok) {
    return `VPS SSH: NG ${st.host} — ${st.error ?? "接続失敗"}`;
  }
  const aruaru = st.aruaru_health
    ? `aruaru OK`
  : st.port_3000
    ? "port 3000"
    : "aruaru 未起動";
  const host = st.hostname ? `${st.hostname} @ ${st.host}` : st.host;
  return `VPS SSH: OK (${host}) / ${aruaru}`;
}

function renderVpsStatusCard(st: VpsConnectionStatus): string {
  const ssh = st.ssh_ok
    ? `<span class="status-ok">SSH 接続 OK</span>`
    : `<span class="status-ng">SSH 接続 NG</span>`;
  const rows = [
    ["ホスト", `${st.user}@${st.host}`],
    ["秘密鍵", st.key_path + (st.key_exists ? "" : " （見つかりません）")],
    ["リモート dir", st.remote_dir],
    st.hostname ? ["Hostname", st.hostname] : null,
    st.port_3000 !== undefined ? ["Port 3000", st.port_3000 ? "LISTEN" : "closed"] : null,
    st.systemd ? ["systemd aruaru", st.systemd] : null,
    st.public_url ? ["公開 URL", st.public_url] : null,
    st.aruaru_health?.version ? ["aruaru 版", st.aruaru_health.version] : null,
    st.error && !st.ssh_ok ? ["エラー", st.error] : null,
    ["確認時刻", st.checked_at],
  ]
    .filter(Boolean)
    .map(
      (r) =>
        `<tr><th>${(r as [string, string])[0]}</th><td>${(r as [string, string])[1]}</td></tr>`
    )
    .join("");
  return `
<div class="vps-status-card ${st.ssh_ok ? "ok" : "ng"}">
  <div class="vps-status-head">${ssh}</div>
  <table class="vps-status-table"><tbody>${rows}</tbody></table>
</div>`;
}

function pctBar(label: string, value: number | null | undefined): string {
  const v = value ?? null;
  const display = v != null ? `${Math.round(v)}%` : "—";
  const width = v != null ? Math.min(100, Math.max(0, v)) : 0;
  const color = v != null && v >= 85 ? "danger" : v != null && v >= 70 ? "warn" : "ok";
  return `
<div class="metric">
  <div class="metric-head"><span>${label}</span><strong>${display}</strong></div>
  <div class="metric-bar"><div class="metric-fill ${color}" style="width:${width}%"></div></div>
</div>`;
}

function renderServerDashboardCard(st: ServerStatusPayload): string {
  const overall = st.ssl?.ok && st.domain?.ok && st.services?.port_3000 !== false;
  return `
<div class="vps-status-card ${overall ? "ok" : "ng"}">
  <div class="vps-status-head">AI Server Manager <span class="badge">${overall ? "正常" : "要確認"}</span></div>
  <p class="small-muted">ホスト: <code>${st.hostname || "—"}</code>${st.public_url ? ` · ${st.public_url}` : ""}</p>
  ${pctBar("CPU", st.cpu_percent)}
  ${pctBar("メモリ", st.memory_percent)}
  ${pctBar("ディスク", st.disk_percent)}
  <div class="status-row">
    <span>${st.ssl?.ok ? "🟢" : "🟡"} SSL: ${st.ssl?.label || "—"}</span>
    <span>${st.domain?.ok ? "🟢" : "🟡"} ドメイン: ${st.domain?.primary || "未設定"}</span>
    <span>${st.backup?.ok ? "🟢" : "🟡"} バックアップ: ${st.backup?.label || "—"}</span>
  </div>
  <p class="small-muted">aruaru: ${st.services?.aruaru || "—"} · nginx: ${st.services?.nginx ? "稼働" : "—"} · :3000 ${st.services?.port_3000 ? "LISTEN" : "closed"}</p>
</div>`;
}

function panelWizard(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  const providerOpts = PROVIDERS.map(([id, label]) => `<option value="${id}">${label}</option>`).join("");
  el.innerHTML = `
    <h2>接続ウィザード — SSH を意識しない VPS 接続</h2>
    <p>ConoHa 等の VPS に秘密鍵で接続します。コマンドは不要です。</p>
    <div class="wizard-steps">
      <div class="field"><label>① プロバイダー</label>
        <select id="wiz-provider">${providerOpts}</select></div>
      <div class="field"><label>② IP アドレス</label>
        <input id="wiz-host" placeholder="160.251.237.162" /></div>
      <div class="field"><label>③ ユーザー名</label>
        <input id="wiz-user" value="root" /></div>
      <div class="field"><label>④ 秘密鍵</label>
        <input id="wiz-key" placeholder="%USERPROFILE%\\.ssh\\conoha-vps.pem" />
        <div class="row" style="margin-top:0.5rem">
          <button type="button" class="secondary" id="wiz-pick-key">ファイル選択</button>
          <button type="button" class="secondary" id="wiz-gen-key">新規鍵を作成</button>
        </div></div>
      <div class="field"><label>リモートディレクトリ</label>
        <input id="wiz-remote" value="/root/aruaru" /></div>
      <label class="checkbox"><input type="checkbox" id="wiz-save-copy" checked /> 標準名 (conoha-vps.pem) にもコピー</label>
    </div>
    <div class="row">
      <button type="button" class="primary" id="wiz-test">接続テスト</button>
      <button type="button" class="primary" id="wiz-save">保存して完了</button>
    </div>
    <div id="wiz-checks"></div>
    <div id="wiz-pubkey" class="small-muted"></div>
    <div id="wiz-msg"></div>
  `;
  return el;
}

function panelDashboard(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>ダッシュボード — サーバー状態</h2>
    <p>Linux 初心者向け。CPU・メモリ・SSL などを一目で確認（コマンド不要）。</p>
    <div class="row">
      <button type="button" class="primary" id="dash-refresh">再読み込み</button>
    </div>
    <div id="dash-body"><p class="text-muted">読み込み中…</p></div>
    <div id="dash-msg"></div>
  `;
  return el;
}

async function refreshServerStatus() {
  const ok = await invoke<boolean>("ping_aruaru", { url: `${config.server_url}/api/health` });
  serverStatus.textContent = ok ? `サーバ: 接続 OK (${config.server_url})` : `サーバ: 未接続 (${config.server_url})`;
}

function showMsg(el: HTMLElement, text: string, isError = false) {
  el.className = isError ? "msg error" : "msg";
  el.textContent = text;
}

function panelFeatures(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>機能一覧 — 選択して実行</h2>
    <p>Windows ローカル機能と、接続中 VPS の機能を一覧表示します（第二の KUSANAGI）。</p>
    <div class="field">
      <label>certbot / DNS 用ドメイン</label>
      <input id="feature-domain" placeholder="example.com" />
    </div>
    <div class="field">
      <label>同期先 / バックアップ先（backup / sync 用）</label>
      <input id="feature-target" placeholder="\\\\NAS\\aruaru-backup" />
    </div>
    <div id="features-msg"></div>
    <div id="features-list"><p class="text-muted">読み込み中…</p></div>
  `;
  return el;
}

function panelDrive(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>ドライブ独立（OS と DATA 分離）</h2>
    <p>Windows の OS は通常 <code>C:</code>。aruaru のアプリ本体と DATA を <code>E:</code> など別ドライブに置き、OS だけ再インストールしても DATA を残せます。</p>
    <div class="field"><label>OS ドライブ</label><input id="os-drive" value="${config.os_drive}" /></div>
    <div class="field"><label>DATA ドライブ</label><select id="data-drive"></select></div>
    <div class="field"><label>DATA ディレクトリ</label><input id="data-dir" value="${config.data_dir}" /></div>
    <div class="field"><label>アプリ (root) ディレクトリ</label><input id="root-dir" value="${config.root_dir}" /></div>
    <div class="field"><label>aruaru サーバ URL</label><input id="server-url" value="${config.server_url}" /></div>
    <div class="row">
      <button class="secondary" id="pick-data">DATA フォルダ選択</button>
      <button class="secondary" id="pick-root">アプリフォルダ選択</button>
      <button class="primary" id="save-config">設定を保存</button>
      <button class="primary" id="bind-drive">ドライブを独立（footprint 作成）</button>
    </div>
    <div id="drive-msg"></div>
  `;
  return el;
}

function panelRecovery(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>aruaru-OS-Recovery — アクセス復活</h2>
    <p>OS 再インストール後、各ドライブに残った <code>.aruaru-footprint.json</code> をスキャンし、再インストール前と同じ DATA / アプリパスへ復元します。</p>
    <div class="row">
      <button class="primary" id="scan-fp">footprint をスキャン</button>
    </div>
    <ul class="list" id="fp-list"></ul>
    <div id="recovery-msg"></div>
  `;
  return el;
}

function panelBackup(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>バックアップ・引越し・同期</h2>
    <p>DATA をローカルバックアップ、別ドライブ / NAS への引越し・同期（robocopy ベース）を行います。OS-Recovery プラグインが必要です。</p>
    <div class="field"><label>バックアップ先</label><input id="backup-dir" value="${config.data_dir}\\recovery\\backups" /></div>
    <div class="field"><label>同期 / 引越し先（NAS 等）</label><input id="sync-target" placeholder="\\\\NAS\\aruaru-backup" /></div>
    <div class="row">
      <button class="primary" id="run-backup">バックアップ実行</button>
      <button class="primary" id="run-sync">同期実行</button>
      <button class="secondary" id="pick-backup">バックアップ先選択</button>
    </div>
    <div id="backup-msg"></div>
  `;
  return el;
}

function panelVps(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>VPS アップロード — 第二の KUSANAGI をすぐ使う</h2>
    <p>新規 VPS 借り直し / OS 再インストール後、<code>config/vps.toml</code> の SSH 設定で aruaru をアップロードし <code>vps-full-setup.sh</code> を実行します。</p>
    <ol>
      <li>ConoHa 等で VPS を用意、SSH 鍵を <code>%USERPROFILE%\\.ssh\\</code> に配置</li>
      <li><code>config/vps.toml</code> に host / remote_dir を設定</li>
      <li>接続確認 → アップロード → ブラウザでドメイン登録</li>
    </ol>
    <div id="vps-status-panel"><p class="text-muted">接続確認中…</p></div>
    <div class="row">
      <button class="secondary" id="check-vps">VPS 接続を確認</button>
    </div>
    <div class="row">
      <button class="primary" id="upload-vps">VPS にアップロード + セットアップ</button>
      <button class="secondary" id="upload-only">アップロードのみ</button>
    </div>
    <pre id="vps-log" style="margin-top:1rem;max-height:240px;overflow:auto;font-size:0.85rem"></pre>
    <div id="vps-msg"></div>
  `;
  return el;
}

const HANDOFF_TARGETS = [
  ["cursor", "Cursor"],
  ["claude", "Claude"],
  ["gemini", "Gemini"],
  ["deepseek", "DeepSeek"],
  ["aruaru", "aruaru（戻す）"],
] as const;

const AI_USE_CASES = [
  ["development", "アプリ/Web 開発（Tauri+Poem+Cosmo）"],
  ["english_conversation", "英会話（音声・翻訳）"],
  ["cheap", "低単価 LLM"],
  ["learning", "学習・AI 先生"],
  ["server_ops", "VPS 運用"],
] as const;

async function serverApi<T>(path: string, init?: RequestInit): Promise<T> {
  const base = config.server_url.replace(/\/$/, "");
  const res = await fetch(`${base}/api${path}`, {
    headers: { "Content-Type": "application/json", Accept: "application/json" },
    ...init,
  });
  return res.json() as Promise<T>;
}

function panelHandoff(): HTMLElement {
  const targetOpts = HANDOFF_TARGETS.map(([id, label]) => `<option value="${id}">${label}</option>`).join("");
  const useCaseOpts = AI_USE_CASES.map(([id, label]) => `<option value="${id}">${label}</option>`).join("");
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>引き継ぎ / AI — 第二の Cursor</h2>
    <p>aruaru ↔ Cursor / Claude / Gemini / DeepSeek を<strong>Markdown</strong>で双方向引き継ぎ。サーバ: <code>${config.server_url}</code></p>
    <div class="handoff-grid">
      <section class="handoff-card">
        <h3>📤 エクスポート（aruaru → 他ツール）</h3>
        <div class="field"><label>プロジェクト名</label><input id="ho-project" value="aruaru" /></div>
        <div class="field"><label>引き継ぎ先</label><select id="ho-to">${targetOpts}</select></div>
        <div class="field"><label>概要（任意）</label><textarea id="ho-summary" rows="2" placeholder="第二の KUSANAGI + 第二の Cursor"></textarea></div>
        <div class="row">
          <button type="button" class="primary" id="ho-export">Markdown を生成</button>
          <button type="button" class="secondary" id="ho-copy">コピー</button>
        </div>
        <pre id="ho-export-out" class="handoff-pre"></pre>
      </section>
      <section class="handoff-card">
        <h3>📥 インポート（他ツール → aruaru）</h3>
        <p class="small-muted">Cursor / Claude 等からコピーした Markdown を貼り付け</p>
        <textarea id="ho-import-in" rows="8" placeholder="---&#10;project: myapp&#10;---"></textarea>
        <button type="button" class="primary" id="ho-import">aruaru に取り込む</button>
        <div id="ho-import-msg"></div>
      </section>
    </div>
    <section class="handoff-card" style="margin-top:1rem">
      <h3>🤖 AI おすすめ（用途別）</h3>
      <div class="field"><label>用途</label><select id="ai-use-case">${useCaseOpts}</select></div>
      <button type="button" class="secondary" id="ai-recommend">おすすめを表示</button>
      <div id="ai-list" class="ai-list"></div>
      <p class="small-muted">詳細版: F:/aruaru/docs/HANDOFF_2026-06-24.md</p>
    </section>
  `;
  return el;
}

async function wireHandoff(panel: HTMLElement) {
  const exportOut = panel.querySelector("#ho-export-out") as HTMLElement;
  const importMsg = panel.querySelector("#ho-import-msg") as HTMLElement;

  panel.querySelector("#ho-export")?.addEventListener("click", async () => {
    const project = (panel.querySelector("#ho-project") as HTMLInputElement).value.trim();
    const to = (panel.querySelector("#ho-to") as HTMLSelectElement).value;
    const summary = (panel.querySelector("#ho-summary") as HTMLTextAreaElement).value.trim();
    exportOut.textContent = "生成中…";
    try {
      const res = await serverApi<{ ok: boolean; markdown?: string; error?: string }>("/handoff/export", {
        method: "POST",
        body: JSON.stringify({ project, to, from: "aruaru-desktop", summary, goals: [] }),
      });
      exportOut.textContent = res.ok && res.markdown ? res.markdown : res.error || "failed";
    } catch (e) {
      exportOut.textContent = String(e);
    }
  });

  panel.querySelector("#ho-copy")?.addEventListener("click", async () => {
    const text = exportOut.textContent || "";
    if (text) await navigator.clipboard.writeText(text);
  });

  panel.querySelector("#ho-import")?.addEventListener("click", async () => {
    const text = (panel.querySelector("#ho-import-in") as HTMLTextAreaElement).value;
    try {
      const res = await serverApi<{ ok: boolean; packet?: { project?: string }; error?: string }>("/handoff/import", {
        method: "POST",
        body: JSON.stringify({ text, source: "aruaru-desktop" }),
      });
      showMsg(importMsg, res.ok ? `取り込み完了: ${res.packet?.project ?? ""}` : res.error || "error", !res.ok);
    } catch (e) {
      showMsg(importMsg, String(e), true);
    }
  });

  panel.querySelector("#ai-recommend")?.addEventListener("click", async () => {
    const useCase = (panel.querySelector("#ai-use-case") as HTMLSelectElement).value;
    const list = panel.querySelector("#ai-list") as HTMLElement;
    list.innerHTML = "<p class=\"text-muted\">読み込み中…</p>";
    try {
      const res = await serverApi<{
        ok: boolean;
        recommended?: { name: string; cost_tier: string; vendor: string; strengths: string[] }[];
      }>(`/ai/recommend?use_case=${encodeURIComponent(useCase)}`);
      if (!res.recommended?.length) {
        list.innerHTML = "<p class=\"text-muted\">該当なし（サーバ未接続の可能性）</p>";
        return;
      }
      list.innerHTML = res.recommended
        .map(
          (p) =>
            `<div class="ai-item"><strong>${p.name}</strong> <span class="badge">${p.cost_tier}</span> <span class="badge">${p.vendor}</span><br/><small>${p.strengths.join(" · ")}</small></div>`
        )
        .join("");
    } catch (e) {
      list.innerHTML = `<p class="msg error">${e}</p>`;
    }
  });
}

function panelPlugins(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>プラグイン管理</h2>
    <p><strong>aruaru-OS-Recovery</strong> は単体でインストール / アンインストールできます。</p>
    <div class="row">
      <button class="primary" id="install-recovery">OS-Recovery をインストール</button>
      <button class="secondary" id="uninstall-recovery">OS-Recovery をアンインストール</button>
      <button class="secondary" id="uninstall-purge">アンインストール（データも削除）</button>
    </div>
    <div id="plugin-msg"></div>
  `;
  return el;
}

async function populateDrives(select: HTMLSelectElement) {
  const drives = await invoke<DriveInfo[]>("list_drives");
  select.innerHTML = drives
    .map((d) => `<option value="${d.letter}">${d.letter} ${d.label} (空き ${d.free_gb} GB)</option>`)
    .join("");
  select.value = config.data_drive || drives[0]?.letter || "F:";
}

async function renderTab(name: string) {
  panels.innerHTML = "";
  let panel: HTMLElement;
  switch (name) {
    case "wizard":
      panel = panelWizard();
      panels.append(panel);
      await wireWizard(panel);
      break;
    case "dashboard":
      panel = panelDashboard();
      panels.append(panel);
      await wireDashboard(panel);
      break;
    case "features":
      panel = panelFeatures();
      panels.append(panel);
      await wireFeatures(panel);
      break;
    case "handoff":
      panel = panelHandoff();
      panels.append(panel);
      await wireHandoff(panel);
      break;
    case "drive":
      panel = panelDrive();
      panels.append(panel);
      await wireDrive(panel);
      break;
    case "recovery":
      panel = panelRecovery();
      panels.append(panel);
      wireRecovery(panel);
      break;
    case "backup":
      panel = panelBackup();
      panels.append(panel);
      wireBackup(panel);
      break;
    case "vps":
      panel = panelVps();
      panels.append(panel);
      wireVps(panel);
      break;
    case "plugins":
      panel = panelPlugins();
      panels.append(panel);
      wirePlugins(panel);
      break;
  }
}

function wizardInputFrom(panel: HTMLElement): VpsWizardInput {
  return {
    provider: (panel.querySelector("#wiz-provider") as HTMLSelectElement).value,
    host: (panel.querySelector("#wiz-host") as HTMLInputElement).value.trim(),
    user: (panel.querySelector("#wiz-user") as HTMLInputElement).value.trim() || "root",
    key_path: (panel.querySelector("#wiz-key") as HTMLInputElement).value.trim(),
    remote_dir: (panel.querySelector("#wiz-remote") as HTMLInputElement).value.trim() || "/root/aruaru",
    save_key_copy: (panel.querySelector("#wiz-save-copy") as HTMLInputElement).checked,
  };
}

function renderWizardChecks(result: WizardTestResult): string {
  return `<ul class="wizard-checks">${result.checks
    .map(
      (c) =>
        `<li class="${c.ok ? "ok" : "ng"}"><span>${c.ok ? "✓" : "✗"}</span> <strong>${c.label}</strong><br/><small>${c.message}</small></li>`
    )
    .join("")}</ul>${result.fix_hint ? `<div class="msg error">${result.fix_hint}</div>` : ""}`;
}

async function wireWizard(panel: HTMLElement) {
  const msg = panel.querySelector("#wiz-msg") as HTMLElement;
  const checks = panel.querySelector("#wiz-checks") as HTMLElement;
  const pubkey = panel.querySelector("#wiz-pubkey") as HTMLElement;
  const keyInput = panel.querySelector("#wiz-key") as HTMLInputElement;

  try {
    const keys = await invoke<{ path: string; name: string }[]>("list_ssh_keys");
    const conoha = keys.find((k) => k.name.includes("conoha"));
    if (conoha && !keyInput.value) keyInput.value = conoha.path;
  } catch { /* ignore */ }

  panel.querySelector("#wiz-pick-key")?.addEventListener("click", async () => {
    const p = await invoke<string | null>("pick_pem_file");
    if (p) keyInput.value = p;
  });

  panel.querySelector("#wiz-gen-key")?.addEventListener("click", async () => {
    const name = prompt("鍵の名前（例: conoha-main）", "aruaru-vps") || "aruaru-vps";
    try {
      const k = await invoke<{ path: string }>("generate_ssh_key", { name });
      keyInput.value = k.path;
      showMsg(msg, `鍵を作成しました: ${k.path}`);
    } catch (e) {
      showMsg(msg, String(e), true);
    }
  });

  const runTest = async () => {
    checks.innerHTML = "<p class=\"text-muted\">確認中…</p>";
    pubkey.textContent = "";
    try {
      const result = await invoke<WizardTestResult>("test_vps_wizard", {
        input: wizardInputFrom(panel),
      });
      checks.innerHTML = renderWizardChecks(result);
      if (result.public_key) {
        pubkey.innerHTML = `<p>公開鍵（ConoHa に登録）:</p><pre class="feature-out">${result.public_key}</pre>`;
      }
      showMsg(msg, result.ok ? "接続成功" : "接続できませんでした", !result.ok);
      if (result.ok) await refreshVpsStatus();
    } catch (e) {
      showMsg(msg, String(e), true);
    }
  };

  panel.querySelector("#wiz-test")?.addEventListener("click", () => void runTest());

  panel.querySelector("#wiz-save")?.addEventListener("click", async () => {
    const input = wizardInputFrom(panel);
    try {
      await invoke("save_vps_wizard", { input, rootDir: config.root_dir });
      if (input.host) {
        config.server_url = `http://${input.host}:3000`;
        await invoke("save_desktop_config", { config });
      }
      await runTest();
      showMsg(msg, "設定を保存しました");
      await refreshVpsStatus();
      await refreshServerStatus();
    } catch (e) {
      showMsg(msg, String(e), true);
    }
  });
}

async function wireDashboard(panel: HTMLElement) {
  const body = panel.querySelector("#dash-body") as HTMLElement;
  const msg = panel.querySelector("#dash-msg") as HTMLElement;

  const load = async () => {
    body.innerHTML = "<p class=\"text-muted\">読み込み中…</p>";
    try {
      const dash = await invoke<ServerDashboard>("fetch_server_dashboard", {
        rootDir: config.root_dir,
      });
      if (!dash.ssh_ok) {
        body.innerHTML = `<div class="msg error">SSH 未接続: ${dash.error || "接続ウィザードで設定してください"}</div>`;
        return;
      }
      if (dash.status) {
        body.innerHTML = renderServerDashboardCard(dash.status);
        if (dash.status.public_url) {
          config.server_url = dash.status.public_url;
        }
      } else {
        body.innerHTML = `<div class="msg error">${dash.error || "サーバー状態を取得できません（aruaru を VPS にデプロイしてください）"}</div>`;
      }
      showMsg(msg, `更新: ${dash.checked_at}`);
    } catch (e) {
      body.innerHTML = "";
      showMsg(msg, String(e), true);
    }
  };

  await load();
  panel.querySelector("#dash-refresh")?.addEventListener("click", () => void load());
}

const catLabels: Record<string, string> = {
  drive: "ドライブ",
  recovery: "OS 復元",
  backup: "バックアップ",
  vps: "VPS",
  plugin: "プラグイン",
  server: "サーバ",
  easyweb: "easyweb",
  domain: "ドメイン",
  https: "HTTPS",
  vhost: "Web サーバー",
  dns: "DNS",
};

async function wireFeatures(panel: HTMLElement) {
  const list = panel.querySelector("#features-list")!;
  const msg = panel.querySelector("#features-msg") as HTMLElement;
  const domainInput = panel.querySelector("#feature-domain") as HTMLInputElement;
  const targetInput = panel.querySelector("#feature-target") as HTMLInputElement;

  let local: DesktopFeatureItem[] = [];
  let remote: DesktopFeatureItem[] = [];
  try {
    local = await invoke<DesktopFeatureItem[]>("list_desktop_features");
    try {
      remote = await invoke<DesktopFeatureItem[]>("fetch_server_features", {
        serverUrl: config.server_url,
      });
    } catch {
      remote = [];
    }
  } catch (e) {
    showMsg(msg, String(e), true);
    return;
  }

  const all = [...local, ...remote];
  const byCat = all.reduce<Record<string, DesktopFeatureItem[]>>((acc, f) => {
    (acc[f.category] ??= []).push(f);
    return acc;
  }, {});

  list.innerHTML = Object.entries(byCat)
    .map(
      ([cat, items]) => `
<section class="feature-cat">
  <h3>${catLabels[cat] ?? cat}</h3>
  <ul class="list">
    ${items
      .map(
        (f) => `
    <li data-id="${f.id}" data-source="${f.source}">
      <strong>${f.name}</strong>
      <span class="badge">${f.platform}</span>
      <span class="badge">${f.source}</span>
      <p><small>${f.description}</small></p>
      ${f.runnable ? `<button class="primary run-feature" data-id="${f.id}" data-source="${f.source}">実行</button>` : ""}
      <pre class="feature-out" hidden></pre>
    </li>`
      )
      .join("")}
  </ul>
</section>`
    )
    .join("");

  list.querySelectorAll(".run-feature").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = (btn as HTMLButtonElement).dataset.id!;
      const source = (btn as HTMLButtonElement).dataset.source!;
      const li = btn.closest("li")!;
      const pre = li.querySelector(".feature-out") as HTMLElement;
      pre.hidden = false;
      pre.textContent = "実行中…";
      try {
        const params: Record<string, string | boolean> = {};
        if (id === "https.certbot") {
          const d = domainInput.value.trim();
          if (!d) throw new Error("ドメインを入力してください");
          params.domain = d;
        }
        if (id === "dns.guide") {
          params.zone = domainInput.value.trim() || "example.com";
        }
        if (id === "backup.run") {
          const t = targetInput.value.trim();
          if (t) params.backup_dir = t;
        }
        if (id === "sync.run") {
          const t = targetInput.value.trim();
          if (!t) throw new Error("同期先を入力してください");
          params.target_dir = t;
        }
        if (id === "plugin.uninstall_recovery") {
          params.purge = false;
        }

        let res: unknown;
        if (source === "server") {
          res = await invoke("run_server_feature", {
            serverUrl: config.server_url,
            id,
            params,
          });
        } else {
          res = await invoke("run_desktop_feature", { id, params });
        }
        pre.textContent = JSON.stringify(res, null, 2);
        showMsg(msg, `${id} 完了`);
        if (id === "server.ping") await refreshServerStatus();
        if (id === "vps.check") await refreshVpsStatus();
      } catch (e) {
        pre.textContent = String(e);
        showMsg(msg, String(e), true);
      }
    });
  });
}

async function wireDrive(panel: HTMLElement) {
  const dataDrive = panel.querySelector<HTMLSelectElement>("#data-drive")!;
  await populateDrives(dataDrive);

  panel.querySelector("#pick-data")?.addEventListener("click", async () => {
    const p = await invoke<string | null>("pick_folder", { title: "DATA フォルダ" });
    if (p) (panel.querySelector("#data-dir") as HTMLInputElement).value = p;
  });
  panel.querySelector("#pick-root")?.addEventListener("click", async () => {
    const p = await invoke<string | null>("pick_folder", { title: "アプリフォルダ" });
    if (p) (panel.querySelector("#root-dir") as HTMLInputElement).value = p;
  });
  panel.querySelector("#save-config")?.addEventListener("click", async () => {
    const msg = panel.querySelector("#drive-msg") as HTMLElement;
    config = {
      os_drive: (panel.querySelector("#os-drive") as HTMLInputElement).value,
      data_drive: dataDrive.value,
      data_dir: (panel.querySelector("#data-dir") as HTMLInputElement).value,
      root_dir: (panel.querySelector("#root-dir") as HTMLInputElement).value,
      server_url: (panel.querySelector("#server-url") as HTMLInputElement).value,
    };
    await invoke("save_desktop_config", { config });
    showMsg(msg, "設定を保存しました");
    await refreshServerStatus();
  });
  panel.querySelector("#bind-drive")?.addEventListener("click", async () => {
    const msg = panel.querySelector("#drive-msg") as HTMLElement;
    const dataDir = (panel.querySelector("#data-dir") as HTMLInputElement).value;
    const rootDir = (panel.querySelector("#root-dir") as HTMLInputElement).value;
    try {
      const fp = await invoke("bind_drive", { dataDir, appDir: rootDir, serverUrl: config.server_url });
      showMsg(msg, `footprint 作成: ${JSON.stringify(fp)}`);
    } catch (e) {
      showMsg(msg, String(e), true);
    }
  });
}

function wireRecovery(panel: HTMLElement) {
  const list = panel.querySelector("#fp-list")!;
  const msg = panel.querySelector("#recovery-msg") as HTMLElement;

  panel.querySelector("#scan-fp")?.addEventListener("click", async () => {
    const items = await invoke<FootprintScan[]>("scan_footprints");
    list.innerHTML = items.length
      ? items
          .map(
            (f) =>
              `<li><strong>${f.data_drive}</strong> ${f.data_dir}<br/><small>${f.marker_path}</small>
               <div class="row" style="margin-top:0.5rem">
                 <button class="primary" data-marker="${f.marker_path}">アクセス復活</button>
               </div></li>`
          )
          .join("")
      : "<li>footprint が見つかりませんでした</li>";

    list.querySelectorAll("button[data-marker]").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const marker = (btn as HTMLButtonElement).dataset.marker!;
        try {
          const path = await invoke<string>("restore_access", {
            markerPath: marker,
            serverUrl: config.server_url,
          });
          showMsg(msg, `復元完了: ${path}`);
          await loadConfig();
        } catch (e) {
          showMsg(msg, String(e), true);
        }
      });
    });
  });
}

function wireBackup(panel: HTMLElement) {
  const msg = panel.querySelector("#backup-msg") as HTMLElement;
  panel.querySelector("#pick-backup")?.addEventListener("click", async () => {
    const p = await invoke<string | null>("pick_folder", { title: "バックアップ先" });
    if (p) (panel.querySelector("#backup-dir") as HTMLInputElement).value = p;
  });
  panel.querySelector("#run-backup")?.addEventListener("click", async () => {
    const backupDir = (panel.querySelector("#backup-dir") as HTMLInputElement).value;
    try {
      const dest = await invoke<string>("run_backup", {
        dataDir: config.data_dir,
        backupDir,
      });
      showMsg(msg, `バックアップ完了: ${dest}`);
    } catch (e) {
      showMsg(msg, String(e), true);
    }
  });
  panel.querySelector("#run-sync")?.addEventListener("click", async () => {
    const target = (panel.querySelector("#sync-target") as HTMLInputElement).value;
    if (!target) {
      showMsg(msg, "同期先を入力してください", true);
      return;
    }
    try {
      const dest = await invoke<string>("run_sync", {
        dataDir: config.data_dir,
        targetDir: target,
      });
      showMsg(msg, `同期完了: ${dest}`);
    } catch (e) {
      showMsg(msg, String(e), true);
    }
  });
}

function wireVps(panel: HTMLElement) {
  const msg = panel.querySelector("#vps-msg") as HTMLElement;
  const log = panel.querySelector("#vps-log") as HTMLElement;
  const statusPanel = panel.querySelector("#vps-status-panel") as HTMLElement;

  const refreshPanel = async () => {
    statusPanel.innerHTML = `<p class="text-muted">接続確認中…</p>`;
    try {
      const st = await invoke<VpsConnectionStatus>("check_vps_connection", {
        rootDir: config.root_dir,
      });
      statusPanel.innerHTML = renderVpsStatusCard(st);
      await refreshVpsStatus();
    } catch (e) {
      statusPanel.innerHTML = `<div class="msg error">${String(e)}</div>`;
    }
  };

  void refreshPanel();
  panel.querySelector("#check-vps")?.addEventListener("click", () => void refreshPanel());

  const run = async (skipSetup: boolean) => {
    log.textContent = "実行中...（数分かかります）";
    showMsg(msg, "");
    try {
      const out = await invoke<string>("run_upload_vps", {
        rootDir: config.root_dir,
        skipSetup,
      });
      log.textContent = out || "完了";
      showMsg(msg, skipSetup ? "アップロード完了" : "アップロード + VPS セットアップ完了");
      await refreshPanel();
    } catch (e) {
      log.textContent = String(e);
      showMsg(msg, String(e), true);
    }
  };

  panel.querySelector("#upload-vps")?.addEventListener("click", () => run(false));
  panel.querySelector("#upload-only")?.addEventListener("click", () => run(true));
}

function wirePlugins(panel: HTMLElement) {
  const msg = panel.querySelector("#plugin-msg") as HTMLElement;
  const install = async (purge = false) => {
    try {
      if (purge) {
        await invoke("uninstall_plugin", {
          serverUrl: config.server_url,
          id: "aruaru-os-recovery",
          purge: true,
        });
        showMsg(msg, "OS-Recovery をアンインストールしました（データ削除）");
      } else {
        await invoke("uninstall_plugin", {
          serverUrl: config.server_url,
          id: "aruaru-os-recovery",
          purge: false,
        });
        showMsg(msg, "OS-Recovery をアンインストールしました");
      }
    } catch (e) {
      showMsg(msg, String(e), true);
    }
  };
  panel.querySelector("#install-recovery")?.addEventListener("click", async () => {
    try {
      await invoke("install_plugin", {
        serverUrl: config.server_url,
        id: "aruaru-os-recovery",
      });
      showMsg(msg, "OS-Recovery をインストールしました");
    } catch (e) {
      showMsg(msg, String(e), true);
    }
  });
  panel.querySelector("#uninstall-recovery")?.addEventListener("click", () => install(false));
  panel.querySelector("#uninstall-purge")?.addEventListener("click", () => install(true));
}

window.addEventListener("DOMContentLoaded", async () => {
  document.querySelectorAll("#tabs button").forEach((btn) => {
    btn.addEventListener("click", () => {
      document.querySelectorAll("#tabs button").forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      renderTab((btn as HTMLButtonElement).dataset.tab!);
    });
  });
  await loadConfig();
  renderTab("wizard");
});
