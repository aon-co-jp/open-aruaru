# aruaru-readme-auto-rs

FTP / SFTP / ファイルマネージャーで Web サイト配下へ `README.md` をアップロードしたときに、同じフォルダへ `README.rs` を自動生成する Rust + Poem 製サーバー用アプリです。

`README.rs` には、スマホ縦横、タブレット、PC、1280x720、1280x800、1280x960、1366x768、1920x1080、WQHD、4K、8K、16K を想定したレスポンシブ HTML が `README_HTML` 定数として入ります。

必要な場合は `--output both` で `README.html` も同時生成できます。

## 目的

- GitHub 用の `README.md` を Web 表示にも使う
- FTP アップロード後に手作業で HTML 化しない
- スマホから 16K まで読みやすいレイアウトに自動変換する
- JavaScript ではなく TypeScript 相当の最小ブラウザ補助だけを HTML に入れる
- REST API / Tauri は使わない

## 起動例

```bash
cargo run -- --root /var/www/html --listen 127.0.0.1:7878 --output rs
```

HTMLも同時に欲しい場合:

```bash
cargo run -- --root /var/www/html --listen 127.0.0.1:7878 --output both
```

## 動作

1. `/var/www/html` 以下を定期スキャンします。
2. `README.md` を見つけます。
3. FTP アップロード中の途中読みを避けるため、少し待ってからサイズと更新時刻を再確認します。
4. Markdown を安全な HTML に変換します。
5. 同じフォルダへ `README.rs` を atomic write で生成します。
6. `--output both` の場合は `README.html` も生成します。

## 環境変数

| 変数 | 内容 | 既定値 |
|---|---|---|
| `ARUARU_README_ROOT` | 監視するWebルート | カレントディレクトリ |
| `ARUARU_README_LISTEN` | 管理画面の待受 | `127.0.0.1:7878` |
| `ARUARU_README_OUTPUT` | `rs` / `html` / `both` | `rs` |
| `ARUARU_README_SCAN_INTERVAL_SECS` | スキャン間隔 | `5` |
| `ARUARU_README_STABLE_WAIT_MILLIS` | FTPアップロード安定待ち | `1200` |
| `ARUARU_README_MAX_BYTES` | README.md最大サイズ | `1048576` |



## aruaru-llm DeepSeek Folding Engine 追加

aruaru-llm には、DeepSeek 系の公開技術を調査・分析したうえで、普通のPCや1枚GPUでもBUGチェック・README生成・長文ログ解析を行いやすくする **DeepSeek Folding Engine** を追加予定です。

この機能は、巨大スーパーコンピューター上のLLMをそのまま1枚GPUで完全再現するものではありません。DeepSeek-V3 / R1 / Native Sparse Attention で公開されている考え方を、aruaru向けに **圧縮・凝縮・折り畳み** して実装する研究機能です。

aruaruでの「折り畳み新理論」は以下です。

```text
1. Model Folding
   大型モデルの能力を、小型蒸留モデル・量子化モデル・APIフォールバックへ分解する。

2. Expert Folding
   BUG、PowerShell、README、Poem、GraphQLなどの専門処理へ分ける。

3. Context Folding
   長文ログやREADMEを、重要行・要約・再読込可能ファイル参照へ折り畳む。

4. KV Folding
   重い文脈を毎回LLMへ渡さず、要約・ハッシュ・重要断片として保持する。

5. Repair Folding
   BUG原因、修正案、差分、再発防止テスト、再チェックを1つの修正計画にまとめる。
```

実装済みの実験ファイル:

```text
src/llm_folding.rs
ARUARU_LLM_DEEPSEEK_FOLDING.md
```

GT730のような小型GPUでは、巨大モデルを直接GPU実行するのではなく、ログ要約・エラー分類・小型蒸留モデル・CPU/Hybrid・APIフォールバックを組み合わせます。RTX 3060 12GB以上では、7B〜14B級、RTX 3090/4090 24GBでは14B〜32B級の4bit量子化モデルを候補にします。

東芝の疑似量子コンピューター系の考え方は、次回以降に **aruaru-llm Quantum-Inspired Optimizer** として、BUG修正順序・テスト順序・LLM選択・プロンプト圧縮設定の最適化から実装予定です。

