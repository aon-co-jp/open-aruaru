/** VersionLess API client — Poem /api/* typed wrapper (WunderGraph client と置換予定) */

export type FeatureItem = {
  id: string;
  name: string;
  description: string;
  category: string;
  platform: string;
  runnable: boolean;
};

export type ServerStatus = {
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

export type HttpsStatus = {
  enabled: boolean;
  auto_configure: boolean;
  certbot_email_set: boolean;
  web_server: string;
  acme_webroot: string;
  renewal_interval_secs: number;
  scan_interval_secs: number;
  queue_count: number;
  pending_queue: { domain?: string; note?: string; queued_at?: string }[];
};

export type ApiResult<T> = T & { ok: boolean; error?: string };

export type Health = {
  ok: boolean;
  version: string;
  easyweb: string;
  registered: boolean;
  public_url: string;
  primary_domain?: string;
  database: { primary: string; backend: string };
};

export type PlatformDomain = {
  name: string;
  is_primary: number;
  https_enabled: number;
  last_checked?: string;
};

export type Plugin = { id: string; version: string };

export type Domain = {
  name: string;
  https_enabled: number;
  last_checked?: string;
};

async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch('/api' + path, {
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    credentials: 'same-origin',
    ...init,
  });
  return res.json() as Promise<T>;
}

export const aruaruApi = {
  health: () => api<Health>('/health'),
  serverStatus: () => api<ServerStatus>('/server/status'),
  httpsStatus: () => api<ApiResult<{ https: HttpsStatus }>>('/https/status'),
  register: (email: string) =>
    api<ApiResult<{ magic_link?: string; message?: string }>>('/register', {
      method: 'POST',
      body: JSON.stringify({ email }),
    }),
  login: (email: string) =>
    api<ApiResult<{ magic_link?: string }>>('/login', {
      method: 'POST',
      body: JSON.stringify({ email }),
    }),
  platformDomains: () =>
    api<ApiResult<{ domains: PlatformDomain[] }>>('/platform/domains'),
  addPlatformDomain: (name: string) =>
    api<ApiResult<Record<string, never>>>('/platform/domains', {
      method: 'POST',
      body: JSON.stringify({ name }),
    }),
  deletePlatformDomain: (name: string) =>
    api<ApiResult<Record<string, never>>>('/platform/domains/delete', {
      method: 'POST',
      body: JSON.stringify({ name }),
    }),
  setPrimaryDomain: (name: string) =>
    api<ApiResult<Record<string, never>>>('/platform/domains/primary', {
      method: 'POST',
      body: JSON.stringify({ name }),
    }),
  swapDomains: (from: string, to: string) =>
    api<ApiResult<Record<string, never>>>('/platform/domains/swap', {
      method: 'POST',
      body: JSON.stringify({ from, to }),
    }),
  domains: () => api<ApiResult<{ domains: Domain[] }>>('/domains'),
  addDomain: (name: string) =>
    api<ApiResult<Record<string, never>>>('/domains', {
      method: 'POST',
      body: JSON.stringify({ name }),
    }),
  plugins: () => api<ApiResult<{ plugins: Plugin[] }>>('/plugins'),
  reminders: () =>
    api<{ due: boolean; items: { question: string }[] }>('/reminders'),
  features: () => api<ApiResult<{ features: FeatureItem[] }>>('/features'),
  runFeature: (id: string, params: Record<string, unknown> = {}) =>
    api<ApiResult<{ id: string; result: unknown }>>('/features', {
      method: 'POST',
      body: JSON.stringify({ id, params }),
    }),
  handoffTargets: () => api<ApiResult<{ data: unknown }>>('/handoff/targets'),
  handoffExport: (body: {
    project: string;
    to: string;
    from?: string;
    summary?: string;
    goals?: string[];
  }) =>
    api<ApiResult<{ packet: unknown; markdown: string }>>('/handoff/export', {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  handoffImport: (body: { text: string; source?: string }) =>
    api<ApiResult<{ packet: { project?: string } }>>('/handoff/import', {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  aiProviders: () =>
    api<ApiResult<{ providers: AiProvider[]; stack: unknown }>>('/ai/providers'),
  aiRecommend: (useCase: string) =>
    api<ApiResult<{ recommended: AiProvider[]; use_case: string }>>(
      `/ai/recommend?use_case=${encodeURIComponent(useCase)}`
    ),
};

export type AiProvider = {
  id: string;
  name: string;
  vendor: string;
  kind: string;
  cost_tier: string;
  strengths: string[];
  speech_quality?: string;
  translation_quality?: string;
  score?: number;
  reason?: string;
};
