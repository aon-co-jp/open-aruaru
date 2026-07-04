use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod install_target;
mod serve;
mod uninstall_target;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "aruaru", version = VERSION)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Serve,
    Install { target: String },
    Uninstall {
        id: String,
        /// recovery 等のプラグインデータも削除
        #[arg(long)]
        purge: bool,
    },
    Reminders,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let data_dir = env_path("ARUARU_DATA_DIR", "./data");
    let root_dir = env_path("ARUARU_ROOT", ".");

    match args.command {
        None | Some(Command::Serve) => serve::serve(&data_dir, &root_dir).await,
        Some(Command::Install { target }) => {
            let app = crate::state::AppState::new(&data_dir, &root_dir).await?;
            install_target::run(&app, &target).await
        }
        Some(Command::Uninstall { id, purge }) => {
            let app = crate::state::AppState::new(&data_dir, &root_dir).await?;
            uninstall_target::run(&app, &id, purge).await
        }
        Some(Command::Reminders) => {
            let app = crate::state::AppState::new(&data_dir, &root_dir).await?;
            let urls = crate::auth::send_reminder_links(&app).await?;
            for u in urls {
                println!("{u}");
            }
            Ok(())
        }
    }
}

fn env_path(key: &str, default: &str) -> PathBuf {
    std::env::var(key).map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(default))
}
