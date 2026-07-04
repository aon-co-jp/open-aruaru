use aruaru_ai::AiRouter;
use aruaru_core::{
    default_project_bundle_folder, default_readme_folder, default_state_file, ensure_workspace_dirs,
    load_state, save_state, sanitize_folder_name, validate_domain, validate_github_url,
    validate_ip_or_host, ActivitySnapshot, ClientKind, DeviceProfile, DomainProfile, DriveFileEntry,
    OfflineSource, OfflineCacheEntry, PlatformState, ProjectSource, ReadmeReport, ServerProfile,
};
use aruaru_crawler::CrawlPlanner;
use aruaru_drive_indexer::{scan_allowlisted_path, DriveScanPolicy};
use aruaru_llm::LlmProfile;
use aruaru_offline_cache::OfflineCacheManager;
use aruaru_versionless::{FunctionInvocation, FunctionRegistry, VersionlessFunction};
use async_graphql::{Context, EmptySubscription, InputObject, Object, Schema, SimpleObject};
use async_graphql_poem::{GraphQLRequest, GraphQLResponse};
use chrono::Utc;
use poem::{listener::TcpListener, web::Data, EndpointExt, Route, Server};
use serde_json::json;
use std::{path::PathBuf, sync::{Arc, Mutex}};
use tokio::time::{interval, Duration};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    state: Arc<Mutex<PlatformState>>,
    state_file: PathBuf,
    functions: FunctionRegistry,
}

impl AppState {
    fn new() -> anyhow::Result<Self> {
        ensure_workspace_dirs()?;
        let state_file = default_state_file();
        let state = load_state(&state_file)?;
        Ok(Self { state: Arc::new(Mutex::new(state)), state_file, functions: FunctionRegistry::with_builtin() })
    }
    fn persist(&self) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.updated_at = Utc::now();
        save_state(&self.state_file, &state)
    }
    fn run_cycle(&self) -> anyhow::Result<ReadmeReport> {
        let planner = CrawlPlanner::new();
        let cache_manager = OfflineCacheManager::default();
        let (projects, domains, devices, offline_sources) = {
            let state = self.state.lock().unwrap();
            (
                state.projects.clone(),
                state.domains.clone(),
                state.devices.clone(),
                state.offline_sources.clone(),
            )
        };

        let mut new_snapshots = Vec::<ActivitySnapshot>::new();
        for project in projects.iter().filter(|p| p.enabled) {
            new_snapshots.push(planner.collect_snapshot(project));
        }
        let new_cache_entries = cache_manager.prefetch_enabled_sources(&offline_sources);

        let report = {
            let mut state = self.state.lock().unwrap();
            state.snapshots.extend(new_snapshots);
            for source in &mut state.offline_sources {
                if new_cache_entries.iter().any(|entry| entry.source_id == source.id) {
                    source.last_cached_at = Some(Utc::now());
                }
            }
            state.offline_cache.extend(new_cache_entries);
            let _removed = cache_manager.purge_expired_entries(&mut state.offline_cache)?;

            let ai = AiRouter::new(LlmProfile::gt730_safe_default());
            let report = ai.generate_github_readme_report(&state.snapshots, &projects, &domains, &devices);
            ai.write_report_file(&report)?;
            state.reports.push(report.clone());
            state.updated_at = Utc::now();
            save_state(&self.state_file, &state)?;
            report
        };
        Ok(report)
    }

    fn run_offline_prefetch(&self) -> anyhow::Result<(usize, usize)> {
        let cache_manager = OfflineCacheManager::default();
        let sources = { self.state.lock().unwrap().offline_sources.clone() };
        let entries = cache_manager.prefetch_enabled_sources(&sources);
        let added = entries.len();
        let removed = {
            let mut state = self.state.lock().unwrap();
            for source in &mut state.offline_sources {
                if entries.iter().any(|entry| entry.source_id == source.id) {
                    source.last_cached_at = Some(Utc::now());
                }
            }
            state.offline_cache.extend(entries);
            let removed = cache_manager.purge_expired_entries(&mut state.offline_cache)?;
            state.updated_at = Utc::now();
            save_state(&self.state_file, &state)?;
            removed
        };
        Ok((added, removed))
    }

    fn purge_offline_cache(&self) -> anyhow::Result<usize> {
        let cache_manager = OfflineCacheManager::default();
        let mut state = self.state.lock().unwrap();
        let removed = cache_manager.purge_expired_entries(&mut state.offline_cache)?;
        state.updated_at = Utc::now();
        save_state(&self.state_file, &state)?;
        Ok(removed)
    }
}

