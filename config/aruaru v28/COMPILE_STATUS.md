# COMPILE_STATUS v28

## 実行環境確認

このChatGPT実行コンテナでは、現時点で `rustc` / `cargo` は見つかりませんでした。

```text
rustc: command not found
cargo: command not found
```

そのため、この環境での `cargo check --workspace` / `cargo test --workspace` は未実行です。

## 実施済み

```text
validate-package OK for v28
v28 packaging verification OK
```

## ローカルPCで行う確認

```bash
cargo check --workspace
cargo test --workspace
```

## v28で追加したBUG低減策

- ドメイン、GitHub URL、HTTP/HTTPS URLの入力検証
- 許可済みローカルドライブだけを調査する drive-indexer
- 5分単位の自動更新 loop
- オフラインキャッシュの保持期限: 3日
- `audiocafe.tokyo` / `*.audiocafe.tokyo` 由来DATAは保持
- 期限切れ削除時に、キャッシュroot外のファイルを削除しない安全確認
- Rust unit test追加
- Tauri関連ファイル・依存の禁止検証

## 注意

YouTubeについては、動画ファイルそのものの保存ではなく、オンライン時に閲覧用ページ/リンク/メタ情報相当のキャッシュを保存する設計です。著作権や各サービス規約に反する動画ファイル保存は実装対象外です。
