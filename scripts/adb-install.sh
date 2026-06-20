#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VARIANT="release"

if [[ "${1:-}" == "--debug" ]]; then
  VARIANT="debug"
  shift
elif [[ "${1:-}" == "--release" ]]; then
  VARIANT="release"
  shift
fi

if [[ "$VARIANT" == "debug" ]]; then
  DEFAULT_APK_PATH="$ROOT_DIR/output/tpush-debug.apk"
else
  DEFAULT_APK_PATH="$ROOT_DIR/output/tpush-release.apk"
fi

APK_PATH="${1:-$DEFAULT_APK_PATH}"
PACKAGE_NAME="${TPUSH_ANDROID_PACKAGE:-moe.tnxg.push}"
ACTIVITY_NAME="${TPUSH_ANDROID_ACTIVITY:-.MainActivity}"

if [[ ! -f "$APK_PATH" ]]; then
  echo "APK not found: $APK_PATH" >&2
  if [[ "$VARIANT" == "debug" ]]; then
    echo "Run: bun run android:debug" >&2
  else
    echo "Run: bun run build:output" >&2
  fi
  exit 1
fi

adb install -r "$APK_PATH"
adb shell pm grant "$PACKAGE_NAME" android.permission.POST_NOTIFICATIONS >/dev/null 2>&1 || true
adb shell am start -n "$PACKAGE_NAME/$ACTIVITY_NAME"