#[derive(SimpleObject, Clone)]
struct StatusGql { version: String, transport: String, no_rest: bool, no_tauri: bool, workspace: String, readme_folder: String, project_bundle_folder: String }

#[derive(SimpleObject, Clone)]
struct DeviceGql { id: String, name: String, kind: String, trusted: bool, last_seen_at: Option<String> }
impl From<DeviceProfile> for DeviceGql { fn from(d: DeviceProfile) -> Self { Self { id: d.id.to_string(), name: d.name, kind: format!("{:?}", d.kind), trusted: d.trusted, last_seen_at: d.last_seen_at.map(|v| v.to_rfc3339()) } } }

#[derive(SimpleObject, Clone)]
struct DomainGql { id: String, domain: String, primary: bool, public_root: String, vps_ip: String, verified: bool, registered_at: String }
impl From<DomainProfile> for DomainGql { fn from(d: DomainProfile) -> Self { Self { id: d.id.to_string(), domain: d.domain, primary: d.primary, public_root: d.public_root, vps_ip: d.vps_ip, verified: d.verified, registered_at: d.registered_at.to_rfc3339() } } }

#[derive(SimpleObject, Clone)]
struct ServerGql { id: String, name: String, vps_ip: String, upload_directory: String, public_root: String, ssh_or_ftp_hint: String, created_at: String }
impl From<ServerProfile> for ServerGql { fn from(s: ServerProfile) -> Self { Self { id: s.id.to_string(), name: s.name, vps_ip: s.vps_ip, upload_directory: s.upload_directory, public_root: s.public_root, ssh_or_ftp_hint: s.ssh_or_ftp_hint, created_at: s.created_at.to_rfc3339() } } }

#[derive(SimpleObject, Clone)]
struct ProjectGql { id: String, name: String, github_url: Option<String>, local_drive_path: Option<String>, vps_directory: Option<String>, enabled: bool, registered_at: String }
impl From<ProjectSource> for ProjectGql { fn from(p: ProjectSource) -> Self { Self { id: p.id.to_string(), name: p.name, github_url: p.github_url, local_drive_path: p.local_drive_path, vps_directory: p.vps_directory, enabled: p.enabled, registered_at: p.registered_at.to_rfc3339() } } }

#[derive(SimpleObject, Clone)]
struct OfflineSourceGql { id: String, label: String, source_url: String, category: String, enabled: bool, last_cached_at: Option<String> }
impl From<OfflineSource> for OfflineSourceGql { fn from(o: OfflineSource) -> Self { Self { id: o.id.to_string(), label: o.label, source_url: o.source_url, category: o.category, enabled: o.enabled, last_cached_at: o.last_cached_at.map(|v| v.to_rfc3339()) } } }

#[derive(SimpleObject, Clone)]
struct OfflineCacheEntryGql { id: String, source_id: String, label: String, source_url: String, source_host: String, local_path: String, content_type: String, cached_at: String, keep_forever: bool, bytes: i64 }
impl From<OfflineCacheEntry> for OfflineCacheEntryGql { fn from(o: OfflineCacheEntry) -> Self { Self { id: o.id.to_string(), source_id: o.source_id.to_string(), label: o.label, source_url: o.source_url, source_host: o.source_host, local_path: o.local_path, content_type: o.content_type, cached_at: o.cached_at.to_rfc3339(), keep_forever: o.keep_forever, bytes: o.bytes as i64 } } }

#[derive(SimpleObject, Clone)]
struct SnapshotGql { id: String, project_name: String, source: String, summary: String, collected_at: String }
impl From<ActivitySnapshot> for SnapshotGql { fn from(s: ActivitySnapshot) -> Self { Self { id: s.id.to_string(), project_name: s.project_name, source: s.source, summary: s.summary, collected_at: s.collected_at.to_rfc3339() } } }

#[derive(SimpleObject, Clone)]
struct DriveFileGql { path: String, size_bytes: i64, modified_at: Option<String> }
impl From<DriveFileEntry> for DriveFileGql { fn from(f: DriveFileEntry) -> Self { Self { path: f.path, size_bytes: f.size_bytes as i64, modified_at: f.modified_at.map(|v| v.to_rfc3339()) } } }