## GT730付きPCで毎回行うBUGチェック

```powershell
cd F:\aruaru\aruaru-readme-auto-rs

cargo fmt --all
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

禁止仕様チェック:

```powershell
Select-String -Path .\src\*.rs,.\Cargo.toml,.\README.md -Pattern "tauri","REST API","rest api" -CaseSensitive:$false
```

`README.md` を置いて生成確認:

```powershell
New-Item -ItemType Directory -Force .\tmp-web
Set-Content .\tmp-web\README.md "# TEST`n`n- item 1`n- item 2" -Encoding UTF8
cargo run -- --root .\tmp-web --listen 127.0.0.1:7878 --output both
```

別のPowerShellで確認:

```powershell
Test-Path .\tmp-web\README.rs
Test-Path .\tmp-web\README.html
Get-Content .\tmp-web\README.rs -TotalCount 20
```

## 手動表示チェック

ブラウザで `README.html` を開いて、開発者ツールのレスポンシブ表示で確認します。

- 360x640 スマホ縦
- 640x360 スマホ横
- 768x1024 タブレット縦
- 1024x768 タブレット横
- 1280x720
- 1280x800
- 1280x960
- 1366x768
- 1920x1080
- 2560x1440 WQHD
- 3840x2160 4K
- 7680x4320 8K
- 15360x8640 16K想定

## 注意

`README.rs` は自動生成物です。直接編集せず、元の `README.md` を編集して再アップロードしてください。

## GT730環境での毎回BUGチェック

短縮版:

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

完全版:

```powershell
.\scripts\check-full.ps1
```

詳しい手順は `BUG_CHECKLIST_GT730.md` を参照してください。


## aruaru-llm Toshiba SBM-Inspired Quantum Optimization Engine v7

v7では、東芝の Simulated Bifurcation Machine / SQBM+ の公開情報を参考にした、aruaru-llm用の量子インスパイアード最適化エンジンを追加しました。

この機能は、巨大LLMそのものを普通のPCへ完全移植するものではありません。BUG修正順序、テスト順序、LLM選択、プロンプト圧縮、README生成計画などをQUBO風の組合せ最適化問題として扱い、普通のPCでも良い選択を探すための実験機能です。

追加ファイル:

```text
src/quasi_quantum.rs
ARUARU_LLM_TOSHIBA_SBM_ENGINE.md
ARUARU_LLM_NEXT_HYBRID_THEORY.md
RUN_CHECK_COMMANDS_WINDOWS.md
```

重要な注意:

- 東芝SQBM+そのものを実装したとは主張しません。
- 量子コンピューターを実装したとは主張しません。
- 「富士通量子コンピューターの100倍」と断定しません。
- 公開情報上の「約100倍」は、第三世代SBMが第二世代SBMと比較して高速化したという文脈で扱います。
- aruaruでは、普通のPCでBUGチェック・LLM選択・文脈圧縮・修正順序最適化に応用します。

次回バージョンでは、DeepSeek Folding Engine と Toshiba SBM-Inspired Engine をハイブリッド化し、aruaru-llm Hybrid Folding Quantum-Inspired Planner として全体を見直します。

## aruaru-desktop / aruaru-web 標準連携: 手動スクリプト作成

aruaru-desktop 内の aruaru-web から、ブラウザ経由で aruaru-ai を利用し、自動BUGチェックと手動BUGチェックを両方扱えるようにします。

標準機能:

- aruaru-ai による自動BUGチェック
- aruaru-llm / iLumi.llm 連動
- ローカルLLMと補助ツールの自動ダウンロード計画
- Windows PowerShell用の手動BUGチェックスクリプト作成
- Bash用の手動BUGチェックスクリプト作成
- 使い方説明書の同時出力
- 実行ログ保存
- AIによるBUG要約
- ユーザー承認後の修正支援

手動実行例:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-bugcheck-template.ps1
```

詳細は以下を参照してください。

- `ARUARU_WEB_DESKTOP_SCRIPT_GENERATOR.md`
- `scripts/manual/README_USAGE.md`


## aruaru-ai API KEY Handoff / Import Wizard v9

aruaru-ai には、Claude Opus、OpenAI / ChatGPT、Gemini、DeepSeek などの有料API型LLMを使うための API KEY 引き継ぎ・移行・安全保存機能を標準機能として追加します。

