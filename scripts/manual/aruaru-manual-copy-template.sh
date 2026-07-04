#!/usr/bin/env bash
set -euo pipefail

SOURCE_ROOT="${SOURCE_ROOT:-/opt/aruaru/aruaru-rs4}"
DESTINATION_ROOT="${DESTINATION_ROOT:-/opt/aruaru/aruaru-rs4-copy}"
DRY_RUN="${DRY_RUN:-1}"
INCLUDE_GENERATED_README="${INCLUDE_GENERATED_README:-0}"
INCLUDE_LOGS="${INCLUDE_LOGS:-0}"

echo '== copy plan =='
echo "Source:      $SOURCE_ROOT"
echo "Destination: $DESTINATION_ROOT"
echo "DryRun:      $DRY_RUN"

test -d "$SOURCE_ROOT"
mkdir -p "$DESTINATION_ROOT"

EXCLUDES=(--exclude='.git' --exclude='target' --exclude='node_modules' --exclude='dist' --exclude='build' --exclude='tmp-web' --exclude='.env' --exclude='*.key' --exclude='*.pem' --exclude='*.pfx' --exclude='id_rsa' --exclude='id_ed25519')
if [[ "$INCLUDE_GENERATED_README" != "1" ]]; then EXCLUDES+=(--exclude='README.rs' --exclude='README.html'); fi
if [[ "$INCLUDE_LOGS" != "1" ]]; then EXCLUDES+=(--exclude='.aruaru' --exclude='*.log'); fi

if ! command -v rsync >/dev/null 2>&1; then
  echo 'rsync not found. Please install rsync or use the PowerShell script on Windows.'
  exit 1
fi

if [[ "$DRY_RUN" == "1" ]]; then
  rsync -av --dry-run "${EXCLUDES[@]}" "$SOURCE_ROOT/" "$DESTINATION_ROOT/"
  echo 'OK: manual copy dry-run passed'
else
  rsync -av "${EXCLUDES[@]}" "$SOURCE_ROOT/" "$DESTINATION_ROOT/"
  echo 'OK: manual copy completed'
fi