#[derive(SimpleObject, Clone)]
struct ReadmeReportGql { id: String, title: String, markdown: String, target_folder: String, file_path: String, generated_at: String }
impl From<ReadmeReport> for ReadmeReportGql { fn from(r: ReadmeReport) -> Self { Self { id: r.id.to_string(), title: r.title, markdown: r.markdown, target_folder: r.target_folder, file_path: r.file_path, generated_at: r.generated_at.to_rfc3339() } } }

#[derive(SimpleObject, Clone)]
struct FunctionGql { id: String, name: String, capability: String, current_revision: String, enabled: bool, no_rest: bool }
impl From<VersionlessFunction> for FunctionGql { fn from(f: VersionlessFunction) -> Self { Self { id: f.id.to_string(), name: f.name, capability: f.capability, current_revision: f.current_revision, enabled: f.enabled, no_rest: f.no_rest } } }

#[derive(InputObject)] struct RegisterDeviceInput { name: String, kind: String }
#[derive(InputObject)] struct RegisterDomainInput { domain: String, vps_ip: String, public_root: String, primary: bool }
#[derive(InputObject)] struct RegisterServerInput { name: String, vps_ip: String, upload_directory: String, public_root: String, ssh_or_ftp_hint: Option<String> }
#[derive(InputObject)] struct RegisterProjectInput { name: String, github_url: Option<String>, local_drive_path: Option<String>, vps_directory: Option<String> }
#[derive(InputObject)] struct RegisterOfflineSourceInput { label: String, source_url: String, category: String }
#[derive(InputObject)] struct ScanDriveInput { path: String, max_depth: Option<i32>, max_files: Option<i32> }