ローカルLLMは既存ディレクトリを指定できます。API型LLMはモデル本体をダウンロードせず、API KEYを安全に登録して利用します。

重要なルール:

- 本人または同じ組織・同じプロジェクトで利用権限があるAPI KEYだけを取り込む
- 他人のキーや別アプリの秘密情報を勝手に探して取り出さない
- API KEYをREADME.md / README.html / README.rs / Git / ログに書かない
- 画面では常にマスク表示する
- `.env`、秘密鍵、`.git`、`target`、`node_modules` はBUGチェック送信対象から除外する
- 有料API利用前に料金発生の確認を出す
- 自動修正はユーザー承認後のみ行う

追加ファイル:

```text
src/api_key_handoff.rs
ARUARU_AI_API_KEY_HANDOFF.md
scripts/manual/API_KEY_HANDOFF_USAGE.md
scripts/manual/aruaru-api-key-import-template.ps1
```

手動でAPI KEYを一時設定する例:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manualruaru-api-key-import-template.ps1 -Provider anthropic -Scope process
```

OpenAI用:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manualruaru-api-key-import-template.ps1 -Provider openai -Scope process
```

詳しくは `ARUARU_AI_API_KEY_HANDOFF.md` と `scripts/manual/API_KEY_HANDOFF_USAGE.md` を参照してください。

## aruaru-llm Hybrid Folding + SBM-Inspired Planner

aruaru-llm には、DeepSeek系の公開技術を再調査した **DeepSeek Folding Engine** と、東芝のSimulated Bifurcation Machine / SQBM+系の公開情報を再調査した **Toshiba SBM-Inspired Optimizer** を統合する実験機能を追加しています。

### 1. DeepSeek Folding Engine

DeepSeek-V3の公開情報では、671B総パラメータのMoEモデルでありながら、各トークンで37Bだけを活性化する設計が説明されています。また、MLA、DeepSeekMoE、auxiliary-loss-free load balancing、multi-token predictionなどの効率化が説明されています。

aruaru-llmでは、この考え方をそのまま巨大モデル再現として扱うのではなく、次のように低コスト開発へ応用します。

- 長いcargoログ、PowerShellログ、README差分、RustソースをそのままLLMへ送らず、重要行へ折り畳む。
- BUG原因、関連ファイル、差分、再発防止テストへ分割する。
- 小型蒸留モデル、量子化モデル、ローカルLLM、API型LLMを用途別に切り替える。
- GT730などの低性能GPUでは、1.5B〜7B級、CPU/Hybrid、短いcontextを優先する。
- 24GB級GPUでは、14B〜32B級の4bitモデルを候補にする。

### 2. Toshiba SBM-Inspired Optimizer

東芝のSQBM+は、Simulated Bifurcation Algorithmを使った量子インスパイアード最適化ソリューションとして説明されています。第三世代SBアルゴリズムでは、edge of chaosを活用して、第二世代SBMと比較してTTSで約10〜100倍高速化したと説明されています。

aruaru-llmでは、これを「1枚GPUで量子コンピューターを完全代替する魔法」とは扱いません。代わりに、以下のような開発支援の最適化へ使います。

- BUG修正順序の最適化
- cargo fmt/check/test/clippy の実行順序最適化
- ローカルLLM / API型LLM / 手動チェックの選択最適化
- プロンプト断片のkeep/drop最適化
- README.md / README.rs / README.html 生成順序の最適化

### 3. Hybrid Folding + SBM Fusion

追加ファイル:

```text
src/hybrid_theory.rs
```

この機能は、DeepSeek Foldingで情報量を圧縮し、Toshiba SBM-Inspired Optimizerで「何を先に実行するか」を決めます。

処理の基本方針:

```text
1. 安いチェックを先に行う
2. 長いログを折り畳む
3. 小型ローカルLLMで分類する
4. 難しい箇所だけ有料API型LLMへ送る
5. 差分を表示する
6. ユーザー承認後のみ修正する
7. cargo fmt/check/test/clippyで再確認する
```

### 4. 重要な注意

このプロジェクトは、以下を主張しません。

