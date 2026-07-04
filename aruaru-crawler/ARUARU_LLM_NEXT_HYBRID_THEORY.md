# 次回バージョン予定: DeepSeek Folding x Toshiba SBM Hybrid Theory

## 目的

次回バージョンでは、DeepSeek系の折り畳み新理論と、東芝SBM系の量子インスパイアード最適化新理論を融合する。

## DeepSeek側の担当

- LLM文脈を圧縮する
- 長いcargoログを重要エラーへ折り畳む
- MoE風に専門家へ振り分ける
- 小型蒸留モデル、量子化モデル、API fallbackを使い分ける
- README / Poem / Rust / GraphQL / PowerShellなどの修正候補を作る

## Toshiba SBM側の担当

- 修正候補の適用順序を最適化する
- テスト順序を最適化する
- どのLLMを使うかを最適化する
- どの文脈を残すかをQUBO風に最適化する
- 速度重視 bSB風、精度重視 dSB風、局所最適脱出 Edge-of-Chaos風を使い分ける

## 融合後の処理フロー

```text
1. cargo / PowerShell / README / GraphQL のログを収集
2. DeepSeek Folding でログとファイルを重要断片へ圧縮
3. 修正候補、テスト候補、LLM候補を生成
4. Toshiba SBM-Inspired Engine がQUBO風に選択・順序を最適化
5. AIが修正案を出す
6. diff表示
7. ユーザー承認
8. 修正適用
9. 再BUGチェック
```

## 重要な表記

この融合理論は aruaru 独自の研究実装であり、DeepSeek本体や東芝SQBM+そのものではない。

性能目標は「1枚GPUで巨大モデルを完全再現」ではなく、普通のPCでBUGチェックと開発支援の無駄を削減することである。
