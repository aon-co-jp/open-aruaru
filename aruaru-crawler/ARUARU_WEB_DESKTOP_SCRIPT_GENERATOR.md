# aruaru-desktop / aruaru-web 標準機能: 手動スクリプト作成・自動BUGチェック連携

## 目的

aruaru-desktop 内の aruaru-web と連携し、ブラウザからも aruaru-ai を利用できるようにする。
さらに、aruaru-ai / aruaru-llm / iLumi.llm と連動して、以下を標準機能として扱う。

- ローカルLLMや補助ツールの自動ダウンロード計画
- 自動BUGチェック
- PowerShell / Bash の手動実行スクリプト作成
- 作成した手動スクリプトの説明書出力
- 実行ログ保存
- AIによるBUG要約
- ユーザー承認後の修正支援

## 基本方針

自動実行だけに頼らず、必ず手動で再現できるスクリプトを生成する。
これにより、aruaru-desktopが使えない環境、ブラウザ操作が止まった環境、PowerShell設定で止まった環境でも、ユーザーが自分でBUGチェックを再実行できる。

## 標準画面案

```text
aruaru-desktop
└─ aruaru-web
   └─ aruaru-ai
      ├─ プロジェクト選択
      ├─ LLM選択
      │  ├─ aruaru-ai 標準
      │  ├─ iLumi.llm ローカル
      │  ├─ API型LLM
      │  └─ ローカルLLM
      ├─ 自動ダウンロード
      ├─ 自動BUGチェック
      ├─ 手動スクリプト作成
      │  ├─ Windows PowerShell用
      │  ├─ Linux/macOS Bash用
      │  └─ 説明書つき出力
      ├─ 実行ログ
      ├─ AI要約
      └─ 修正候補・差分・承認
```

## 手動スクリプト作成機能

aruaru-web から以下を選択して生成する。

```text
対象プロジェクト:
F:\aruaru\aruaru-rs4

出力するスクリプト:
- aruaru-manual-bugcheck.ps1
- aruaru-manual-bugcheck.sh

含めるチェック:
- cargo fmt
- cargo check
- cargo test
- cargo clippy
- 禁止仕様混入チェック
- README生成スモークテスト
- ローカルLLMダウンロード前チェック
```

## Windows PowerShellでの使い方

プロジェクトルートから実行する場合:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-bugcheck-template.ps1
```

scripts/manual フォルダ内から実行する場合:

```powershell
powershell -ExecutionPolicy Bypass -File .\aruaru-manual-bugcheck-template.ps1
```

PowerShellの未署名エラーが出る場合は、PC全体の設定を変更せず、上記の `-ExecutionPolicy Bypass` を使う。

## Bashでの使い方

```bash
chmod +x ./scripts/manual/aruaru-manual-bugcheck-template.sh
./scripts/manual/aruaru-manual-bugcheck-template.sh
```

## 自動ダウンロードの安全ルール

aruaru-ai は自動ダウンロード前に必ず次を確認する。

```text
[ ] ダウンロード元URLが登録済みか
[ ] ライセンスが許可されているか
[ ] 必要ディスク容量が足りるか
[ ] ハッシュ確認が可能か
[ ] モデルサイズがPC性能に合っているか
[ ] ユーザー承認があるか
```

API型LLMはダウンロード対象ではなく、APIキー登録とモデル選択で扱う。
ローカルLLM、GGUF、Ollama/LM Studio互換モデル、iLumi.llm などを自動ダウンロード対象にする。

## 自動BUGチェックの流れ

```text
1. プロジェクト選択
2. Cargo.toml 検出
3. check-full.ps1 検出
4. 署名エラー時は一回だけ Bypass 実行
5. cargo fmt / check / test / clippy
6. README生成テスト
7. ログ保存
8. aruaru-ai がログ要約
9. aruaru-llm が修正候補作成
10. 差分表示
11. ユーザー承認
12. 修正適用
13. 再BUGチェック
```

## 実装メモ

Rust実装側には `src/manual_script_generator.rs` を追加した。
これにより、aruaru-web の画面からスクリプト本文と利用方法を同時に生成できる。

## 品質ゲート

```text
[ ] ブラウザから手動スクリプトを作成できる
[ ] Windows PowerShell用スクリプトを出力できる
[ ] Bash用スクリプトを出力できる
[ ] 説明書を同時に出力できる
[ ] 自動BUGチェックと手動BUGチェックの内容が大きくズレない
[ ] PowerShellの未署名エラー時に案内できる
[ ] ユーザー承認なしに自動修正しない
[ ] ローカルLLMだけ自動ダウンロード対象にする
[ ] API型LLMを誤ってダウンロード対象にしない
```
