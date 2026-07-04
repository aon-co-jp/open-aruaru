# aruaru デプロイ手順（Windows → VPS）

## 必要なもの（手元 Windows 側）

| ツール | 用途 | 入手先 |
|--------|------|--------|
| WinSCP または Git Bash | ファイルアップロード | https://winscp.net / Git for Windows |
| TeraTerm または PuTTY | SSH ターミナル | https://teratermproject.github.io |

---

## STEP 1 — VPS に SSH 接続して初期フォルダを作成

TeraTerm / PuTTY で VPS に SSH ログインして、以下を貼り付けて実行：

```bash
mkdir -p /open-aruaru
mkdir -p /open-aruaru-easyweb/{data,logs,static,config}
mkdir -p /open-aruaru-core/{data,logs,static,config}
echo "フォルダ作成完了"
ls -la /home/
```

---

## STEP 2 — ソースコードを VPS へアップロード

### 方法 A：WinSCP（GUI、推奨）

1. WinSCP を起動
2. 接続設定：
   - プロトコル: SFTP
   - ホスト名: `<VPS_IP>`
   - ユーザー名: `root`
   - パスワード: VPS のパスワード（または SSH 鍵）
3. 左ペイン（ローカル）で `F:\aruaru\` を開く
4. 右ペイン（VPS）で `/open-aruaru/` を開く
5. 以下のフォルダ・ファイルをドラッグ＆ドロップでアップロード：

```
アップロード対象（F:\aruaru\ の中から）:
  ✅ Cargo.toml          ← ワークスペースルート
  ✅ Cargo.lock
  ✅ aruaru-easyweb/     ← フォルダごと（target/ は除外）
  ✅ aruaru-core/        ← フォルダごと（target/ は除外）
  ✅ aruaru-ai-core/     ← フォルダごと（target/ は除外）

除外（アップロード不要）:
  ❌ target/             ← ビルド成果物（VPS でビルドするので不要）
  ❌ .git/               ← Git 履歴（不要）
  ❌ *.db                ← ローカルの DB（不要）
```

### 方法 B：Git Bash / WSL から scp コマンド

```bash
# Git Bash または WSL で実行
cd /f/aruaru    # Git Bash の場合
# cd /mnt/f/aruaru  # WSL の場合

scp -r \
  Cargo.toml Cargo.lock \
  aruaru-easyweb \
  aruaru-core \
  aruaru-ai-core \
  root@<VPS_IP>:/open-aruaru/
```

### 方法 C：setup-vps.sh / test.sh もアップロード

```
  ✅ setup-vps.sh
  ✅ deploy.sh
  ✅ test.sh
```
→ `/open-aruaru/` にアップロード

---

## STEP 3 — VPS 上で初期セットアップ（Rust + 依存パッケージ）

SSH ターミナル（TeraTerm）に戻り：

```bash
cd /open-aruaru
bash setup-vps.sh
```

または手元から直接実行（Git Bash）：

```bash
ssh root@<VPS_IP> 'bash -s' < setup-vps.sh
```

所要時間：約 5〜15 分（Rust のインストール込み）

---

## STEP 4 — VPS 上でビルド

```bash
cd /open-aruaru
export PATH="$HOME/.cargo/bin:$PATH"

# ビルド（リリースモード）
cargo build --release -p aruaru-easyweb
cargo build --release -p aruaru-core

# 確認
ls -lh target/release/aruaru-easyweb target/release/aruaru-core
```

所要時間：初回約 10〜20 分

---

## STEP 5 — バイナリと静的ファイルを配置

```bash
# aruaru-easyweb
cp target/release/aruaru-easyweb /open-aruaru-easyweb/
cp -r aruaru-easyweb/static/     /open-aruaru-easyweb/
cp -r aruaru-easyweb/config/     /open-aruaru-easyweb/

# aruaru-core
cp target/release/aruaru-core /open-aruaru-core/
cp -r aruaru-core/static/     /open-aruaru-core/
cp -r aruaru-core/config/     /open-aruaru-core/

echo "配置完了"
```

---

## STEP 6 — systemd サービス登録 & 起動

```bash
# サービスファイルをコピー
cp /open-aruaru/aruaru-easyweb/aruaru-easyweb.service \
   /etc/systemd/system/aruaru-easyweb.service

# aruaru-core のサービスファイル作成
cat > /etc/systemd/system/aruaru-core.service << 'EOF'
[Unit]
Description=aruaru-core (SSH / SFTP / VPS 管理)
After=network.target

[Service]
User=root
WorkingDirectory=/open-aruaru-core
ExecStart=/open-aruaru-core/aruaru-core
Restart=always
RestartSec=3
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# 有効化 & 起動
systemctl daemon-reload
systemctl enable  aruaru-easyweb aruaru-core
systemctl start   aruaru-easyweb aruaru-core

# 起動確認
systemctl status aruaru-easyweb --no-pager
systemctl status aruaru-core    --no-pager
```

---

## STEP 7 — 動作テスト

Git Bash または WSL から：

```bash
bash test.sh <VPS_IP>
```

または curl で手動確認：

```bash
curl -I http://<VPS_IP>:8080/          # → 302 /setup
curl -I http://<VPS_IP>:8080/setup     # → 200
curl -I http://<VPS_IP>:8080/anthropic # → 200
curl -I http://<VPS_IP>:8081/          # → 200
curl    http://<VPS_IP>:8081/api/vps/detect-ip  # → {"ip":"xxx.xxx.xxx.xxx"}
```

---

## STEP 8 — ブラウザで初期設定

| URL | 内容 |
|-----|------|
| `http://<VPS_IP>:8080/setup` | ① メール・SMTP 設定（初回のみ） |
| `http://<VPS_IP>:8080/login` | ② Magic Link ログイン |
| `http://<VPS_IP>:8080/anthropic` | ③ Anthropic API Key 連携 |
| `http://<VPS_IP>:8081/` | ④ VPS 追加・SSH・ファイル管理 |

---

## トラブルシュート

```bash
# ログ確認
journalctl -u aruaru-easyweb -f     # リアルタイムログ
journalctl -u aruaru-core    -f

# 手動起動（デバッグ）
cd /open-aruaru-easyweb && RUST_LOG=debug ./aruaru-easyweb
cd /open-aruaru-core    && RUST_LOG=debug ./aruaru-core

# ポート確認
ss -tlnp | grep -E '8080|8081'

# サービス再起動
systemctl restart aruaru-easyweb
systemctl restart aruaru-core
```
