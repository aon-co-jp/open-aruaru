# aruaru-ai README output checkbox menu

## 問題点

アプリやWEBサイト開発では `README.md` は必須だが、実行環境ごとに `README.rs`、`README.html`、`README.php`、その他言語別READMEを必要とする場合がある。
以前の設計では `README.rs` / `README.html` が固定寄りで、aruaru-ai メニューから任意選択する標準UIが不足していた。

## 原因

README生成機能が「変換エンジン」中心で、aruaru-ai の開発メニューにあるチェックボックス選択モデルとして整理されていなかった。

## 修正方針

`README.md` は常に正本として固定する。追加の `README.*` はチェックボックスで、選択なし・1つ・複数選択のすべてを許可する。

## 標準チェックボックス

- README.rs / Rust
- README.html / HTML5 + CSS3 + TypeScript
- README.php / PHP
- README.py / Python
- README.ts / TypeScript
- README.js / JavaScript
- README.go / Go
- README.java / Java
- README.cs / C#
- README.kt / Kotlin
- README.swift / Swift
- README.rb / Ruby
- README.json / JSON metadata

## CLI対応

従来互換:

```powershell
cargo run -- --root .\tmp-web --output both
```

チェックボックス相当:

```powershell
cargo run -- --root .\tmp-web --output none --extra-outputs "rs,html,php,python,ts,js,go,java,csharp,kotlin,swift,ruby,json"
```

選択なし:

```powershell
cargo run -- --root .\tmp-web --output none --extra-outputs none
```

## 安全ルール

- `README.md` が正本。
- `README.*` は生成物なので手書きしない。
- APIキー、`.env`、秘密鍵、SSHキーは生成物に入れない。
- HTMLはサニタイズ済みの内容だけを出力する。
- PHP/TypeScript/JavaScript等の生成物に外部通信処理を勝手に入れない。
