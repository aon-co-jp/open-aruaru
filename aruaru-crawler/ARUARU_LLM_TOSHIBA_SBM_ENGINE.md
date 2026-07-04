# aruaru-llm Toshiba SBM-Inspired Quantum Optimization Engine v7

## 目的

aruaru-llm に、東芝の量子インスパイアード最適化技術である Simulated Bifurcation Machine / SQBM+ の公開情報を参考にした、実験的な最適化エンジンを追加する。

この機能は、巨大LLMを1枚GPUへ魔法のように完全搭載するものではない。aruaru の開発・BUGチェック・LLM選択・README生成・プロンプト圧縮を、QUBO風の組合せ最適化問題へ変換して、普通のPCでも「良い順番」「良い選択」「良い圧縮」を探すためのエンジンである。

## 調査メモ

公開情報では、東芝 SQBM+ は SBアルゴリズムを用いた量子インスパイアード最適化ソリューションであり、組合せ最適化問題を短時間で解くことを目的としている。

東芝の公式説明では、SQBM+ は Simulated Bifurcation Machine を核とする量子インスパイアード最適化ソリューションで、金融ポートフォリオ、配送経路、送電網、創薬などの組合せ最適化を対象としている。

また、東芝の技術説明では、SBアルゴリズムは逐次更新型のシミュレーテッドアニーリングと異なり、微分方程式を計算的に解く構造により並列更新しやすく、GPU/FPGAなどの並列計算と相性がよいと説明されている。

東芝の2026年発表では、第三世代SBアルゴリズムが「edge of chaos」を活用し、第二世代SBMと比較してTTSが約10〜100倍高速化したと説明されている。

重要: これは「富士通の量子コンピューターの100倍」と断定する公開根拠ではない。aruaru README では、100倍の対象を「第二世代SBMとの比較」と正確に書く。

## aruaru での実装方針

### 1. QUBO化する対象

- BUG修正順序
- テスト実行順序
- LLMモデル選択
- プロンプト圧縮
- README.md / README.rs / README.html 生成計画
- GraphQL / Poem / Rust の修正候補選択

### 2. 実装するアルゴリズムモード

- bSB風: 早く良解を探すモード
- dSB風: 精度重視モード
- Edge-of-Chaos風: 局所最適から抜けるために変数ごとの圧力と摂動を使うモード

### 3. ハードウェア別方針

#### GT730 / 小型GPU

- GPU推論や巨大QUBOは狙わない
- 小型QUBOへ分割
- CPU安全モードで実行
- BUG順序・テスト順序・LLM選択など軽量最適化に使う

#### RTX 3090 / 4090 24GB級

- 1枚GPU向けの並列shotバッチ化を将来実装
- 大きめのQUBOブロックを扱う
- DeepSeek Folding Engine と組み合わせて、文脈圧縮・モデル選択・修正順序を最適化

#### 外部SQBM互換

- 将来、東芝 SQBM+ のような外部ソルバーを使う場合は、aruaru内部ではQUBOをエクスポートし、結果だけインポートする
- ユーザー承認なしに修正適用しない

## 追加ファイル

- `src/quasi_quantum.rs`

主な関数:

```rust
build_toshiba_sbm_plan(input: &ToshibaSbmPlanInput) -> ToshibaSbmPlan
solve_qubo_local(problem: &QuboProblem, config: &LocalSbmConfig) -> Result<LocalSbmSolution, String>
```

## 安全表記

この機能は以下を主張しない。

- 東芝SQBM+そのものを実装したとは主張しない
- 量子コンピューターを実装したとは主張しない
- 富士通量子コンピューターの100倍と断定しない
- 1枚GPUで10万GPU級スーパーコンピューターを完全代替できるとは主張しない

この機能が目指すのは、普通のPCで aruaru の開発支援タスクをより賢く並べ替え、圧縮し、選択すること。

## 次回バージョンの融合予定

次回は以下を融合する。

- DeepSeek Folding Engine: モデル・文脈・専門家・修正計画を折り畳む
- Toshiba SBM-Inspired Engine: 選択・順序・圧縮・探索をQUBO風に最適化する

融合後の名前候補:

```text
aruaru-llm Hybrid Folding Quantum-Inspired Planner
```

## 参考公開情報

- Toshiba SQBM+ official introduction
- Toshiba SBM technologies page
- Toshiba 2026 edge-of-chaos SBM announcement
- QUBO solver benchmark papers including Toshiba SBM and Fujitsu Digital Annealer
