import { fetchRecommend, getSelectedAi, renderAiCard, wireAiSelect } from "./ai";

const SERVER_KEY = "aruaru_server";

function getServer(): string {
  return (localStorage.getItem(SERVER_KEY) || "http://127.0.0.1:3000").replace(/\/$/, "");
}

function setServer(url: string): void {
  localStorage.setItem(SERVER_KEY, url.replace(/\/$/, ""));
}

async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${getServer()}/api${path}`, {
    headers: { "Content-Type": "application/json", Accept: "application/json" },
    ...init,
  });
  return res.json() as Promise<T>;
}

type ServerStatus = {
  hostname?: string;
  cpu_percent?: number | null;
  memory_percent?: number | null;
  disk_percent?: number | null;
  ssl?: { ok: boolean; label: string };
  domain?: { ok: boolean; primary?: string };
  public_url?: string;
};

const HANDOFF_TARGETS = [
  ["cursor", "Cursor"],
  ["claude", "Claude"],
  ["gemini", "Gemini"],
  ["deepseek", "DeepSeek"],
  ["aruaru", "aruaru"],
] as const;

const panels = document.querySelector("#panels")!;
const serverStatus = document.querySelector("#server-status")!;

function pctBar(label: string, value: number | null | undefined): string {
  const v = value ?? null;
  const display = v != null ? `${Math.round(v)}%` : "—";
  const width = v != null ? Math.min(100, Math.max(0, v)) : 0;
  return `<div class="metric"><div class="metric-head"><span>${label}</span><strong>${display}</strong></div><div class="bar"><div class="bar-fill" style="width:${width}%"></div></div></div>`;
}

async function refreshStatus(): Promise<void> {
  try {
    const h = await api<{ ok: boolean; public_url?: string }>("/health");
    serverStatus.textContent = h.ok ? `OK ${h.public_url || getServer()}` : "NG";
  } catch {
    serverStatus.textContent = `未接続 (${getServer()})`;
  }
}

function panelDevelop(): HTMLElement {
  const sel = getSelectedAi("development");
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>第二の Cursor — アプリ/Web 開発</h2>
    <p class="muted">Opus / GPT / DeepSeek 等 — 低単価で高性能な AI を選択</p>
    <p class="selected-ai">選択中: <strong id="dev-sel">${sel ?? "未選択"}</strong></p>
    <button class="primary" id="dev-load">おすすめ AI を表示</button>
    <div id="dev-list"></div>
  `;
  el.querySelector("#dev-load")?.addEventListener("click", async () => {
    const list = el.querySelector("#dev-list")!;
    list.innerHTML = "<p class='muted'>読み込み…</p>";
    try {
      const rec = await fetchRecommend(api, "development");
      const cur = getSelectedAi("development");
      list.innerHTML = rec.map((p) => renderAiCard(p, p.id === cur, "development")).join("") || "<p class='muted'>該当なし</p>";
      wireAiSelect(list, "development");
      list.querySelectorAll(".select-ai").forEach((b) => {
        b.addEventListener("click", () => {
          const id = (b as HTMLButtonElement).dataset.id!;
          (el.querySelector("#dev-sel") as HTMLElement).textContent = id;
        });
      });
    } catch (e) {
      list.innerHTML = `<p class="error">${e}</p>`;
    }
  });
  return el;
}

