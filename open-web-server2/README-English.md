# open-web-server

**Making "it never disappears" the default — open-aruaru (a.k.a. iLumi), Layer 1**

Paid items, balances, the data people have spent years building. open-web-server is the thin, fast gatekeeper standing at the front door of it all. It hands the hard problems straight to open-runo and keeps its own job simple: receive, forward, report back. That simplicity is, in our view, the shortest path to a system that can genuinely promise 24/7/365.

Built in pure Rust with [Poem](https://docs.rs/poem/latest/poem/) — the same type-safe feeling that makes Rust addictive starts right here, on the front layer.

---

## Highlights

- **A thin front door to a three-tier system** — holds no data of its own; every write delegates to open-runo (Layer 2). All failure-handling complexity is contained in one place.
- **Loosely coupled via a formal contract** — talks to open-runo over gRPC defined in `proto/runo.proto`. Internal changes to Layer 2/3 never force a Layer 1 rebuild.
- **End-to-end integrity** — computes the BLAKE3 checksum here at write time, and the same value is verified all the way down to Layer 3 (aruaru-db), catching corruption anywhere in transit.
- **Priority switch built into the API** — set `finance: true` on a request to route it as `PRIORITY_FINANCE` (per-record fsync, 3/3 commit required). Game items and financial data share one API safely.
- **Never swallows a failure silently** — if open-runo is unreachable, the response explicitly reports `committed_layers: 0` so the caller knows to retry.

## Specifications

| Item | Detail |
|---|---|
| Language / framework | Rust + Poem |
| Listen port | 8080 (HTTP) |
| Layer 2 connection | gRPC (`RUNO_GRPC_ADDR` env var, default `http://127.0.0.1:50051`) |
| Integrity check | BLAKE3 (32 bytes) |
| Supported priorities | PRIORITY_GAME / PRIORITY_FINANCE |

## API

```bash
# Write (delegated to triple commit)
curl -X POST http://localhost:8080/v1/kv \
  -H "Content-Type: application/json" \
  -d '{"key":"user:1234:wallet","value":{"gold":50000},"finance":true}'

# Read (quorum)
curl http://localhost:8080/v1/kv/user:1234:wallet
```

Example response:

```json
{ "txn_id": "01906e2a-...", "committed_layers": 3, "checksum": "af1349b9...", "status": "committed" }
```

Only finalize a purchase once `committed_layers: 3` is confirmed. `2` means quorum-safe; `0` means open-runo was unreachable.

## Compatibility with Apache/Nginx-style Servers

Apache and Nginx are language-agnostic front doors: they receive HTTP and hand off to a separate process (PHP-FPM, Node, etc). open-web-server was originally designed to run application logic itself, in-process, via Poem (Rust) — so **out of the box it does not have the same "any language" compatibility**. v0.3 adds three routing modes to close that gap.

| Mode | Use case | Apache/Nginx equivalent |
|---|---|---|
| `Native` | Rust+Poem+OpenRuno stack (open-e-gov, OpenRedmine, OpenWordPress) | Embedded module (fastest path) |
| `Proxy` | Existing PHP/Node/Python backends | mod_proxy / proxy_pass |
| `Static` | SPA / static assets | DocumentRoot |

Switched via `routes.toml` (auto-generated — see below).

## No-Contract, Locally Self-Tuning Autoconf

To avoid hand-writing Linux server config files, open-web-server ships a self-tuning loop that makes **no external API calls and requires no subscription or contract**. Stated plainly, without overclaiming: it is an adaptive controller that accumulates its own request statistics (EMA latency, a lightweight p99) in `data/tuning.json` and re-tunes itself against threshold rules. Every 100 requests it:

- increases `worker_threads` if latency is trending high (congestion relief),
- decreases it when load is consistently light, to **save CPU and memory**,
- shortens `keepalive_secs` when p99 spikes, releasing stuck connections sooner.

It then rewrites `data/generated/open-web-server.service` (a systemd unit) and `data/generated/routes.toml`. Learning carries over across restarts via `tuning.json`, so the tuning keeps improving over time. Deploying is just copying the generated files into `/etc/systemd/system/`.

## Supported Linux Distributions

| Distro family | Package manager | Firewall | Notes |
|---|---|---|---|
| Ubuntu / Debian / Linux Mint | apt | ufw | Works out of the box |
| RHEL / AlmaLinux / Rocky Linux | dnf (rclone via EPEL or the official installer) | firewalld | If SELinux is enforcing, the binary needs a `bin_t` context |

`scripts/install-deps.sh` reads `/etc/os-release`, detects the family automatically, and handles rclone installation, port opening (8080/tcp, 50051/tcp, 4433/udp), and the SELinux context in one pass.

```bash
bash scripts/install-deps.sh
```

## Carrying "Learned Smarts" Across OS Reinstalls and New Machines

Building a from-scratch, cross-platform Google Drive client (Windows/Linux/Mac) is out of scope for this project. Instead we take the pragmatic route: **let the battle-tested `rclone` handle sync, and let aruaru's own BLAKE3-based integrity checking guarantee correctness.**

- autoconf writes its learned state to `data/aruaru-brain.json` with an embedded BLAKE3 checksum every 100 requests.
- On startup, if that file exists, its checksum is verified before the learned state is restored. **A file that fails verification (e.g. corrupted mid-sync) is rejected and treated as a fresh start** — the same fail-safe philosophy used everywhere else in this project, so a bad sync can never poison the running state.
- `scripts/sync-brain.sh` (Linux/Mac) and `scripts/sync-brain.ps1` (Windows) push/pull the file to Google Drive (or any rclone-supported backend). Wire either into cron / Task Scheduler for full automation.
- **On RHEL-family distros (AlmaLinux/RHEL/Rocky Linux), a systemd timer is the more idiomatic choice than cron.** `scripts/systemd/aruaru-brain-sync.service` + `.timer` are included — drop them into `/etc/systemd/system/` and run `systemctl enable --now aruaru-brain-sync.timer` for an automatic push every 5 minutes.

```bash
# One-time setup (Google auth happens here, once)
rclone config   # name the remote "gdrive"

# Ongoing: back up every 5 minutes
*/5 * * * * bash /path/to/open-web-server/scripts/sync-brain.sh push

# After a reinstall or on a new machine
bash scripts/sync-brain.sh pull
cargo run --release   # aruaru-brain.json is auto-detected and restored at startup
```

On Ubuntu/Linux Mint, adding your Google account under "Online Accounts" also lets Nautilus/Nemo browse the same Drive, handy if you just want to eyeball `aruaru-brain.json`. The actual sync/restore automation is handled by the scripts above.

## Keeping the Three Repos Compatible

- `proto/runo.proto` is kept identical to the copy in the open-runo repo. Run `scripts/check-proto-sync.sh` to verify both hashes match after any change.
- Layer 1 has no knowledge of replica count or priority-lane internals, so changes on the Layer 2 side (e.g. moving to 4 replicas) never ripple back here.

## Quick Start

```bash
# Start open-runo first (default tcp/50051)
set RUNO_GRPC_ADDR=http://127.0.0.1:50051
cargo run --release
```

## Related Repositories

| Repository | Role |
|---|---|
| open-web-server | Layer 1: front web server (this repo) |
| open-runo | Layer 2: transaction arbiter (QUIC + gRPC dual ingress) |
| aruaru-db | Layer 3: ARU3 triple-write storage |
| open-client | Native frontend shell (Tauri v2) |
| OpenCosmo | GraphQL federation (pure Rust + Poem) |

## Pushing to GitHub

From `f:\open-aruaru` (the drive-mounted working folder), you can `git init` → `commit` → `push` all five repos in one go.

```bash
export GITHUB_USER=your-account
bash open-web-server/scripts/push-all.sh
# On Windows PowerShell:
# .\open-web-server\scripts\push-all.ps1 -GitHubUser your-account
```

Create the matching repos on GitHub first (`open-web-server`, etc.) and make sure SSH keys or the GitHub CLI are authenticated. `data/` (learned state, DB segments, and other runtime data) is auto-added to `.gitignore` to prevent accidental pushes.

## License

MIT OR Apache-2.0

---

*Once you've read this far, all that's left is `cargo run`. The moment it's talking to open-runo and aruaru-db, your next idea already stands on ground that doesn't disappear.*
