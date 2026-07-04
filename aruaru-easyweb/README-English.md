# aruaru-easyweb

**The Second KUSANAGI** — VPS Web Server Manager built with Rust + Poem

---

## Overview

aruaru-easyweb is a browser-based management app for VPS web server configuration.  
No VI, nano, or any manual file editing required — everything is handled through the GUI.

---

## Features

### 🔐 Magic Link Login (Passwordless)
- Access via IP address immediately after VPS boot
- Enter your email → receive a one-time login link
- Links expire automatically after 15 minutes
- Simple security: re-enter your email address every session

### 🌐 Domain & Subdomain Management
- **Automatic www + bare domain handling** (e.g. `example.com` and `www.example.com`)
- Auto-redirect from `www.example.com` → `example.com`
- Register/update the aruaru-easyweb app's own domain
- Add, remove, and update domains entirely via GUI

### 🔒 Automatic HTTPS (Let's Encrypt)
- **Auto-obtains** SSL certificates on domain registration (certbot integration)
- Covers both bare domain and www simultaneously
- **Auto-renews** when fewer than 30 days remain (scheduler runs daily at 03:00)
- Dashboard shows certificate expiry in real time

### ⚙️ Nginx / Apache Auto-Configuration (No VI required)
- **Auto-generates and writes** config files on domain add/update
- **Auto-runs** `nginx -s reload` or `systemctl reload apache2`
- **Auto-runs** `a2ensite`, `a2enmod` for Apache
- **Auto-runs** config tests (`nginx -t` / `apachectl configtest`)
- **One-click switch** between Apache and Nginx from the GUI toggle

### 🤖 Web Server Selection Assistant
- Select your framework from buttons and receive an AI recommendation
- WordPress → Apache, Rust/Go/Node.js → Nginx, with reasoning included

---

## Quick Start

```bash
# 1. Build
cd f:\aruaru\aruaru-easyweb
cargo build --release

# 2. Configure
cp .env.example .env
# Edit .env — set your SMTP credentials and admin email

# 3. Run
./target/release/aruaru-easyweb

# 4. Open in browser
# http://<VPS_IP>:8080
```

---

## VPS Deployment

```bash
# Copy binary to VPS
scp target/release/aruaru-easyweb user@your-vps:/home/aruaru-easyweb/

# Register as systemd service
sudo cp aruaru-easyweb.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now aruaru-easyweb

# Check status
sudo systemctl status aruaru-easyweb
```

---

## Directory Structure

```
aruaru-easyweb/
├── Cargo.toml
├── .env.example                    # Environment variable template
├── config/default.toml             # App configuration
├── aruaru-easyweb.service          # systemd service unit
├── src/
│   ├── main.rs                     # Entry point
│   ├── config.rs                   # Config loader
│   ├── scheduler.rs                # Auto-renewal scheduler
│   ├── auth/                       # Magic Link authentication
│   ├── db/                         # SQLite + migrations
│   ├── domain/                     # Domain management + SSL
│   ├── webserver/
│   │   ├── nginx.rs                # Config gen, write, reload
│   │   ├── apache.rs               # Config gen, write, reload
│   │   └── recommender.rs          # AI recommendation logic
│   ├── routes/                     # API endpoints
│   └── templates/                  # HTML templates
├── static/
│   ├── css/app.css                 # Dark UI
│   └── js/app.js                   # Toggle & modal logic
└── data/
    └── aruaru.db                   # SQLite (auto-created)
```

---

## Tech Stack

| Item | Technology |
|------|------------|
| Language | Rust 2021 edition |
| Web Framework | Poem 3 |
| Database | SQLite (sqlx 0.7) |
| SSL Certificates | Let's Encrypt / certbot |
| Scheduler | tokio-cron-scheduler |
| Email | lettre 0.11 |

---

## Automated Operations (No Manual File Editing)

When you add a domain through the GUI, aruaru-easyweb automatically:

1. Normalizes the domain to bare form (strips `www.`)
2. Generates Nginx or Apache config with www → bare redirect
3. Writes the config file to `/etc/nginx/sites-available/` or `/etc/apache2/sites-available/`
4. Enables the site (`symlink` or `a2ensite`)
5. Runs a config syntax test
6. Reloads the web server
7. Optionally runs certbot to obtain HTTPS certificates for both bare and www

All of this happens with a single click — no SSH, no VI, no manual steps.

---

## License

aruaru project
