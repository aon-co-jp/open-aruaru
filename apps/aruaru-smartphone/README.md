# aruaru-android — インストーラー付き Android アプリ

> **第二の KUSANAGI / 第二の Cursor** のスマホ面。desktop / web と同一引き継ぎ・AI 選択を共有。

## 成果物

| ファイル | 説明 |
|----------|------|
| `dist/aruaru-android_0.1.0_universal.apk` | 端末へ直接インストール（サイドロード） |
| `dist/aruaru-android_0.1.0.aab` | Google Play 用（ビルド時） |
| `dist/install-android.ps1` | PC から adb インストール |
| `dist/INSTALL.md` | インストール手順 |

## ビルド（Windows + Android SDK）

```powershell
cd F:/aruaru/apps/aruaru-smartphone

# 前提（Android Studio 推奨）
$env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"

# インストーラー付きビルド
.\installer\build-installer.ps1
```

初回のみ `tauri android init` はビルドスクリプトが自動実行します。

## スマホへインストール

**方法 A — APK を転送**
1. `F:/aruaru/apps/aruaru-smartphone/dist/aruaru-android_0.1.0_universal.apk` をスマホへコピー
2. ファイルマネージャーからインストール（提供元不明を許可）

**方法 B — adb（USB デバッグ）**

```powershell
cd F:/aruaru/apps/aruaru-smartphone/dist
.\install-android.ps1
```

## 初回設定

1. アプリを起動
2. **設定** タブ → VPS の aruaru-web URL（例: `http://160.251.237.162:3000`）
3. **状態** でダッシュボード、**引き継ぎ** で Cursor/Claude 連携

## 開発

```powershell
cd F:/aruaru/apps/aruaru-smartphone
npm install
npm run android:dev    # エミュレータ / 実機
```

## 機能

| タブ | 内容 |
|------|------|
| 状態 | サーバー CPU/メモリ/SSL |
| 引き継ぎ | Markdown エクスポート / インポート |
| AI | 用途別 LLM おすすめ |
| 設定 | VPS URL |

## スタック

Tauri Mobile 2 + Rust + TypeScript + Poem API（VPS）

## パス

- プロジェクト: `F:/aruaru/apps/aruaru-smartphone/`
- 詳細引き継ぎ: `F:/aruaru/docs/HANDOFF_2026-06-24.md`
