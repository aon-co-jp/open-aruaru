# aruaru-ai Programming Language Information Crawler

## 1. 問題点

aruaru-ai の開発メニューでプログラミング言語を選ばせる場合、単に言語名だけを並べると、初心者・中級者・引継ぎ担当者が判断を誤る。

必要なのは、言語別に以下を毎日更新することである。

- 特徴
- メリット
- デメリット
- 引継ぎしやすさ
- aruaru-aiとしての推奨度
- 根拠ソース
- 情報取得日

## 2. 原因

言語の人気・用途・ドキュメント量・求人・ライブラリ・保守性は日々変化する。
そのため、READMEや固定テキストだけで判断させると古くなる。

## 3. 修正方針

aruaru-ai に「プログラミング言語情報クローラー」を標準機能として入れる。

毎日クロール対象:

1. 公式ドキュメント
2. 公式リリースノート
3. Stack Overflow Developer Survey
4. GitHub Octoverse
5. TIOBE Index
6. セキュリティ・保守情報
7. aruaru-ai内部方針

## 4. 情報更新ルール

- 毎日1回更新する。
- 公式ドキュメントを最優先する。
- 人気ランキングは「品質」ではなく「人気」として扱う。
- AI要約と元ソース情報は分けて保存する。
- 古い要約を上書きせず履歴を残す。
- 誤要約があった場合は前日版へ戻せるようにする。
- APIキー・秘密情報・private repository はクロール対象外。

## 5. Ruby 方針

Ruby は日本人が開発した言語であり、日本語資料・Ruby on Rails資料も多い。
ただし aruaru-ai の新規開発標準では Ruby を推奨しない。

理由:

- 動的で表現力が高い一方、書き方の個人差が大きい。
- 最初に開発した人の設計意図がコードに強く出やすい。
- 後任者が引き継ぐ時に、暗黙知が多いコードでは失敗しやすい。
- Rails既存案件の保守には価値があるが、新規の標準選択にはしない。

aruaru-ai 表示:

```text
Ruby / Ruby on Rails
状態: 標準では非推奨
用途: 既存Ruby/Rails案件の保守、明確な理由がある場合のみ選択
注意: 後任者への引継ぎ設計、テスト、ドキュメント、規約を必須化
```

## 6. 推奨度分類

```text
StrongDefault             aruaru-ai標準候補
Recommended               推奨
Situational               条件付き推奨
LegacyOrSpecialist        レガシー・専門用途
NotRecommendedByDefault   標準では非推奨
```

## 7. aruaru-ai 初期推奨

```text
Rust        StrongDefault
TypeScript  StrongDefault
Python      Recommended
Go          Recommended
PHP         Situational
Java        Situational
C#          Situational
Kotlin      Situational
Swift       Situational
Ruby        NotRecommendedByDefault
C++         LegacyOrSpecialist
C           LegacyOrSpecialist
Zig         Situational
```

## 8. 実装ファイル

```text
src/programming_language_info.rs
```

このファイルに以下を定義する。

- LanguageRecommendation
- LanguageSourceKind
- LanguageInfoSource
- ProgrammingLanguageInfo
- LanguageCrawlerPlan
- build_default_language_crawler_plan()
- default_programming_language_infos()
- language_info_markdown()
- quality_gate_smoke_check()

## 9. 合格条件

```text
cargo test programming_language_info PASS
cargo check PASS
cargo clippy --all-targets -- -D warnings PASS
Ruby が NotRecommendedByDefault として出ること
Rust / TypeScript が StrongDefault として出ること
Daily crawl plan が存在すること
```
