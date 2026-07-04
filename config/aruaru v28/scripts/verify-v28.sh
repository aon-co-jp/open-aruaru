#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
[ -f "$ROOT/Cargo.toml" ]
[ -f "$ROOT/crates/aruaru-gql-gateway/src/main.rs" ]
[ -f "$ROOT/crates/aruaru-offline-cache/src/lib.rs" ]
[ -f "$ROOT/cosmo/subgraphs/aruaru-platform-v28.graphql" ]
[ -f "$ROOT/apps/aruaru-web/src/main.tsx" ]
[ -f "$ROOT/apps/aruaru-desktop/src/main.tsx" ]
[ -f "$ROOT/apps/aruaru-mobile/src/main.tsx" ]
python3 "$ROOT/scripts/validate-package-v28.py"
echo "v28 packaging verification OK"