struct QueryRoot;
#[Object]
impl QueryRoot {
    async fn platform_status(&self) -> StatusGql {
        StatusGql { version: "v28".to_string(), transport: "GraphQL Federation / Cosmo ready".to_string(), no_rest: true, no_tauri: true, workspace: aruaru_core::workspace_root().to_string_lossy().to_string(), readme_folder: default_readme_folder().to_string_lossy().to_string(), project_bundle_folder: default_project_bundle_folder().to_string_lossy().to_string() }
    }
    async fn devices(&self, ctx: &Context<'_>) -> Vec<DeviceGql> { ctx.data_unchecked::<AppState>().state.lock().unwrap().devices.clone().into_iter().map(Into::into).collect() }
    async fn domains(&self, ctx: &Context<'_>) -> Vec<DomainGql> { ctx.data_unchecked::<AppState>().state.lock().unwrap().domains.clone().into_iter().map(Into::into).collect() }
    async fn servers(&self, ctx: &Context<'_>) -> Vec<ServerGql> { ctx.data_unchecked::<AppState>().state.lock().unwrap().servers.clone().into_iter().map(Into::into).collect() }
    async fn projects(&self, ctx: &Context<'_>) -> Vec<ProjectGql> { ctx.data_unchecked::<AppState>().state.lock().unwrap().projects.clone().into_iter().map(Into::into).collect() }
    async fn offline_sources(&self, ctx: &Context<'_>) -> Vec<OfflineSourceGql> { ctx.data_unchecked::<AppState>().state.lock().unwrap().offline_sources.clone().into_iter().map(Into::into).collect() }
    async fn offline_cache(&self, ctx: &Context<'_>, limit: Option<i32>) -> Vec<OfflineCacheEntryGql> {
        let mut items = ctx.data_unchecked::<AppState>().state.lock().unwrap().offline_cache.clone();
        items.reverse();
        items.into_iter().take(limit.unwrap_or(100).max(0) as usize).map(Into::into).collect()
    }
    async fn activity_snapshots(&self, ctx: &Context<'_>, limit: Option<i32>) -> Vec<SnapshotGql> {
        let mut items = ctx.data_unchecked::<AppState>().state.lock().unwrap().snapshots.clone();
        items.reverse();
        items.into_iter().take(limit.unwrap_or(50).max(0) as usize).map(Into::into).collect()
    }
    async fn drive_files(&self, ctx: &Context<'_>, limit: Option<i32>) -> Vec<DriveFileGql> {
        ctx.data_unchecked::<AppState>().state.lock().unwrap().drive_files.clone().into_iter().take(limit.unwrap_or(100).max(0) as usize).map(Into::into).collect()
    }
    async fn latest_readme_report(&self, ctx: &Context<'_>) -> Option<ReadmeReportGql> { ctx.data_unchecked::<AppState>().state.lock().unwrap().reports.last().cloned().map(Into::into) }
    async fn versionless_functions(&self, ctx: &Context<'_>) -> Vec<FunctionGql> { ctx.data_unchecked::<AppState>().functions.list().to_vec().into_iter().map(Into::into).collect() }
}

struct MutationRoot;
#[Object]
impl MutationRoot {
    async fn register_device(&self, ctx: &Context<'_>, input: RegisterDeviceInput) -> async_graphql::Result<DeviceGql> {
        if input.name.trim().is_empty() { return Err("device name is empty".into()); }
        let device = DeviceProfile { id: Uuid::new_v4(), name: input.name.trim().to_string(), kind: ClientKind::parse(&input.kind), trusted: true, last_seen_at: Some(Utc::now()) };
        let app = ctx.data_unchecked::<AppState>();
        app.state.lock().unwrap().devices.push(device.clone());
        app.persist().map_err(|e| e.to_string())?;
        Ok(device.into())
    }
    async fn register_domain(&self, ctx: &Context<'_>, input: RegisterDomainInput) -> async_graphql::Result<DomainGql> {
        validate_domain(&input.domain).map_err(|e| e.to_string())?;
        validate_ip_or_host(&input.vps_ip).map_err(|e| e.to_string())?;
        let domain = DomainProfile { id: Uuid::new_v4(), domain: input.domain.trim().to_string(), primary: input.primary, public_root: input.public_root, vps_ip: input.vps_ip, verified: false, registered_at: Utc::now() };
        let app = ctx.data_unchecked::<AppState>();
        let mut state = app.state.lock().unwrap();
        if domain.primary { for d in &mut state.domains { d.primary = false; } }
        state.domains.retain(|d| d.domain != domain.domain);
        state.domains.push(domain.clone());
        drop(state);
        app.persist().map_err(|e| e.to_string())?;
        Ok(domain.into())
    }
    async fn register_server(&self, ctx: &Context<'_>, input: RegisterServerInput) -> async_graphql::Result<ServerGql> {
        validate_ip_or_host(&input.vps_ip).map_err(|e| e.to_string())?;
        let server = ServerProfile { id: Uuid::new_v4(), name: input.name, vps_ip: input.vps_ip, upload_directory: input.upload_directory, public_root: input.public_root, ssh_or_ftp_hint: input.ssh_or_ftp_hint.unwrap_or_else(|| "WinSCP/FTP/SFTP upload target".to_string()), created_at: Utc::now() };
        let app = ctx.data_unchecked::<AppState>();
        app.state.lock().unwrap().servers.push(server.clone());
        app.persist().map_err(|e| e.to_string())?;
        Ok(server.into())
    }
    async fn register_project_source(&self, ctx: &Context<'_>, input: RegisterProjectInput) -> async_graphql::Result<ProjectGql> {
        if let Some(url) = &input.github_url { validate_github_url(url).map_err(|e| e.to_string())?; }
        if input.name.trim().is_empty() { return Err("project name is empty".into()); }
        let project = ProjectSource { id: Uuid::new_v4(), name: input.name.trim().to_string(), github_url: input.github_url, local_drive_path: input.local_drive_path, vps_directory: input.vps_directory, enabled: true, registered_at: Utc::now() };
        let app = ctx.data_unchecked::<AppState>();
        let saved = { app.state.lock().unwrap().add_or_update_project(project) };
        app.persist().map_err(|e| e.to_string())?;
        Ok(saved.into())
    }
    async fn register_offline_source(&self, ctx: &Context<'_>, input: RegisterOfflineSourceInput) -> async_graphql::Result<OfflineSourceGql> {
        aruaru_core::validate_http_or_https_url(&input.source_url).map_err(|e| e.to_string())?;
        if input.label.trim().is_empty() { return Err("label is empty".into()); }
        let item = OfflineSource { id: Uuid::new_v4(), label: input.label.trim().to_string(), source_url: input.source_url.trim().to_string(), category: input.category, enabled: true, last_cached_at: None };
        let app = ctx.data_unchecked::<AppState>();
        app.state.lock().unwrap().offline_sources.push(item.clone());
        app.persist().map_err(|e| e.to_string())?;
        Ok(item.into())
    }
    async fn scan_drive_now(&self, ctx: &Context<'_>, input: ScanDriveInput) -> async_graphql::Result<Vec<DriveFileGql>> {
        let mut policy = DriveScanPolicy::with_root(input.path.clone());
        if let Some(depth) = input.max_depth { policy.max_depth = depth.clamp(1, 12) as usize; }
        if let Some(max_files) = input.max_files { policy.max_files = max_files.clamp(1, 10_000) as usize; }
        let files = scan_allowlisted_path(&input.path, &policy).map_err(|e| e.to_string())?;
        let app = ctx.data_unchecked::<AppState>();
        app.state.lock().unwrap().drive_files = files.clone();
        app.persist().map_err(|e| e.to_string())?;
        Ok(files.into_iter().map(Into::into).collect())
    }
    async fn crawl_now(&self, ctx: &Context<'_>) -> async_graphql::Result<i32> {
        let app = ctx.data_unchecked::<AppState>();
        let planner = CrawlPlanner::new();
        let projects = app.state.lock().unwrap().projects.clone();
        let mut snapshots = Vec::new();
        for p in projects.iter().filter(|p| p.enabled) { snapshots.push(planner.collect_snapshot(p)); }
        let count = snapshots.len() as i32;
        app.state.lock().unwrap().snapshots.extend(snapshots);
        app.persist().map_err(|e| e.to_string())?;
        Ok(count)
    }
    async fn generate_readme_report(&self, ctx: &Context<'_>, llm_profile: Option<String>) -> async_graphql::Result<ReadmeReportGql> {
        let app = ctx.data_unchecked::<AppState>();
        let (snapshots, projects, domains, devices) = { let state = app.state.lock().unwrap(); (state.snapshots.clone(), state.projects.clone(), state.domains.clone(), state.devices.clone()) };
        let ai = AiRouter::new(llm_profile.as_deref().map(LlmProfile::by_name).unwrap_or_else(LlmProfile::gt730_safe_default));
        let report = ai.generate_github_readme_report(&snapshots, &projects, &domains, &devices);
        ai.write_report_file(&report).map_err(|e| e.to_string())?;
        app.state.lock().unwrap().reports.push(report.clone());
        app.persist().map_err(|e| e.to_string())?;
        Ok(report.into())
    }
    async fn run_full_update_cycle(&self, ctx: &Context<'_>) -> async_graphql::Result<ReadmeReportGql> {
        let report = ctx.data_unchecked::<AppState>().run_cycle().map_err(|e| e.to_string())?;
        Ok(report.into())
    }
    async fn mark_offline_sources_cached(&self, ctx: &Context<'_>) -> async_graphql::Result<i32> {
        let app = ctx.data_unchecked::<AppState>();
        let mut state = app.state.lock().unwrap();
        let now = Utc::now();
        for s in &mut state.offline_sources { if s.enabled { s.last_cached_at = Some(now); } }
        let count = state.offline_sources.iter().filter(|s| s.enabled).count() as i32;
        drop(state);
        app.persist().map_err(|e| e.to_string())?;
        Ok(count)
    }

    async fn run_offline_prefetch(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let (added, removed) = ctx.data_unchecked::<AppState>().run_offline_prefetch().map_err(|e| e.to_string())?;
        Ok(serde_json::to_string(&json!({
            "cached_items_added": added,
            "expired_items_removed": removed,
            "retention_days": 3,
            "audiocafe_tokyo_retained": true
        })).unwrap_or_else(|_| "{}".to_string()))
    }

    async fn purge_offline_cache(&self, ctx: &Context<'_>) -> async_graphql::Result<i32> {
        let removed = ctx.data_unchecked::<AppState>().purge_offline_cache().map_err(|e| e.to_string())?;
        Ok(removed as i32)
    }
    async fn invoke_versionless_function(&self, ctx: &Context<'_>, function_name: String) -> async_graphql::Result<String> {
        let invocation = FunctionInvocation { function_name, input: json!({}), requested_at: Utc::now() };
        let out = ctx.data_unchecked::<AppState>().functions.invoke(invocation).map_err(|e| e.to_string())?;
        Ok(serde_json::to_string(&out).unwrap_or_else(|_| "{}".to_string()))
    }
}

type AruaruSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
async fn graphql_handler(schema: Data<&AruaruSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::new()?;
    let scheduler_state = state.clone();
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(300));
        loop {
            tick.tick().await;
            if let Err(err) = scheduler_state.run_cycle() { eprintln!("aruaru scheduled cycle failed: {err}"); }
        }
    });
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).data(state).finish();
    let app = Route::new().at("/graphql", poem::post(graphql_handler).data(schema));
    let bind = std::env::var("ARUARU_BIND").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    println!("aruaru-gql-gateway v28 listening on http://{bind}/graphql");
    Server::new(TcpListener::bind(bind)).run(app).await?;
    Ok(())
}
