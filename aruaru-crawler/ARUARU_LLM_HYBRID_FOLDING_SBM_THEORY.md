# aruaru-llm Hybrid Folding + SBM-Inspired Planner

## 目的

この機能は、aruaru-llm に以下の2つの研究方針を統合する実験機能です。

1. DeepSeek公開技術に基づく Folding Engine
   - MoE的な専門処理分離
   - 蒸留モデル活用
   - 量子化
   - 長文ログの折り畳み
   - sparse/context/KV folding
   - API fallback

2. 東芝SBM/SQBM+公開情報に基づく SBM-Inspired Optimizer
   - QUBO風の選択問題化
   - BUG修正順序の最適化
   - テスト順序の最適化
   - LLMモデル選択の最適化
   - 低コスト優先の探索
   - edge-of-chaos風の局所解脱出

## 重要な正確性ルール

この実装は、以下を主張しません。

- GT730または1枚GPUでDeepSeek-V3 671B相当を完全同品質で実行できる、とは主張しません。
- DeepSeekの非公開スーパーコンピューター環境を再現した、とは主張しません。
- 東芝SQBM+そのものを実装した、とは主張しません。
- 富士通の量子コンピューターを100倍上回ることを実証した、とは主張しません。
- 1枚GPUの普通のPCが量子コンピューターや10万GPU級設備を完全代替する、とは主張しません。

公開情報に基づき、aruaruでは「低コストの開発・BUGチェックに使える実用的な折り畳み・最適化」として実装します。

## DeepSeek側の再調査結果

DeepSeek-V3は、公開技術報告で671B総パラメータ、各トークンで37B活性化のMoEモデルとして説明されています。また、MLA、DeepSeekMoE、auxiliary-loss-free load balancing、multi-token predictionなどの効率化が説明されています。

DeepSeek-R1では、大型推論モデルから小型モデルへ推論パターンを蒸留する考え方が公開され、1.5B、7B、8B、14B、32B、70B級の蒸留モデルが公開されています。

Native Sparse Attention系では、長文処理のために粗いトークン圧縮、細かいトークン選択、ハードウェアに合わせた疎な注意機構が説明されています。

aruaru-llmでは、これらを以下へ変換します。

- 巨大モデルを直接動かすのではなく、小型モデル・API・ログ圧縮を組み合わせる。
- BUGログを丸ごとLLMへ送らず、エラー行、関連ファイル、diff、再現手順へ折り畳む。
- GT730環境では1.5B〜7B級またはCPU/Hybridを優先する。
- 24GB級GPUでは14B〜32B級の4bitモデルを候補にする。
- 有料APIは難しい箇所だけに使う。

## 東芝SBM側の再調査結果

東芝のSQBM+は、Simulated Bifurcation Algorithmを用いた量子インスパイアード最適化ソリューションとして説明されています。対象は配送、金融、創薬、送電網などの大規模組合せ最適化です。

東芝の技術説明では、SBMは微分方程式を計算的に解く構造により、並列更新しやすく、従来のコンピューターで高速化しやすいと説明されています。

2026年の東芝発表では、第三世代SBアルゴリズムがedge of chaosを活用し、第二世代SBMと比較してTTSで約10〜100倍高速化したと説明されています。

aruaru-llmでは、これを以下へ変換します。

- BUG修正候補をbinary変数として扱う。
- テスト順序をQUBO風に最適化する。
- LLM選択をコスト・速度・精度・プライバシーの最適化問題として扱う。
- プロンプト断片をkeep/drop変数として扱う。
- GT730では小さいQUBO窓をCPUで安全に解く。

## 新統合・新融合の実装

追加ファイル:

```text
src/hybrid_theory.rs
```

主な関数:

```rust
build_hybrid_folding_sbm_plan(input: &HybridPlannerInput) -> HybridFoldingSbmPlan
```

処理の流れ:

1. 開発PCのGPU、VRAM、RAM、AI予算、API利用可否を受け取る。
2. DeepSeek Foldingでログ・README・コード差分を圧縮する。
3. Toshiba SBM-Inspired Optimizerで、修正順序・テスト順序・LLM選択順序を決める。
4. まず安いチェックを実行する。
5. 失敗箇所だけローカルLLMまたはAPI型LLMへ渡す。
6. 差分を表示する。
7. ユーザー承認後のみ修正する。
8. cargo fmt/check/test/clippyで再検証する。

## GT730 / 無報酬ボランティア前提

この機能は、開発者がGT730級の低性能PCしか使えない場合でも、最低限の自動BUGチェックを続けられるように設計しています。

- まずPowerShell/Cargoの安いチェックを走らせる。
- ログを短く折り畳む。
- ローカル小型LLMで分類する。
- 有料APIは本当に必要な難問だけに使う。
- APIキー、.env、SSHキー、個人情報、target、node_modules、.gitは送信しない。
- 自動修正は必ずユーザー承認後に行う。

## READMEに書く姿勢

READMEには、無職・無報酬ボランティア・自腹運営という厳しい開発背景を、恥ではなく「低コストでも品質を落とさないための設計条件」として明記します。

ただし、寄付や支援をお願いする場合でも、技術的な性能主張は誇張せず、検証済み範囲と研究目標を分けて書きます。
