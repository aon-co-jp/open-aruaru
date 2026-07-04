from pathlib import Path
root = Path(__file__).resolve().parents[1]
required = [
    'Cargo.toml',
    'crates/aruaru-core/src/lib.rs',
    'crates/aruaru-gql-gateway/src/main.rs',
    'crates/aruaru-offline-cache/src/lib.rs',
    'cosmo/subgraphs/aruaru-platform-v28.graphql',
    'apps/aruaru-web/src/main.tsx',
    'apps/aruaru-desktop/src/main.tsx',
    'apps/aruaru-mobile/src/main.tsx',
]
missing = [p for p in required if not (root/p).exists()]
if missing:
    raise SystemExit(f'missing required files: {missing}')
text = '\n'.join((root/p).read_text(errors='ignore') for p in required if (root/p).is_file())
checks = [
    'runOfflinePrefetch',
    'purgeOfflineCache',
    'audiocafe.tokyo',
    'retention_days: 3',
    'GraphQL Federation',
    'noRest',
]
missing_checks = [c for c in checks if c not in text]
if missing_checks:
    raise SystemExit(f'missing expected implementation markers: {missing_checks}')
for banned in ['@tauri-apps/', 'src-tauri']:
    hits = [str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and p.name != 'validate-package-v28.py' and banned in p.read_text(errors='ignore')]
    if hits:
        raise SystemExit(f'banned marker {banned!r} found in {hits[:10]}')
print('validate-package OK for v28')
