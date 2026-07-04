# open-client

**Making "it never disappears" the default — open-aruaru (a.k.a. iLumi), native frontend**

Built on **Tauri v2**, not Electron. The OS's native WebView keeps the bundle small and memory footprint low, and lets the frontend share a Rust core with open-web-server, open-runo, and aruaru-db. On top of that, its own **predictive caching engine** trims unnecessary round-trips to the server, improving both perceived speed and server load.

---

## Why Tauri

| Aspect | Electron | Tauri |
|---|---|---|
| Runtime | Bundled Chromium | OS-native WebView |
| Bundle size | Large (100MB+ typical) | Small (a few MB) |
| Resident memory | Node.js + Chromium overhead | Substantially lower |
| Attack surface | Resident Node.js process | Minimal Rust core only |
| Backend language | JS/Node | **Rust** (shared with open-runo, etc.) |

Since all three backend layers (open-web-server / open-runo / aruaru-db) are written in Rust, writing the frontend in Rust too means shared types and shared checksum logic — fewer chances for a subtle mismatch to turn into a data-integrity bug.

## Predictive Caching Engine (predictive.rs)

Where WunderGraph Cosmo is a Go-based GraphQL federation layer, open-runo takes a different path: **pure Rust plus a lightweight, pluggable predictive layer** that decides what can skip the round-trip. open-client extends that same philosophy to the client:

- Tracks per-key access frequency (exponential moving average) and only allows short-TTL local caching for **hot, non-financial** keys.
- **Financial data (balances, etc.) is never eligible for prediction** — it always resolves through a real quorum read on the server, structurally ruling out cache-induced inconsistency.
- A key whose predictions keep missing is automatically demoted back to "untrusted," falling back to the safe, server-confirmed path.
- Built around a `Predictor` trait so the heuristic model here can later be swapped for a real model from aruaru-llm (OpenLLM) without touching call sites.

Writes always go through open-runo's real triple-commit path. The predictive engine only pre-filters reads — it never bypasses the durability guarantees.

## Communication Path

```
open-client ──QUIC (udp/4433)──▶ open-runo ──▶ aruaru-db ×3
     │                          (real triple-commit path)
     └─(future) HTTPS ──▶ open-web-server ──▶ same path
```

Game/real-time traffic connects straight to open-runo, skipping open-web-server as an extra hop.

## Quick Start

```bash
cd src-tauri
set RUNO_QUIC_ADDR=127.0.0.1:4433
cargo tauri dev
```

Production build:

```bash
cargo tauri build
```

## Related Repositories

| Repository | Role |
|---|---|
| open-web-server | Layer 1: front web server |
| open-runo | Layer 2: transaction arbiter |
| aruaru-db | Layer 3: triple-write storage |
| open-client | Unified frontend shell (this repo) |

## License

MIT OR Apache-2.0
