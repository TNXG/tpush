# TPush scripts

## Debug install

Debug build does not require `secure_sign.p12`.

Build debug APK:

```bash
bun run android:debug
```

Debug APK output:

```plaintext
output/tpush-debug.apk
```

Install debug APK through adb and launch the app:

```bash
bun run android:install
```

Build debug APK, install it, then start server and panel:

```bash
bun run local
```

The app starts `ForegroundService` from `MainApplication`, so launching the app after adb install is enough to bring the push service up.

## Release signing env

Release APK builds use `secure_sign.p12` by default.

When running in an interactive terminal, missing password and alias values will be requested automatically.

```bash
export TPUSH_SIGNING_STORE_PASSWORD="your-p12-password"
export TPUSH_SIGNING_KEY_ALIAS="your-key-alias"
export TPUSH_SIGNING_KEY_PASSWORD="$TPUSH_SIGNING_STORE_PASSWORD"
```

Optional:

```bash
export TPUSH_SIGNING_STORE_FILE="/absolute/path/to/secure_sign.p12"
```

If you need to inspect the alias:

```bash
keytool -list -storetype pkcs12 -keystore secure_sign.p12
```

## Release commands

Build signed APK, release server binary and panel static assets into `output/`:

```bash
bun run build:output
```

Install the built APK through adb:

```bash
bun run install
```

Build, install, then start server and panel:

```bash
bun run local:release
```

Run checks:

```bash
bun run typecheck
bun run test
```
