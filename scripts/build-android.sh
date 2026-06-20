#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ANDROID_DIR="$ROOT_DIR/app/android"
NATIVE_LIB_DIR="$ANDROID_DIR/app/src/main/jniLibs/arm64-v8a"
VARIANT="${1:-${TPUSH_ANDROID_VARIANT:-release}}"
ANDROID_SDK_DIR="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
OUTPUT_DIR="$ROOT_DIR/output"

prompt_required() {
  local variable_name="$1"
  local prompt_text="$2"
  local silent="${3:-false}"

  if [[ -n "${!variable_name:-}" ]]; then
    return
  fi

  local value

  # macOS: prefer GUI dialog to avoid turbo TUI keyboard capture issues
  if [[ "$(uname -s)" == "Darwin" ]]; then
    local hidden_flag=""
    [[ "$silent" == "true" ]] && hidden_flag="with hidden answer"
    value=$(osascript \
      -e "Tell application \"System Events\" to display dialog \"$prompt_text\" default answer \"\" $hidden_flag with title \"TPush Signing\"" \
      -e 'text returned of result' 2>/dev/null)

    if [[ -n "$value" ]]; then
      export "$variable_name=$value"
      return
    fi
    # osascript failed — fall through to terminal prompt
  fi

  # Terminal prompt (also serves as fallback when osascript fails)
  if [[ -t 0 ]] || [[ -r /dev/tty ]]; then
    if [[ "$silent" == "true" ]]; then
      read -r -s -p "$prompt_text: " value < /dev/tty
      echo > /dev/tty
    else
      read -r -p "$prompt_text: " value < /dev/tty
    fi

    if [[ -n "$value" ]]; then
      export "$variable_name=$value"
      return
    fi
  fi

  echo "Missing signing env: $variable_name" >&2
  echo "Run in an interactive terminal, set $variable_name, or run on macOS for GUI prompt." >&2
  exit 1
}

detect_alias() {
  # Extract the first alias from the p12 keystore.
  # keytool output lines with an alias look like: "app_sign, 2026年6月19日, PrivateKeyEntry,"
  local first_entry
  first_entry=$(keytool -list -keystore "$TPUSH_SIGNING_STORE_FILE" \
    -storetype PKCS12 \
    -storepass "$TPUSH_SIGNING_STORE_PASSWORD" 2>/dev/null \
    | grep ',' | head -1 | cut -d',' -f1 | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')

  if [[ -n "$first_entry" ]]; then
    export TPUSH_SIGNING_KEY_ALIAS="$first_entry"
    echo "Detected key alias: $first_entry" >&2
    return 0
  fi
  return 1
}

if [[ "$VARIANT" != "debug" && "$VARIANT" != "release" ]]; then
  echo "Unsupported Android variant: $VARIANT" >&2
  echo "Use: debug or release" >&2
  exit 1
fi

if [[ "$VARIANT" == "release" ]]; then
  export TPUSH_SIGNING_STORE_FILE="${TPUSH_SIGNING_STORE_FILE:-$ROOT_DIR/secure_sign.p12}"

  if [[ ! -f "$TPUSH_SIGNING_STORE_FILE" ]]; then
    echo "Signing store not found: $TPUSH_SIGNING_STORE_FILE" >&2
    exit 1
  fi

  prompt_required "TPUSH_SIGNING_STORE_PASSWORD" "P12 password for $TPUSH_SIGNING_STORE_FILE" true

  if [[ -z "${TPUSH_SIGNING_KEY_ALIAS:-}" ]]; then
    detect_alias || prompt_required "TPUSH_SIGNING_KEY_ALIAS" "P12 key alias"
  fi

  export TPUSH_SIGNING_KEY_PASSWORD="${TPUSH_SIGNING_KEY_PASSWORD:-$TPUSH_SIGNING_STORE_PASSWORD}"
fi

if [[ -z "$ANDROID_SDK_DIR" ]]; then
  if [[ -d "$HOME/Library/Android/sdk" ]]; then
    ANDROID_SDK_DIR="$HOME/Library/Android/sdk"
  elif [[ -d "$HOME/Android/Sdk" ]]; then
    ANDROID_SDK_DIR="$HOME/Android/Sdk"
  fi
fi

if [[ -z "$ANDROID_SDK_DIR" || ! -d "$ANDROID_SDK_DIR" ]]; then
  echo "Android SDK not found. Set ANDROID_HOME or ANDROID_SDK_ROOT." >&2
  exit 1
fi

export ANDROID_HOME="$ANDROID_SDK_DIR"
printf "sdk.dir=%s\n" "$ANDROID_SDK_DIR" > "$ANDROID_DIR/local.properties"

unset RUSTC_WRAPPER
cargo build -p tpush_core --target aarch64-linux-android --release
mkdir -p "$NATIVE_LIB_DIR"
cp "$ROOT_DIR/target/aarch64-linux-android/release/libtpush_core.so" "$NATIVE_LIB_DIR/libtpush_core.so"

cd "$ANDROID_DIR"
if [[ "$VARIANT" == "debug" ]]; then
  JAVA_HOME="${JAVA_HOME:-$(/usr/libexec/java_home -v 21)}" ./gradlew :app:assembleDebug --no-daemon
  mkdir -p "$OUTPUT_DIR"
  cp "$ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk" "$OUTPUT_DIR/tpush-debug.apk"
  echo "Debug APK written to $OUTPUT_DIR/tpush-debug.apk"
else
  JAVA_HOME="${JAVA_HOME:-$(/usr/libexec/java_home -v 21)}" ./gradlew :app:assembleRelease --no-daemon
  mkdir -p "$OUTPUT_DIR"
  cp "$ANDROID_DIR/app/build/outputs/apk/release/app-release.apk" "$OUTPUT_DIR/tpush-release.apk"
  echo "Release APK written to $OUTPUT_DIR/tpush-release.apk"
fi
