import { invoke } from "@tauri-apps/api/core";

interface DnsRecord {
  id: number;
  zone: string;
  name: string;
  record_type: string;
  value: string;
  ttl: number;
  created_at: string;
}

interface DbStatus {
  postgres: boolean;
  aruaru_db: boolean;
  aruaru_db_path: string;
  postgres_url: string;
  aruaru_db_url: string;
}

interface DnsGuide {
  zone: string;
  vps_ip: string;
  nameservers: string[];
  steps: string[];
  suggested_records: {
    record_type: string;
    name: string;
    value: string;
    ttl: number;
    note: string;
  }[];
}

async function loadStatus() {
  const s = await invoke<DbStatus>("db_status");
  const el = document.getElementById("db-status");
  if (!el) return;
  const pg = s.postgres ? "PG:OK" : "PG:NG";
  const adb = s.aruaru_db ? "aruaru-db:OK" : "aruaru-db:off";
  el.textContent = `${pg} · ${adb}`;
  el.className = `badge ${s.postgres ? "text-bg-success" : "text-bg-danger"}`;
}

async function loadRecords() {
  const rows = await invoke<DnsRecord[]>("list_dns_records", { zone: null });
  const body = document.getElementById("records-body");
  if (!body) return;
  if (!rows.length) {
    body.innerHTML = `<tr><td colspan="6" class="text-muted">レコードなし</td></tr>`;
    return;
  }
  body.innerHTML = rows
    .map(
      (r) => `<tr>
        <td>${esc(r.zone)}</td>
        <td>${esc(r.name)}</td>
        <td>${esc(r.record_type)}</td>
        <td class="font-monospace">${esc(r.value)}</td>
        <td>${r.ttl}</td>
        <td><button class="btn btn-sm btn-outline-danger del" data-id="${r.id}">削除</button></td>
      </tr>`
    )
    .join("");
  body.querySelectorAll(".del").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = Number((btn as HTMLButtonElement).dataset.id);
      await invoke("delete_dns_record", { id });
      await loadRecords();
    });
  });
}

function esc(s: string) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

document.getElementById("guide-form")?.addEventListener("submit", async (e) => {
  e.preventDefault();
  const fd = new FormData(e.target as HTMLFormElement);
  const guide = await invoke<DnsGuide>("dns_guide", {
    zone: fd.get("zone"),
    vpsIp: fd.get("vps_ip"),
  });
  const out = document.getElementById("guide-result");
  if (!out) return;
  const steps = guide.steps.map((s) => `<li>${esc(s)}</li>`).join("");
  const ns = guide.nameservers.map((n) => `<li><code>${esc(n)}</code></li>`).join("");
  const sug = guide.suggested_records
    .map(
      (r) =>
        `<div class="suggested mb-1">${esc(r.record_type)} ${esc(r.name)} → ${esc(r.value)} <span class="text-muted">(${esc(r.note)})</span></div>`
    )
    .join("");
  out.innerHTML = `
    <h3 class="h6">手順</h3><ol>${steps}</ol>
    <h3 class="h6 mt-3">ネームサーバー例</h3><ul>${ns}</ul>
    <h3 class="h6 mt-3">推奨レコード</h3>${sug}`;
});

document.getElementById("record-form")?.addEventListener("submit", async (e) => {
  e.preventDefault();
  const fd = new FormData(e.target as HTMLFormElement);
  await invoke("add_dns_record", {
    zone: fd.get("zone"),
    name: fd.get("name") || "@",
    recordType: fd.get("record_type"),
    value: fd.get("value"),
    ttl: Number(fd.get("ttl") || 3600),
  });
  (e.target as HTMLFormElement).reset();
  await loadRecords();
});

document.getElementById("sync-btn")?.addEventListener("click", async () => {
  const n = await invoke<number>("sync_to_aruaru_db");
  alert(`aruaru-db に ${n} 件同期しました`);
  await loadStatus();
});

window.addEventListener("DOMContentLoaded", async () => {
  await loadStatus();
  await loadRecords();
});