function panelEnglish(): HTMLElement {
  const sel = getSelectedAi("english_conversation");
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>英会話モード</h2>
    <p class="muted">音声・翻訳が高性能で単価の安い AI をおすすめ</p>
    <p class="selected-ai">選択中: <strong id="eng-sel">${sel ?? "未選択"}</strong></p>
    <button class="primary" id="eng-mic">🎤 音声入力（マイク）</button>
    <div id="eng-transcript" class="code-block"></div>
    <button class="secondary" id="eng-load">おすすめ AI を表示</button>
    <div id="eng-list"></div>
  `;
  const transcript = el.querySelector("#eng-transcript") as HTMLElement;
  el.querySelector("#eng-mic")?.addEventListener("click", () => {
    const SR = (window as unknown as { webkitSpeechRecognition?: typeof SpeechRecognition }).webkitSpeechRecognition || window.SpeechRecognition;
    if (!SR) {
      transcript.textContent = "この端末は音声認識未対応です";
      return;
    }
    const rec = new SR();
    rec.lang = "en-US";
    rec.interimResults = true;
    transcript.textContent = "聞き取り中…";
    rec.onresult = (ev: SpeechRecognitionEvent) => {
      transcript.textContent = Array.from(ev.results).map((r) => r[0].transcript).join("");
    };
    rec.onerror = () => { transcript.textContent = "音声エラー（マイク権限を確認）"; };
    rec.start();
  });
  el.querySelector("#eng-load")?.addEventListener("click", async () => {
    const list = el.querySelector("#eng-list")!;
    list.innerHTML = "<p class='muted'>読み込み…</p>";
    try {
      const rec = await fetchRecommend(api, "english_conversation");
      const cur = getSelectedAi("english_conversation");
      list.innerHTML = rec.map((p) => renderAiCard(p, p.id === cur, "english_conversation")).join("") || "<p class='muted'>該当なし</p>";
      wireAiSelect(list, "english_conversation");
      list.querySelectorAll(".select-ai").forEach((b) => {
        b.addEventListener("click", () => {
          (el.querySelector("#eng-sel") as HTMLElement).textContent = (b as HTMLButtonElement).dataset.id!;
        });
      });
    } catch (e) {
      list.innerHTML = `<p class="error">${e}</p>`;
    }
  });
  return el;
}

function panelDashboard(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `<h2>サーバー状態</h2><div id="dash"><p class="muted">読み込み…</p></div><button class="primary" id="dash-refresh">再読み込み</button>`;
  const load = async () => {
    const box = el.querySelector("#dash")!;
    try {
      const st = await api<ServerStatus>("/server/status");
      box.innerHTML = `${pctBar("CPU", st.cpu_percent)}${pctBar("メモリ", st.memory_percent)}${pctBar("ディスク", st.disk_percent)}<p>${st.ssl?.ok ? "🟢" : "🟡"} SSL: ${st.ssl?.label || "—"}</p>`;
    } catch (e) {
      box.innerHTML = `<p class="error">${e}</p>`;
    }
  };
  el.querySelector("#dash-refresh")?.addEventListener("click", () => void load());
  void load();
  return el;
}

function panelHandoff(): HTMLElement {
  const opts = HANDOFF_TARGETS.map(([id, label]) => `<option value="${id}">${label}</option>`).join("");
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>引き継ぎ</h2>
    <div class="field"><label>プロジェクト</label><input id="p" value="aruaru" /></div>
    <div class="field"><label>先</label><select id="to">${opts}</select></div>
    <button class="primary" id="exp">Markdown 生成</button>
    <button class="secondary" id="copy">コピー</button>
    <pre id="out" class="code-block"></pre>
    <textarea id="import-in" rows="4" placeholder="インポート Markdown"></textarea>
    <button class="primary" id="import">取り込む</button>
    <div id="import-msg"></div>
  `;
  const out = el.querySelector("#out") as HTMLElement;
  el.querySelector("#exp")?.addEventListener("click", async () => {
    const res = await api<{ ok: boolean; markdown?: string }>("/handoff/export", {
      method: "POST",
      body: JSON.stringify({
        project: (el.querySelector("#p") as HTMLInputElement).value,
        to: (el.querySelector("#to") as HTMLSelectElement).value,
        from: "aruaru-android",
        summary: "第二の Cursor",
        goals: [],
      }),
    });
    out.textContent = res.markdown || "error";
  });
  el.querySelector("#copy")?.addEventListener("click", () => navigator.clipboard.writeText(out.textContent || ""));
  el.querySelector("#import")?.addEventListener("click", async () => {
    const text = (el.querySelector("#import-in") as HTMLTextAreaElement).value;
    const msg = el.querySelector("#import-msg")!;
    const res = await api<{ ok: boolean; packet?: { project?: string } }>("/handoff/import", {
      method: "POST",
      body: JSON.stringify({ text, source: "aruaru-android" }),
    });
    msg.textContent = res.ok ? `OK: ${res.packet?.project}` : "error";
  });
  return el;
}

function panelSettings(): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `
    <h2>設定</h2>
    <div class="field"><label>サーバー URL</label><input id="server-url" value="${getServer()}" /></div>
    <button class="primary" id="save-server">保存</button>
    <div id="settings-msg"></div>
    <p class="muted small">例: http://160.251.237.162:3000</p>
  `;
  el.querySelector("#save-server")?.addEventListener("click", async () => {
    setServer((el.querySelector("#server-url") as HTMLInputElement).value.trim());
    await refreshStatus();
    (el.querySelector("#settings-msg") as HTMLElement).textContent = "保存しました";
  });
  return el;
}

function renderTab(name: string): void {
  panels.innerHTML = "";
  const map: Record<string, () => HTMLElement> = {
    develop: panelDevelop,
    english: panelEnglish,
    handoff: panelHandoff,
    dashboard: panelDashboard,
    settings: panelSettings,
  };
  panels.append(map[name]?.() ?? panelDevelop());
}

document.querySelectorAll("#tabs button").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll("#tabs button").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
    renderTab((btn as HTMLButtonElement).dataset.tab!);
  });
});

void refreshStatus();
renderTab("develop");
