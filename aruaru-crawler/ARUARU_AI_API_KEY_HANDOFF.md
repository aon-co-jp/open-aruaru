# aruaru-ai API Key Handoff / Import Wizard

aruaru-ai には、Claude Opus、OpenAI / ChatGPT、Gemini、DeepSeek などの有料API型LLMを使うための **API KEY 引き継ぎ・移行・安全保存機能** を標準機能として追加します。

## 基本方針

API KEY は、本人または同じ組織・同じプロジェクトで利用権限があるキーだけを引き継げます。

この機能は、他人のキー、ブラウザ内のキー、パスワードマネージャー内のキー、別アプリの秘密情報を勝手に探して取り出す機能ではありません。

## 対応する取り込み元

- 手動貼り付け
- 環境変数
- aruaru encrypted vault
- 選択した暗号化バックアップファイル
- Windows Credential Manager
- macOS Keychain
- Linux Secret Service

## 対応する保存先

- セッション内だけ保存
- OS標準の秘密情報ストア
- aruaru encrypted vault
- ユーザー環境変数

## 重要な安全ルール

- API KEY を README.md / README.html / README.rs に書かない
- API KEY をGitにコミットしない
- API KEY をログに出さない
- API KEY は画面では `sk-1...abcd` のようにマスク表示する
- `.env`、`.aruaru/secrets`、vaultファイルは `.gitignore` に入れる
- 有料APIは利用料金が発生するため、初回利用前に確認画面を出す
- 自動BUGチェックに送るファイルから `.env`、秘密鍵、API KEY、SSHキー、`.git`、`target`、`node_modules` を除外する

## OpenAI / ChatGPT API

OpenAI APIキーは、コードや公開リポジトリに埋め込まず、安全な場所に保存する必要があります。OpenAIの公式ベストプラクティスでは、APIキーをコードや公開リポジトリに置かず、チームメンバーごとに一意のキーを使い、クライアント側に公開しないことが推奨されています。

推奨環境変数:

```text
OPENAI_API_KEY
```

## Anthropic Claude / Opus API

Claude / Opus はAPI接続で利用します。AnthropicのConsoleではワークスペース、メンバー、APIキーを管理でき、組織向けにはAPIキー管理やワークスペース管理の仕組みがあります。

推奨環境変数:

```text
ANTHROPIC_API_KEY
```

## aruaru-ai の取り込みフロー

```text
1. プロバイダーを選択
2. API KEYの取り込み元を選択
3. 保存先を選択
4. キーをマスク表示で確認
5. 有料API利用の注意を表示
6. ユーザーが利用権限を確認
7. 必要に応じて最小API疎通テスト
8. aruaru-aiのBUGチェックで利用
9. ログにはキーを残さない
```

## 手動移行の例

PowerShellで一時的に利用する場合:

```powershell
$env:ANTHROPIC_API_KEY = "ここに自分のAPIキー"
```

ユーザー環境変数として保存する場合:

```powershell
[Environment]::SetEnvironmentVariable("ANTHROPIC_API_KEY", "ここに自分のAPIキー", "User")
```

ただし、長期保存はOS標準の秘密情報ストアまたは aruaru encrypted vault を推奨します。

## 自動BUGチェックとの連携

API KEY を取り込むと、aruaru-ai は選択したLLMで以下を実行できます。

- cargoログ解析
- Rustコンパイルエラー分類
- README変換BUG解析
- PowerShellエラー分類
- 修正案生成
- 再発防止テスト案生成

ただし、自動修正は必ずユーザー承認後に行います。
