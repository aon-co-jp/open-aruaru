# aruaru-readme-auto-rs ROADMAP

## v0.1 現在

- README.mdを監視
- README.rsを自動生成
- README.htmlを任意で同時生成
- HTML5 + CSS3レスポンシブ表示
- 日本語README対応
- script除去による安全化
- Poemの簡易ダッシュボード

## v0.2 追加候補

- README.md更新履歴の保存
- README.html生成前後の差分表示
- 複数ドメイン/複数ドキュメントルート対応
- 生成失敗時のエラーHTML出力
- Windowsサービス化スクリプト
- systemdインストールスクリプト改善

## v0.3 aruaru-ai連携

- aruaru-aiがREADME.mdを自動改善
- README_SOURCEからREADME.md/README.html/README.rsを同時生成
- BUGチェック結果をREADMEへ追記する提案
- Redmine互換レポート出力

## v0.4 品質ゲート強化

- Markdown構造チェック
- HTML5検証
- リンク切れチェック
- 画像ファイル存在チェック
- 目次自動生成
- Mermaidなどの安全な図表対応

## 守る方針

- Rust + Poem中心
- Tauriなし
- REST APIなし
- FTPアップロードされたREADME.mdを正本にする
- 生成物は原子的に書き込む
- 途中書き込みREADME.mdを読んで壊れた生成物を出さない


## v17 development menu roadmap

- Display all development menu categories in aruaru-ai.
- Allow none, one, or many checkbox selections per category.
- Generate project plans, README.md, README.* outputs, BUG checks, and package scripts from the selected menu.
- Keep GraphQL/WunderGraph Cosmo as the preferred internal API direction.


## v18 - aruaru-ai Programming Language Information Crawler

Added language-by-language feature / merit / demerit / handoff guidance for the aruaru-ai development menu.
README.md remains fixed, and programming language choices now show daily-refreshable guidance with source metadata.

Ruby policy: Ruby is recognized as a Japanese-created language with many Japanese resources, but aruaru-ai does not recommend Ruby by default for new multi-developer projects because handoff can fail when implicit style and original-developer intent dominate the codebase.

New files:

- src/programming_language_info.rs
- ARUARU_AI_PROGRAMMING_LANGUAGE_INFO.md

Quality gates:

- cargo test programming_language_info
- cargo check
- cargo clippy --all-targets -- -D warnings


## v19 Daily Programming Language Update Job

v19 adds `src/language_update_job.rs` and `ARUARU_AI_DAILY_LANGUAGE_UPDATE_JOB.md`.

The aruaru-ai programming language menu now has a daily update-job design:

- crawl allowlisted public language sources once per day
- separate web evidence from aruaru owner policy
- summarize features, merits, demerits, and handoff risks
- show confidence and stale warnings
- keep `latest.json`, daily history, and diff markdown
- keep Ruby visible for existing Ruby/Rails maintenance, but mark it `NotRecommendedByDefault` for new standard development


## v20 TOP100 Programming Languages

Added `src/language_top100.rs` and `ARUARU_AI_TOP100_PROGRAMMING_LANGUAGES.md`. aruaru-ai now has a daily-updated TOP100 programming language report with features, merits, demerits, author/governance, author policy/claim, and aruaru recommendation. Popularity signals are separated from technical recommendation. Ruby remains visible but NotRecommendedByDefault for new standard development.
