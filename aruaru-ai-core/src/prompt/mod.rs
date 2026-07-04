//! プロンプトテンプレート — VPS操作・ターミナル・SFTP 用の指示をコマンドに変換

/// 自然言語の指示 → シェルコマンドリストに変換するプロンプトを構築
pub fn build_command_prompt(instruction: &str, os_info: &str) -> String {
    format!(
        r#"あなたは Linux VPS の自動操作エージェントです。
ユーザーの指示を安全なシェルコマンドのリストに変換してください。

OS情報: {os_info}

指示: {instruction}

以下の JSON 形式でコマンドリストを返してください（説明文は不要）:
{{
  "commands": ["コマンド1", "コマンド2"],
  "description": "何をするか一行で",
  "safe": true
}}

注意:
- 破壊的なコマンド（rm -rf / など）は safe: false にして commands を空にしてください
- sudo が必要なコマンドには sudo を付けてください
- 一度に最大 10 コマンドまで"#
    )
}

/// SFTP ファイル操作の指示 → 操作リストに変換
pub fn build_sftp_prompt(instruction: &str, current_path: &str) -> String {
    format!(
        r#"あなたは SFTP ファイル操作エージェントです。
現在のパス: {current_path}

指示: {instruction}

以下の JSON 形式で操作リストを返してください:
{{
  "operations": [
    {{"type": "upload|download|delete|mkdir|rename", "path": "...", "dest": "..."}}
  ],
  "message": "実行内容の説明"
}}"#
    )
}

/// VPS セットアップの指示 → 自動化スクリプトに変換
pub fn build_setup_prompt(instruction: &str) -> String {
    format!(
        r#"あなたは VPS セットアップ自動化エージェントです。

セットアップ指示: {instruction}

Nginx/Apache のインストール、証明書取得、Rust アプリのデプロイなどに対応します。
以下の JSON 形式でステップリストを返してください:
{{
  "steps": [
    {{"step": 1, "description": "説明", "commands": ["cmd1", "cmd2"]}}
  ],
  "total_steps": 1
}}"#
    )
}
