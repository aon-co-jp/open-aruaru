# open-aruaru

open-aruaru は、Aruaru 系プロジェクト全体を整理・管理するための統合リポジトリです。

## Main Projects

- aruaru-db
- open-runo
- open-web-server
- aruaru_i
- open-cuda

## Policy

各プロジェクト本体は、それぞれ独立した GitHub リポジトリとして管理します。
この open-aruaru リポジトリは、全体設計、ドキュメント、開発方針、品質ゲート、運用ルールを管理します。

## Development Quality Gates

- 小さな変更単位で commit する
- push 前に git status を確認する
- build / test / format / lint を確認する
- Rust プロジェクトでは cargo check を優先する
- 生成物 target/ は Git 管理しない
