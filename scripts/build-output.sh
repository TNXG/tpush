#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="$ROOT_DIR/output"
TURBO_BIN="${TURBO_BIN:-$ROOT_DIR/node_modules/.bin/turbo}"

if [[ ! -x "$TURBO_BIN" ]]; then
  echo "Turbo binary not found: $TURBO_BIN" >&2
  echo "Run dependency install first." >&2
  exit 1
fi

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/panel" "$OUTPUT_DIR/server"

cd "$ROOT_DIR"
"$TURBO_BIN" run build

cp "$ROOT_DIR/app/android/app/build/outputs/apk/release/app-release.apk" "$OUTPUT_DIR/tpush-release.apk"
if [[ -f "$ROOT_DIR/app/android/app/build/outputs/apk/debug/app-debug.apk" ]]; then
  cp "$ROOT_DIR/app/android/app/build/outputs/apk/debug/app-debug.apk" "$OUTPUT_DIR/tpush-debug.apk"
fi
cp "$ROOT_DIR/target/release/tpush-server" "$OUTPUT_DIR/server/tpush-server"
cp -R "$ROOT_DIR/app/panel/dist/." "$OUTPUT_DIR/panel/"

cat > "$OUTPUT_DIR/README.txt" <<EOF
TPush build output

- tpush-release.apk: signed Android APK
- tpush-debug.apk: debug Android APK, when available
- server/tpush-server: release server binary
- panel/: static panel assets

Run server:
  DATABASE_URL=sqlite://tpush.sqlite BIND_ADDRESS=0.0.0.0:3000 ./server/tpush-server

Install APK:
  adb install -r tpush-release.apk
EOF

echo "Build artifacts written to $OUTPUT_DIR"
