# aruaru-llm DeepSeek Folding Engine

## 目的

aruaru-llm に、DeepSeek 系の公開技術を調査・分析したうえで、普通のPCや1枚GPU環境でも使いやすくするための **DeepSeek Folding Engine** を追加します。

この機能は、巨大スーパーコンピューター上のモデルをそのまま1枚GPUで完全再現するものではありません。DeepSeek-V3 / R1 / NSA で公開されている考え方を、aruaru の開発支援・BUGチェック・README生成・長文ログ解析に使えるよう、圧縮・凝縮・折り畳み方式として実装する研究機能です。

## 調査したDeepSeek系の重要技術

### 1. MoE / DeepSeekMoE

DeepSeek-V3 は総パラメータ671B級のMoEモデルで、各トークンでは一部の専門家だけを活性化する設計です。aruaru-llm ではこれを **expert folding** として扱います。

aruaruでの置き換え:

- コンパイルエラー専門家
- PowerShellエラー専門家
- README生成専門家
- GraphQL / Poem専門家
- 禁止仕様チェック専門家
- 日本語説明専門家

1つの巨大AIに全部やらせるのではなく、タスクごとに小さな専門処理へ分けて、必要な部分だけLLMに渡します。

### 2. MLA / Multi-head Latent Attention

DeepSeek-V3 で使われる MLA は、注意機構とKVキャッシュを効率化する方向の技術です。aruaru-llm ではこれを **KV folding / context folding** として実装します。

aruaruでの置き換え:

- 長いcargoログを全部LLMに投げない
- ERROR行、直前直後の文脈、関連ファイル名だけを抽出
- 古いログは要約して保持
- 重要なソース断片だけ再読み込みする

### 3. MTP / Multi-Token Prediction

DeepSeek-V3 は multi-token prediction objective を採用しています。aruaru-llm では、BUG修正時に1ステップずつではなく、以下をまとめて予測する **repair folding** として扱います。

- 原因
- 修正対象ファイル
- 修正案
- 再発防止テスト
- 再チェックコマンド

### 4. DeepSeek-R1 / reasoning distillation

DeepSeek-R1 では、大型推論モデルの能力を小型の蒸留モデルへ移す考え方が示されています。aruaru-llm では、1枚GPUやGT730環境では小型蒸留モデルを優先します。

推奨:

| 環境 | 推奨モデル階層 | 役割 |
|---|---:|---|
| GT730 / VRAM 2GB級 | 1.5B〜7B、CPU/Hybrid | ログ要約、エラー分類 |
| RTX 3060 12GB | 7B〜14B 4bit | BUG修正案、README生成 |
| RTX 3090 / 4090 24GB | 14B〜32B 4bit | コードレビュー、複雑な修正案 |
| 48GB以上 | 32B〜70B 4bit/8bit | 長文解析、重い推論 |

### 5. NSA / Native Sparse Attention

DeepSeek系のNSAは、長文をすべて密に見るのではなく、階層的なトークン圧縮、ブロック選択、局所窓を組み合わせる方向です。aruaru-llm では **Long Context Folding** として扱います。

aruaruでの置き換え:

- cargoログ全体を粗く圧縮
- ERROR / warning / failed / panicked を細かく選択
- 直近の差分は局所窓として密に見る
- READMEや設計書は章単位で圧縮

## 実装済みファイル

```text
src/llm_folding.rs
```

このファイルには、以下の実験的なプランナーを実装しています。

```rust
build_deepseek_folding_plan(input: &FoldingInput) -> FoldingPlan
```

主な役割:

- GPU名とVRAMからモデル階層を選ぶ
- GT730ではCPU/Hybridに切り替える
- 12GB / 24GB / 48GB級GPUに応じて推奨モデルを変える
- BUGチェック、コードレビュー、README生成、長文調査で処理戦略を変える
- 自動修正ではなく、まず修正案と再チェック計画を作る

## 折り畳み新理論としてのaruaru解釈

aruaru-llm での「折り畳み新理論」は、次の5つです。

```text
1. Model Folding
   大型モデルの能力を、小型蒸留モデル・量子化モデル・APIフォールバックへ分解する。

2. Expert Folding
   1つの巨大AIではなく、BUG、README、PowerShell、Poem、GraphQLなどの専門処理へ分ける。

3. Context Folding
   長文ログやREADMEを、重要行・要約・再読込可能ファイル参照へ折り畳む。

4. KV Folding
   重い文脈を毎回LLMへ渡さず、要約・ハッシュ・重要断片として保持する。

5. Repair Folding
   BUG原因、修正案、差分、再発防止テスト、再チェックを1つの修正計画にまとめる。
```

## 注意

この機能は、DeepSeekの公開論文・公開モデル・公開実装から得られる考え方を aruaru 向けに実装するものです。

以下は主張しません。

```text
- 1枚GPUでDeepSeek-V3 671Bそのものを同品質で完全実行できる
- 非公開のDeepSeek内部学習システムを再現している
- 10万GPU級スーパーコンピューターを普通のPCで完全代替できる
```

代わりに、以下を目標にします。

```text
- 1枚GPUやGT730環境でも、BUGチェックに必要な情報量へ圧縮する
- 小型LLMでも原因分類と修正候補を出せるようにする
- 足りない場合だけAPI型LLMへフォールバックする
- ユーザー承認なしに自動修正しない
```

## 次回以降

東芝の疑似量子コンピューター系の考え方は、次回バージョンで以下として検討します。

```text
aruaru-llm Quantum-Inspired Optimizer
- 組合せ最適化
- モデル選択最適化
- BUG修正順序最適化
- テスト順序最適化
- プロンプト圧縮パラメータ探索
```
