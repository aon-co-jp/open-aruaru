mod git_ops;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 520.0])
            .with_min_inner_size([560.0, 420.0])
            .with_title("aruaru-github"),
        ..Default::default()
    };
    eframe::run_native(
        "aruaru-github",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

enum WorkerMsg {
    Log(String),
    Done(Result<(), String>),
}

struct App {
    folder: PathBuf,
    remote_url: String,
    branch: String,
    commit_message: String,
    log: String,
    busy: bool,
    tx: Sender<WorkerMsg>,
    rx: Receiver<WorkerMsg>,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            folder: PathBuf::new(),
            remote_url: String::new(),
            branch: "main".into(),
            commit_message: "Update via aruaru-github".into(),
            log: "フォルダを選んで「GitHub に Push」を押してください。\n".into(),
            busy: false,
            tx,
            rx,
        }
    }

    fn append_log(&mut self, line: impl AsRef<str>) {
        self.log.push_str(line.as_ref());
        if !self.log.ends_with('\n') {
            self.log.push('\n');
        }
    }

    fn pick_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.folder = path.clone();
            self.append_log(format!("選択: {}", path.display()));
            if let Ok(url) = git_ops::detect_remote(&path) {
                if !url.is_empty() {
                    self.remote_url = url.clone();
                    self.append_log(format!("remote 検出: {url}"));
                }
            }
        }
    }

    fn start_push(&mut self) {
        if self.busy {
            return;
        }
        if self.folder.as_os_str().is_empty() {
            self.append_log("エラー: フォルダを選択してください");
            return;
        }
        if self.remote_url.trim().is_empty() {
            self.append_log("エラー: GitHub リポジトリ URL を入力してください");
            return;
        }

        self.busy = true;
        self.append_log("--- Push 開始 ---");

        let folder = self.folder.clone();
        let remote = self.remote_url.trim().to_string();
        let branch = self.branch.trim().to_string();
        let message = self.commit_message.trim().to_string();
        let tx = self.tx.clone();

        thread::spawn(move || {
            let run = || -> anyhow::Result<()> {
                git_ops::push_folder(&folder, &remote, &branch, &message, |s| {
                    let _ = tx.send(WorkerMsg::Log(s));
                })?;
                Ok(())
            };
            let result = run().map_err(|e| e.to_string());
            let _ = tx.send(WorkerMsg::Done(result));
        });
    }

    fn poll_worker(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMsg::Log(s) => self.append_log(s),
                WorkerMsg::Done(res) => {
                    self.busy = false;
                    match res {
                        Ok(()) => self.append_log("✓ GitHub Push 完了"),
                        Err(e) => self.append_log(format!("✗ 失敗: {e}")),
                    }
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("aruaru-github");
            ui.label("Windows フォルダを指定して GitHub に Push");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("フォルダ:");
                let folder_text = if self.folder.as_os_str().is_empty() {
                    "（未選択）".to_string()
                } else {
                    self.folder.display().to_string()
                };
                ui.label(egui::RichText::new(folder_text).weak());
                if ui.button("📁 参照…").clicked() {
                    self.pick_folder();
                }
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("GitHub URL:");
                ui.text_edit_singleline(&mut self.remote_url);
            });
            ui.label(
                egui::RichText::new("例: https://github.com/user/repo.git")
                    .small()
                    .weak(),
            );

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("ブランチ:");
                ui.text_edit_singleline(&mut self.branch);
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("コミット:");
                ui.text_edit_singleline(&mut self.commit_message);
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let push = ui
                    .add_enabled(!self.busy, egui::Button::new("🚀 GitHub に Push"))
                    .clicked();
                if push {
                    self.start_push();
                }
                if self.busy {
                    ui.spinner();
                    ui.label("処理中…");
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.label("ログ");
            egui::ScrollArea::vertical()
                .max_height(f32::INFINITY)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.log)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Git をインストール").clicked() {
                    let _ = open::that("https://git-scm.com/download/win");
                }
                if ui.button("GitHub で PAT 作成").clicked() {
                    let _ = open::that("https://github.com/settings/tokens");
                }
            });
        });

        if self.busy {
            ctx.request_repaint();
        }
    }
}