- GT730または1枚GPUで、DeepSeek-V3 671B相当を完全同品質で実行できる。
- DeepSeekの非公開スーパーコンピューター環境を再現している。
- 東芝SQBM+そのものを実装している。
- 富士通の量子コンピューターを100倍上回ることを実証している。
- 1枚GPUの普通のPCが、10万GPU級設備や量子コンピューターを完全代替できる。

aruaru-llmは、公開情報から学べる良い所を、低コストの自動BUGチェック、README生成、手動スクリプト生成、AI開発支援へ応用する実験的な開発支援システムです。

## 無報酬ボランティア / GT730環境での開発背景

aruaruは、無職・無報酬のボランティア状態でも、低コストでアプリ開発と品質ゲートを続けられるように設計しています。

開発者は、GT730級のPCしか使えない厳しい環境でも、生成AI代、ドメイン更新料、VPSレンタルサーバー代などを自腹で負担しながら、aruaru-desktop、aruaru-web、aruaru-ai、aruaru-llm、README自動生成、自動BUGチェック機能を開発しています。

このため、aruaruでは以下を重視します。

- 高額APIを使う前に、ローカルで安いBUGチェックを実行する。
- 既にダウンロード済みのローカルLLMは再ダウンロードしない。
- GT730でも動く小さいチェックから開始する。
- 有料APIは本当に必要な難問だけに使う。
- APIキー、.env、SSHキー、秘密情報は絶対にログやREADMEへ出さない。
- 自動修正はユーザー承認後のみ行う。

## Hybrid planner tests

追加確認コマンド:

```powershell
cargo test hybrid_theory
cargo test llm_folding
cargo test quasi_quantum
cargo check
cargo clippy --all-targets -- -D warnings
```


## aruaru-desktop / aruaru-ai 標準機能: 手動COPYスクリプト自動生成

自動BUGチェックを前提にしながら、aruaru-ai はユーザーが手動で実行できるCOPYスクリプトも自動生成します。

標準生成物:

```text
scripts/manual/aruaru-manual-copy-template.ps1
scripts/manual/aruaru-manual-copy-template.sh
scripts/manual/COPY_SCRIPT_USAGE.md
src/copy_script_generator.rs
```

用途:

- 修正済みファイルをバックアップ先へ安全にコピーする
- VPSや別フォルダへアップロードする前にdry-runで確認する
- GT730級PCなど低スペック環境でも、GUIに頼らず手動で作業できるようにする
- aruaru-aiが作った修正差分を、ユーザーが内容確認後にコピーできるようにする

安全仕様:

- 標準はdry-run
- `.env`、秘密鍵、APIキー、SSH秘密鍵はコピー対象外
- `.git`、`target`、`node_modules` などはコピー対象外
- 削除操作はしない
- 実行前に内容表示とユーザー承認を行う

PowerShell例:

```powershell
cd F:ruaruruaru-rs4
powershell -ExecutionPolicy Bypass -File .\scripts\manualruaru-manual-copy-template.ps1 -SourceRoot "F:ruaruruaru-rs4" -DestinationRoot "F:ruaruackupruaru-rs4" -DryRun $true
```

実際にコピーする場合:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manualruaru-manual-copy-template.ps1 -SourceRoot "F:ruaruruaru-rs4" -DestinationRoot "F:ruaruackupruaru-rs4" -DryRun $false
```


## v12 Manual COPY Script Parser Fix

手動COPYスクリプト生成機能のPowerShellテンプレートを修正しました。PowerShellでは `param(...)` が最初の実行文である必要があるため、`$ErrorActionPreference` を `param(...)` の後へ移動しました。これにより、`代入式が無効です` で停止するBUGを防止します。

実行例:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-copy-template.ps1 -SourceRoot "F:\aruaru\aruaru-rs4" -DestinationRoot "F:\aruaru\backup\aruaru-rs4" -DryRun $true
```

### v13 manual COPY script boolean parser fix

The manual COPY script now treats `DryRun`, `IncludeGeneratedReadme`, and `IncludeLogs` as normalized boolean-like values. This avoids Windows PowerShell failures when `powershell -File ... -DryRun $true` arrives as a string.

Recommended:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-copy-template.ps1 -SourceRoot "F:\aruaru\aruaru-rs4" -DestinationRoot "F:\aruaru\backup\aruaru-rs4" -DryRun true
```
