use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use crate::readme_output_menu::ReadmeOutputTarget;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutputMode {
    None,
    Rs,
    Html,
    Both,
}

impl OutputMode {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "none" | "off" | "no" | "readme.md" => Ok(Self::None),
            "rs" | "readme.rs" => Ok(Self::Rs),
            "html" | "readme.html" => Ok(Self::Html),
            "both" | "all" | "rs_and_html" => Ok(Self::Both),
            other => Err(anyhow!(
                "unknown output mode: {other}. Use none, rs, html, or both."
            )),
        }
    }

    pub fn writes_rs(&self) -> bool {
        matches!(self, Self::Rs | Self::Both)
    }

    pub fn writes_html(&self) -> bool {
        matches!(self, Self::Html | Self::Both)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Rs => "rs",
            Self::Html => "html",
            Self::Both => "both",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub root: PathBuf,
    pub listen: SocketAddr,
    pub output_mode: OutputMode,
    pub scan_interval_secs: u64,
    pub stable_wait_millis: u64,
    pub max_readme_bytes: u64,
    pub title_prefix: String,
    pub extra_outputs: Vec<ReadmeOutputTarget>,
}

impl AppConfig {
    pub fn from_env_and_args() -> Result<Self> {
        let mut root = env::var_os("ARUARU_README_ROOT")
            .map(PathBuf::from)
            .unwrap_or(env::current_dir().context("failed to read current dir")?);
        let mut listen: SocketAddr = env::var("ARUARU_README_LISTEN")
            .unwrap_or_else(|_| "127.0.0.1:7878".to_string())
            .parse()
            .context("invalid ARUARU_README_LISTEN")?;
        let mut output_mode = OutputMode::parse(
            &env::var("ARUARU_README_OUTPUT").unwrap_or_else(|_| "rs".to_string()),
        )?;
        let mut scan_interval_secs = env::var("ARUARU_README_SCAN_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        let mut stable_wait_millis = env::var("ARUARU_README_STABLE_WAIT_MILLIS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1_200);
        let mut max_readme_bytes = env::var("ARUARU_README_MAX_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1_048_576);
        let mut title_prefix =
            env::var("ARUARU_README_TITLE_PREFIX").unwrap_or_else(|_| "aruaru README".to_string());
        let mut extra_outputs = ReadmeOutputTarget::parse_csv(
            &env::var("ARUARU_README_EXTRA_OUTPUTS").unwrap_or_else(|_| "".to_string()),
        )?;

        let args: Vec<String> = env::args().collect();
        let mut i = 1usize;
        while i < args.len() {
            match args[i].as_str() {
                "--root" => {
                    i += 1;
                    root = PathBuf::from(args.get(i).context("--root requires a value")?);
                }
                "--listen" => {
                    i += 1;
                    listen = args
                        .get(i)
                        .context("--listen requires a value")?
                        .parse()
                        .context("invalid --listen value")?;
                }
                "--output" => {
                    i += 1;
                    output_mode =
                        OutputMode::parse(args.get(i).context("--output requires a value")?)?;
                }
                "--scan-interval-secs" => {
                    i += 1;
                    scan_interval_secs = args
                        .get(i)
                        .context("--scan-interval-secs requires a value")?
                        .parse()
                        .context("invalid --scan-interval-secs value")?;
                }
                "--stable-wait-millis" => {
                    i += 1;
                    stable_wait_millis = args
                        .get(i)
                        .context("--stable-wait-millis requires a value")?
                        .parse()
                        .context("invalid --stable-wait-millis value")?;
                }
                "--max-readme-bytes" => {
                    i += 1;
                    max_readme_bytes = args
                        .get(i)
                        .context("--max-readme-bytes requires a value")?
                        .parse()
                        .context("invalid --max-readme-bytes value")?;
                }
                "--title-prefix" => {
                    i += 1;
                    title_prefix = args
                        .get(i)
                        .context("--title-prefix requires a value")?
                        .clone();
                }
                "--extra-outputs" => {
                    i += 1;
                    extra_outputs = ReadmeOutputTarget::parse_csv(
                        args.get(i).context("--extra-outputs requires a value")?,
                    )?;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(anyhow!("unknown argument: {other}")),
            }
            i += 1;
        }

        if scan_interval_secs == 0 {
            return Err(anyhow!("scan interval must be 1 second or more"));
        }
        if stable_wait_millis < 200 {
            return Err(anyhow!("stable wait should be 200ms or more"));
        }
        if max_readme_bytes < 1024 {
            return Err(anyhow!("max README size should be 1024 bytes or more"));
        }

        Ok(Self {
            root,
            listen,
            output_mode,
            scan_interval_secs,
            stable_wait_millis,
            max_readme_bytes,
            title_prefix,
            extra_outputs,
        })
    }
}

pub fn print_help() {
    println!(
        "aruaru-readme-auto-rs\n\n\
         Watches a web root for FTP uploaded README.md files and generates optional README.* files.\n\n\
         Options:\n\
           --root <PATH>                 Web root to scan. Default: current dir or ARUARU_README_ROOT\n\
           --listen <ADDR:PORT>          Dashboard listen address. Default: 127.0.0.1:7878\n\
           --output <none|rs|html|both>  Legacy quick output selector. Default: rs\n\
           --extra-outputs <CSV>         Optional checkbox targets: rs,html,php,python,ts,js,go,java,csharp,kotlin,swift,ruby,json\n\
           --scan-interval-secs <N>      Default: 5\n\
           --stable-wait-millis <N>      Wait before reading after FTP upload. Default: 1200\n\
           --max-readme-bytes <N>        Default: 1048576\n\
           --title-prefix <TEXT>         HTML title prefix\n"
    );
}
