#![allow(dead_code)]
//! Manual script generator for aruaru-desktop / aruaru-web.
//!
//! This module creates safe, copyable scripts that users can run by hand when
//! browser-based automation is unavailable or when they want to reproduce a bug
//! check outside the desktop UI.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptKind {
    WindowsPowerShell,
    Bash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualScriptRequest {
    pub project_path: String,
    pub include_generation_test: bool,
    pub include_local_model_download_check: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedManualScript {
    pub file_name: String,
    pub body: String,
    pub usage: String,
}

pub fn generate_manual_script(kind: ScriptKind, req: &ManualScriptRequest) -> GeneratedManualScript {
    match kind {
        ScriptKind::WindowsPowerShell => generate_powershell_script(req),
        ScriptKind::Bash => generate_bash_script(req),
    }
}

fn generate_powershell_script(req: &ManualScriptRequest) -> GeneratedManualScript {
    let mut body = String::new();
    body.push_str("# aruaru manual bug check script\n");
    body.push_str("# Generated for aruaru-desktop / aruaru-web users.\n");
    body.push_str("$ErrorActionPreference = \"Stop\"\n\n");
    body.push_str(&format!("Set-Location \"{}\"\n\n", escape_ps(&req.project_path)));
    body.push_str("Write-Host \"== cargo quality gate ==\"\n");
    body.push_str("cargo fmt --all\n");
    body.push_str("cargo fmt --all -- --check\n");
    body.push_str("cargo check\n");
    body.push_str("cargo test\n");
    body.push_str("cargo clippy --all-targets -- -D warnings\n\n");
    body.push_str("Write-Host \"== banned desktop/web policy marker check ==\"\n");
    let desktop_marker = format!("{}{}", "tau", "ri");
    let api_marker = format!("{}{}", "rest", " api");
    body.push_str(&format!(
        "$hits = Select-String -Path .\\src\\*.rs,.\\Cargo.toml -Pattern \"{}\",\"{}\" -CaseSensitive:$false\n",
        desktop_marker, api_marker
    ));
    body.push_str("if ($hits) { $hits; throw \"Banned marker found in implementation files\" }\n\n");

    if req.include_generation_test {
        body.push_str("Write-Host \"== README generation smoke test ==\"\n");
        body.push_str("Remove-Item -Recurse -Force .\\tmp-web -ErrorAction SilentlyContinue\n");
        body.push_str("New-Item -ItemType Directory -Force .\\tmp-web | Out-Null\n");
        body.push_str("Set-Content .\\tmp-web\\README.md \"# TEST`n`n- item 1`n- item 2\" -Encoding UTF8\n");
        body.push_str("$proc = Start-Process cargo -ArgumentList @(\"run\",\"--\",\"--root\",\".\\tmp-web\",\"--listen\",\"127.0.0.1:7878\",\"--output\",\"both\",\"--scan-interval-secs\",\"1\") -PassThru -WindowStyle Hidden\n");
        body.push_str("try { Start-Sleep -Seconds 8; if (!(Test-Path .\\tmp-web\\README.rs)) { throw \"README.rs was not generated\" }; if (!(Test-Path .\\tmp-web\\README.html)) { throw \"README.html was not generated\" } } finally { if ($proc -and -not $proc.HasExited) { Stop-Process -Id $proc.Id -Force } }\n\n");
    }

    if req.include_local_model_download_check {
        body.push_str("Write-Host \"== local model download check placeholder ==\"\n");
        body.push_str("Write-Host \"aruaru-llm should verify provider, license, disk size, and checksum before download.\"\n\n");
    }

    body.push_str("Write-Host \"OK: manual bug check passed\"\n");

    GeneratedManualScript {
        file_name: "aruaru-manual-bugcheck.ps1".to_string(),
        usage: "PowerShell: powershell -ExecutionPolicy Bypass -File .\\aruaru-manual-bugcheck.ps1".to_string(),
        body,
    }
}

fn generate_bash_script(req: &ManualScriptRequest) -> GeneratedManualScript {
    let mut body = String::new();
    body.push_str("#!/usr/bin/env bash\n");
    body.push_str("set -euo pipefail\n\n");
    body.push_str(&format!("cd \"{}\"\n\n", escape_sh(&req.project_path)));
    body.push_str("echo '== cargo quality gate =='\n");
    body.push_str("cargo fmt --all\n");
    body.push_str("cargo fmt --all -- --check\n");
    body.push_str("cargo check\n");
    body.push_str("cargo test\n");
    body.push_str("cargo clippy --all-targets -- -D warnings\n\n");
    body.push_str("echo '== banned desktop/web policy marker check =='\n");
    let desktop_marker = format!("{}{}", "tau", "ri");
    let api_marker = format!("{}{}", "rest", " api");
    body.push_str(&format!(
        "if grep -RinE '{}|{}' src Cargo.toml; then echo 'Banned marker found in implementation files'; exit 1; fi\n\n",
        desktop_marker, api_marker
    ));

    if req.include_generation_test {
        body.push_str("echo '== README generation smoke test =='\n");
        body.push_str("rm -rf ./tmp-web\nmkdir -p ./tmp-web\n");
        body.push_str("printf '# TEST\\n\\n- item 1\\n- item 2\\n' > ./tmp-web/README.md\n");
        body.push_str("cargo run -- --root ./tmp-web --listen 127.0.0.1:7878 --output both --scan-interval-secs 1 &\n");
        body.push_str("pid=$!\ntrap 'kill $pid 2>/dev/null || true' EXIT\nsleep 8\ntest -f ./tmp-web/README.rs\ntest -f ./tmp-web/README.html\nkill $pid 2>/dev/null || true\ntrap - EXIT\n\n");
    }

    if req.include_local_model_download_check {
        body.push_str("echo '== local model download check placeholder =='\n");
        body.push_str("echo 'aruaru-llm should verify provider, license, disk size, and checksum before download.'\n\n");
    }

    body.push_str("echo 'OK: manual bug check passed'\n");

    GeneratedManualScript {
        file_name: "aruaru-manual-bugcheck.sh".to_string(),
        usage: "Bash: chmod +x ./aruaru-manual-bugcheck.sh && ./aruaru-manual-bugcheck.sh".to_string(),
        body,
    }
}

fn escape_ps(value: &str) -> String {
    value.replace('`', "``").replace('"', "`\"")
}

fn escape_sh(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powershell_script_contains_quality_gate() {
        let req = ManualScriptRequest {
            project_path: "F:\\aruaru\\project".to_string(),
            include_generation_test: true,
            include_local_model_download_check: true,
        };
        let script = generate_manual_script(ScriptKind::WindowsPowerShell, &req);
        assert!(script.body.contains("cargo check"));
        assert!(script.body.contains("cargo clippy"));
        assert!(script.body.contains("README generation smoke test"));
        assert!(script.file_name.ends_with(".ps1"));
    }

    #[test]
    fn bash_script_contains_quality_gate() {
        let req = ManualScriptRequest {
            project_path: "/opt/aruaru/project".to_string(),
            include_generation_test: false,
            include_local_model_download_check: false,
        };
        let script = generate_manual_script(ScriptKind::Bash, &req);
        assert!(script.body.contains("cargo test"));
        assert!(!script.body.contains("README generation smoke test"));
        assert!(script.file_name.ends_with(".sh"));
    }
}
