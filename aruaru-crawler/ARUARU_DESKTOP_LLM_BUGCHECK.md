# aruaru-desktop 追加機能: aruaru-ai / aruaru-llm / iLumi.llm 連動 自動BUGチェック

## 目的

Windows版 aruaru-desktop から、PowerShell操作、Rust品質ゲート、aruaru-ai解析、aruaru-llmモデル選択、iLumi.llmローカル解析、商用API型LLMを統合して、BUGチェックと修正支援を自動化する。

## LLM分類

### API接続型

以下は本体を自動ダウンロードしない。APIキーを登録して利用する。

- OpenAI / ChatGPT API
- Anthropic Claude / Opus / Sonnet API
- Gemini API
- DeepSeek API
- その他API型LLM

### ローカル自動ダウンロード型

以下はライセンス確認後に自動ダウンロード対象にできる。

- iLumi.llm
- GGUFモデル
- Ollama対応モデル
- LM Studio対応モデル
- Hugging Face公開モデル
- Qwen / DeepSeek / Llama / Mistral 系のローカル実行可能モデル

## aruaru-desktop 画面案

```text
AI BUGチェック
├─ プロジェクト選択
├─ LLM選択
│  ├─ aruaru-ai 標準
│  ├─ iLumi.llm ローカル
│  ├─ OpenAI / ChatGPT API
│  ├─ Claude / Opus API
│  ├─ Gemini API
│  └─ DeepSeek API / Local
├─ BUGチェック
│  ├─ 通常チェック
│  ├─ フルチェック
│  ├─ README変換チェック
│  └─ 禁止仕様チェック
├─ 結果
│  ├─ PASS / FAIL
│  ├─ エラー分類
│  ├─ 修正候補
│  └─ 再発防止TEST候補
└─ 操作
   ├─ 差分を見る
   ├─ 修正を適用
   ├─ もう一度チェック
   └─ BUG報告として保存
```

## 安全ルール

- ChatGPT / Claude / Opus 本体を自動ダウンロードしない
- API型LLMはAPI接続として扱う
- ローカルLLMのみ自動ダウンロード対象にする
- 自動修正はユーザー承認後のみ
- 修正前にバックアップを作成する
- 差分を表示してから適用する
- `Set-ExecutionPolicy` を勝手に実行しない
- PC全体のセキュリティ設定を勝手に変更しない
- 管理者権限を勝手に要求しない
- `.env`、秘密鍵、APIキー、トークンをログに平文出力しない
- REST API / Tauri を復活させない

## 自動BUGチェック手順

1. プロジェクトフォルダ選択
2. `Cargo.toml` 自動検出
3. `scripts/check-full.ps1` 自動検出
4. 通常実行を試す
5. 未署名エラーなら `powershell -ExecutionPolicy Bypass -File ...` に自動切替
6. cargo品質ゲートを実行
7. 禁止仕様チェックを実行
8. ログを `.aruaru/logs/latest.log` に保存
9. aruaru-ai がエラー分類
10. aruaru-llm が選択LLMへ解析依頼
11. 修正案と再発防止テスト案を生成
12. 差分表示
13. ユーザー承認
14. 修正適用
15. 再BUGチェック

## エラー分類例

- `UnauthorizedAccess`: PowerShell実行ポリシーエラー
- `E0255`: Rust名前衝突
- `E0599`: trait import不足 / メソッド未検出
- `PathNotFound`: 実行場所または相対パスBUG
- `could not compile`: コンパイル失敗
- `clippy warnings`: 品質ゲート失敗

## 設定ファイル案

```toml
[llm]
default_provider = "aruaru-ai"
fallback_provider = "ilumi-local"

[providers.openai]
enabled = false
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-5.5"

[providers.anthropic]
enabled = false
api_key_env = "ANTHROPIC_API_KEY"
default_model = "claude-opus"

[providers.ilumi]
enabled = true
mode = "local"
auto_download = true

[bugcheck]
allow_powershell_bypass = true
auto_fix = false
require_user_approval = true
save_logs = true
```

## 品質ゲート

- LLMを選択できる
- API型LLMとローカルLLMを区別できる
- ChatGPT / Claude本体を誤ってダウンロードしようとしない
- ローカルLLMだけ自動ダウンロードできる
- APIキーを安全に環境変数経由で参照できる
- PowerShell未署名エラーを検出できる
- `ExecutionPolicy Bypass` で1回だけ実行できる
- `cargo fmt / check / test / clippy` を自動実行できる
- `E0255 / E0599 / PathNotFound` を分類できる
- scriptsフォルダからでもルート基準で実行できる
- ログを保存できる
- AIが修正案を作れる
- 差分を表示できる
- ユーザー承認後のみ修正できる
- 修正後に再BUGチェックできる
- Tauri / REST APIを復活させない
