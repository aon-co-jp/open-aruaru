import React, { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import "bootstrap/dist/css/bootstrap.min.css";
import { gql } from "./versionlessGraphql";

const CLIENT_LABEL = "aruaru-web";
const CLIENT_KIND = "web";

type Status = { version: string; transport: string; workspace: string; readmeFolder: string; projectBundleFolder: string; noRest: boolean; noTauri: boolean };
type Report = { markdown: string; filePath: string; generatedAt: string };
type Project = { name: string; githubUrl?: string; localDrivePath?: string; vpsDirectory?: string; enabled: boolean };
type OfflineCacheEntry = { label: string; sourceUrl: string; sourceHost: string; localPath: string; cachedAt: string; keepForever: boolean; bytes: number };


function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);
  const [report, setReport] = useState<Report | null>(null);
  const [offlineCache, setOfflineCache] = useState<OfflineCacheEntry[]>([]);
  const [message, setMessage] = useState("起動中…");
  const [domain, setDomain] = useState("aruaru.tokyo");
  const [vpsIp, setVpsIp] = useState("127.0.0.1");
  const [projectName, setProjectName] = useState("aruaru");
  const [githubUrl, setGithubUrl] = useState("https://github.com/aon-co-jp/aruaru-db");
  const [localPath, setLocalPath] = useState("");
  const [vpsDir, setVpsDir] = useState("/var/www/aruaru");
  const [offlineUrl, setOfflineUrl] = useState("https://news.yahoo.co.jp/");

  const query = useMemo(() => `query {
    platformStatus { version transport workspace readmeFolder projectBundleFolder noRest noTauri }
    projects { name githubUrl localDrivePath vpsDirectory enabled }
    latestReadmeReport { markdown filePath generatedAt }
    offlineCache(limit: 20) { label sourceUrl sourceHost localPath cachedAt keepForever bytes }
  }`, []);

  async function refresh() {
    const data = await gql(query);
    setStatus(data.platformStatus);
    setProjects(data.projects ?? []);
    setReport(data.latestReadmeReport ?? null);
    setOfflineCache(data.offlineCache ?? []);
    setMessage("同期済み");
  }

  useEffect(() => { refresh().catch(err => setMessage(String(err))); }, []);

  async function linkDevice() {
    await gql(`mutation($input:RegisterDeviceInput!){ registerDevice(input:$input){ id name kind trusted } }`, { input: { name: CLIENT_LABEL, kind: CLIENT_KIND } });
    await refresh();
  }

  async function saveDomain() {
    await gql(`mutation($input:RegisterDomainInput!){ registerDomain(input:$input){ domain vpsIp publicRoot primary } }`, { input: { domain, vpsIp, publicRoot: "/var/www/aruaru/public", primary: true } });
    await refresh();
  }

  async function saveProject() {
    await gql(`mutation($input:RegisterProjectInput!){ registerProjectSource(input:$input){ name githubUrl localDrivePath vpsDirectory } }`, { input: { name: projectName, githubUrl: githubUrl || null, localDrivePath: localPath || null, vpsDirectory: vpsDir || null } });
    await refresh();
  }

  async function addOfflineSource() {
    await gql(`mutation($input:RegisterOfflineSourceInput!){ registerOfflineSource(input:$input){ label sourceUrl category } }`, { input: { label: "offline-pickup", sourceUrl: offlineUrl, category: "news-video-random" } });
    await refresh();
  }

  async function runCycle() {
    setMessage("5分クロールとREADME生成を手動実行中…");
    await gql(`mutation { runFullUpdateCycle { filePath generatedAt } }`);
    await refresh();
  }

  async function runOfflinePrefetch() {
    setMessage("オンラインDATAを取得し、3日超の通常キャッシュを整理中…");
    await gql(`mutation { runOfflinePrefetch }`);
    await refresh();
  }

  async function purgeCache() {
    setMessage("3日超の通常キャッシュを削除中…audiocafe.tokyo DATAは保持します");
    await gql(`mutation { purgeOfflineCache }`);
    await refresh();
  }

  return <main className="container py-4">
    <header className="mb-4">
      <h1>{CLIENT_LABEL}</h1>
      <p className="lead">Rust + Poem + GraphQL Federation / Cosmo 中核へ接続する薄いUIです。</p>
      <div className="alert alert-secondary">{message}</div>
      {status && <div className="small text-muted">version {status.version} / {status.transport} / workspace: {status.workspace}</div>}
    </header>

    <section className="card mb-3"><div className="card-body">
      <h2 className="h5">端末連携</h2>
      <button className="btn btn-primary" onClick={() => linkDevice().catch(e => setMessage(String(e)))}>この端末を登録</button>
    </div></section>

    <section className="card mb-3"><div className="card-body">
      <h2 className="h5">ドメイン / VPS 登録</h2>
      <div className="row g-2">
        <div className="col-md-5"><input className="form-control" value={domain} onChange={e=>setDomain(e.target.value)} placeholder="domain/subdomain" /></div>
        <div className="col-md-4"><input className="form-control" value={vpsIp} onChange={e=>setVpsIp(e.target.value)} placeholder="VPS IP" /></div>
        <div className="col-md-3"><button className="btn btn-success w-100" onClick={() => saveDomain().catch(e=>setMessage(String(e)))}>登録</button></div>
      </div>
    </div></section>

    <section className="card mb-3"><div className="card-body">
      <h2 className="h5">aruaru 関連プロジェクト登録</h2>
      <input className="form-control mb-2" value={projectName} onChange={e=>setProjectName(e.target.value)} placeholder="project name" />
      <input className="form-control mb-2" value={githubUrl} onChange={e=>setGithubUrl(e.target.value)} placeholder="GitHub URL" />
      <input className="form-control mb-2" value={localPath} onChange={e=>setLocalPath(e.target.value)} placeholder="許可するローカルドライブ/フォルダ" />
      <input className="form-control mb-2" value={vpsDir} onChange={e=>setVpsDir(e.target.value)} placeholder="VPS directory" />
      <button className="btn btn-success" onClick={() => saveProject().catch(e=>setMessage(String(e)))}>保存</button>
      <ul className="mt-3">{projects.map((p, i)=><li key={i}><b>{p.name}</b> / {p.githubUrl || "no github"} / {p.localDrivePath || "no local"}</li>)}</ul>
    </div></section>

    <section className="card mb-3"><div className="card-body">
      <h2 className="h5">オフライン閲覧キャッシュ候補</h2>
      <div className="input-group mb-2">
        <input className="form-control" value={offlineUrl} onChange={e=>setOfflineUrl(e.target.value)} />
        <button className="btn btn-outline-primary" onClick={() => addOfflineSource().catch(e=>setMessage(String(e)))}>候補追加</button>
      </div>
      <div className="d-flex gap-2 flex-wrap">
        <button className="btn btn-primary btn-sm" onClick={() => runOfflinePrefetch().catch(e=>setMessage(String(e)))}>今すぐオンラインDATA取得</button>
        <button className="btn btn-outline-danger btn-sm" onClick={() => purgeCache().catch(e=>setMessage(String(e)))}>3日超キャッシュ整理</button>
      </div>
      <p className="small text-muted mt-2">通常ニュース/Youtube/X系のキャッシュは再取得時に3日超を削除。audiocafe.tokyo由来DATAは保持。</p>
      <ul className="small mt-2">{offlineCache.map((c, i)=><li key={i}>{c.label} / {c.sourceHost} / {c.keepForever ? "保持" : "3日保持"} / {c.localPath}</li>)}</ul>
    </div></section>

    <section className="card mb-3"><div className="card-body">
      <h2 className="h5">24時間・5分単位クロール / GitHub README生成</h2>
      <button className="btn btn-warning" onClick={() => runCycle().catch(e=>setMessage(String(e)))}>今すぐUPDATE</button>
      <div className="mt-3 small">出力先: {report?.filePath ?? status?.readmeFolder + "/README.md"}</div>
    </div></section>

    <section className="card"><div className="card-body">
      <h2 className="h5">aruaru（README）/README.md プレビュー</h2>
      <pre className="small bg-light p-3" style={{whiteSpace:"pre-wrap"}}>{report?.markdown ?? "まだ生成されていません"}</pre>
    </div></section>
  </main>;
}

createRoot(document.getElementById("root")!).render(<App />);
