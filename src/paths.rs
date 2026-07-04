//! ユーザー向けパス表示 — ドライブ名から `F:/aruaru/...` 形式

use std::path::Path;

/// メッセージ・引き継ぎ文書用。バックスラッシュを `/` にし、ドライブレターから表示する。
pub fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// `root` 配下の相対パスを絶対表示パスに変換する。
pub fn display_path_under(root: &Path, rel: &str) -> String {
    display_path(&root.join(rel))
}

/// 存在する場合は canonicalize してから表示（失敗時はそのまま）。
pub fn display_path_canonical(path: &Path) -> String {
    let resolved = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    display_path(&resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_uses_forward_slashes() {
        let p = PathBuf::from(r"F:\aruaru\docs\HANDOFF.md");
        assert_eq!(display_path(&p), "F:/aruaru/docs/HANDOFF.md");
    }

    #[test]
    fn display_under_joins_root() {
        let root = PathBuf::from(r"F:\aruaru");
        assert_eq!(
            display_path_under(&root, "docs/HANDOFF.md"),
            "F:/aruaru/docs/HANDOFF.md"
        );
    }
}
