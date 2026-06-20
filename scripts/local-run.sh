#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TURBO_BIN="${TURBO_BIN:-$ROOT_DIR/node_modules/.bin/turbo}"
MODE="${1:-debug}"

if [[ ! -x "$TURBO_BIN" ]]; then
  echo "Turbo binary not found: $TURBO_BIN" >&2
  echo "Run dependency install first." >&2
  exit 1
fi

cd "$ROOT_DIR"
if [[ "$MODE" == "--release" || "$MODE" == "release" ]]; then
  "$ROOT_DIR/scripts/build-output.sh"
  "$ROOT_DIR/scripts/adb-install.sh" --release
  "$TURBO_BIN" run serve --parallel --filter=tpush-server-runtime --filter=tpush-panel
else
  "$TURBO_BIN" run build:debug --filter=tpush-app
  "$ROOT_DIR/scripts/adb-install.sh" --debug
  "$TURBO_BIN" run dev --parallel --filter=tpush-server-runtime --filter=tpush-panel
fi
