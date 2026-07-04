use anyhow::Result;
use aruaru_core::{default_readme_folder, ensure_workspace_dirs, ActivitySnapshot, DeviceProfile, DomainProfile, ProjectSource, ReadmeReport};
use aruaru_llm::LlmProfile;
use chrono::Utc;
use std::{fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AiRouter { pub profile: LlmProfile }
impl AiRouter { pub fn new(profile: LlmProfile) -> Self { Self { profile } } }

impl AiRouter {
    pub fn generate_github_readme_report(&self, snapshots: &[ActivitySnapshot], projects: &[ProjectSource], domains: &[DomainProfile], devices: &[DeviceProfile]) -> ReadmeReport {
        let mut markdown = String::from("# aruaru project activity report\n\n");
        markdown.push_str("このREADMEは aruaru-ai / aruaru-llm による GitHub UPLOAD 用の自動レポートです。\n\n");
        markdown.push_str("## AI routing\n\n");
        markdown.push_str(&format!("- provider: `{:?}`\n", self.profile.provider));
        markdown.push_str(&format!("- model: `{}`\n", self.profile.model_name));
        markdown.push_str(&format!("- low_power_mode: `{}`\n", self.profile.low_power_mode));
        markdown.push_str(&format!("- local_gpu_required: `{}`\n\n", self.profile.local_gpu_required));
        markdown.push_str("## Registered domains\n\n");
        if domains.is_empty() { markdown.push_str("- 未登録\n"); } else { for d in domains { markdown.push_str(&format!("- {} → {} / public_root `{}` / primary `{}`\n", d.domain, d.vps_ip, d.public_root, d.primary)); } }
        markdown.push_str("\n## Linked devices\n\n");
        if devices.is_empty() { markdown.push_str("- 未登録\n"); } else { for d in devices { markdown.push_str(&format!("- {} / {:?} / trusted `{}`\n", d.name, d.kind, d.trusted)); } }
        markdown.push_str("\n## aruaru related projects\n\n");
        if projects.is_empty() { markdown.push_str("- 未登録\n"); } else { for p in projects { markdown.push_str(&format!("- **{}**\n  - GitHub: `{}`\n  - Local: `{}`\n  - VPS: `{}`\n", p.name, p.github_url.clone().unwrap_or_default(), p.local_drive_path.clone().unwrap_or_default(), p.vps_directory.clone().unwrap_or_default())); } }
        markdown.push_str("\n## Collected activity\n\n");
        if snapshots.is_empty() { markdown.push_str("- まだ収集結果がありません。GitHub URL、ローカルドライブ、VPSディレクトリを登録してください。\n"); } else { for s in snapshots.iter().rev().take(50) { markdown.push_str(&format!("- **{}** / {} / {} / {}\n", s.project_name, s.source, s.summary, s.collected_at.to_rfc3339())); } }
        markdown.push_str("\n## Next actions\n\n- aruaru-web の reports/latest で表示\n- `workspace/aruaru（README）/README.md` を GitHub 用READMEとしてUPLOAD可能\n- 5分単位クロールで継続更新\n");
        let folder = default_readme_folder();
        let file_path = folder.join("README.md");
        ReadmeReport { id: Uuid::new_v4(), title: "aruaru README report".to_string(), markdown, target_folder: folder.to_string_lossy().to_string(), file_path: file_path.to_string_lossy().to_string(), generated_at: Utc::now() }
    }

    pub fn write_report_file(&self, report: &ReadmeReport) -> Result<PathBuf> {
        ensure_workspace_dirs()?;
        let path = PathBuf::from(&report.file_path);
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        fs::write(&path, &report.markdown)?;
        Ok(path)
    }
}
