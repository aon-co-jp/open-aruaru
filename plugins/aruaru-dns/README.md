# aruaru-dns

DNS・ネームサーバー設定ガイドとレコード管理。

## デスクトップ（Tauri 2）

```powershell
cd F:\aruaru\apps\aruaru-dns
npm install
npm run tauri dev
```

## ダブル DATABASE

| DB | 役割 | 既定接続 |
|----|------|----------|
| **PostgreSQL** | 正（マスター） | `postgres://aruaru:aruaru@127.0.0.1:5432/aruaru` |
| **aruaru-db** | 同期・バージョン管理 | `postgres://root@127.0.0.1:5433/aruaru_dns` |

aruaru-db ソース: `F:\aruaru\aruaru-db`  
サーバー起動例:

```powershell
cd F:\aruaru\aruaru-db
cargo run -p aruaru-server -- --pg-port 5433 --data ./data/dns
```

## 設定

ユーザー設定（上書き）:

`%APPDATA%\aruaru-dns\config.toml`

組み込み既定: `apps/aruaru-dns/src-tauri/config/default.toml`

## Poem API（aruaru コア）

| Method | Path |
|--------|------|
| GET | `/api/dns/records` |
| POST | `/api/dns/records` |
| POST | `/api/dns/records/delete` |
| GET | `/api/dns/guide?zone=&vps_ip=` |

## インストール（プラグイン）

```powershell
# aruaru サーバーから
curl -X POST http://localhost:3000/api/install -H "Content-Type: application/json" -d "{\"target\":\"aruaru-dns\"}"
```
