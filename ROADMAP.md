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
