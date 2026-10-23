#!/usr/bin/env bash
# Build Rusty Compress for macOS with the Finder right-click extension.
#
#   bash scripts/build-macos.sh [--skip-tauri] [--install] [--restart-finder] [--dmg]
#
#   --skip-tauri      reuse the existing Tauri .app instead of rebuilding it
#   --install         copy the app to /Applications and register the Finder extension
#   --restart-finder  killall Finder after --install (only then)
#   --dmg             also create a .dmg in src-tauri/target/release/bundle/dmg/
#
# Signing identity: $APPLE_SIGNING_IDENTITY, else the first "Apple Development"
# identity in the keychain, else ad-hoc ("-") with a warning.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP_NAME="Rusty Compress"
APPEX_NAME="RustyCompressFinder"
EXT_BUNDLE_ID="com.rusty-compress.app.FinderExtension"
EXT_SRC="$ROOT/macos/FinderExtension"
BUILD_DIR="$ROOT/macos/build"
APPEX="$BUILD_DIR/$APPEX_NAME.appex"
BUNDLE_DIR="$ROOT/src-tauri/target/release/bundle"
APP="$BUNDLE_DIR/macos/$APP_NAME.app"
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"

SKIP_TAURI=0
INSTALL=0
RESTART_FINDER=0
MAKE_DMG=0
for arg in "$@"; do
  case "$arg" in
    --skip-tauri) SKIP_TAURI=1 ;;
    --install) INSTALL=1 ;;
    --restart-finder) RESTART_FINDER=1 ;;
    --dmg) MAKE_DMG=1 ;;
    -h|--help) sed -n '2,14p' "$0"; exit 0 ;;
    *) echo "Unknown option: $arg" >&2; exit 2 ;;
  esac
done

if [[ $RESTART_FINDER -eq 1 && $INSTALL -eq 0 ]]; then
  echo "--restart-finder only applies together with --install" >&2
  exit 2
fi

step() { printf '\n==> %s\n' "$*"; }

# ---------------------------------------------------------------------------
# 1. Main app (Tauri)
# ---------------------------------------------------------------------------
if [[ $SKIP_TAURI -eq 0 ]]; then
  step "Building Tauri app (npm run tauri build -- --bundles app)"
  npm run tauri build -- --bundles app
fi

if [[ ! -d "$APP" ]]; then
  echo "App bundle not found: $APP (run without --skip-tauri first)" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# 2. FinderSync appex, universal (arm64 + x86_64)
# ---------------------------------------------------------------------------
step "Compiling $APPEX_NAME.appex (arm64 + x86_64)"
rm -rf "$APPEX"
mkdir -p "$APPEX/Contents/MacOS" "$BUILD_DIR/tmp"

SWIFT_SRC="$EXT_SRC/FinderSync.swift"
for arch in arm64 x86_64; do
  swiftc -O -module-name RustyCompressFinder -parse-as-library -application-extension \
    -target "$arch-apple-macos11.0" \
    -framework Cocoa -framework FinderSync \
    -Xlinker -e -Xlinker _NSExtensionMain \
    "$SWIFT_SRC" -o "$BUILD_DIR/tmp/$APPEX_NAME-$arch"
done
lipo -create "$BUILD_DIR/tmp/$APPEX_NAME-arm64" "$BUILD_DIR/tmp/$APPEX_NAME-x86_64" \
  -output "$APPEX/Contents/MacOS/$APPEX_NAME"
rm -rf "$BUILD_DIR/tmp"
lipo -info "$APPEX/Contents/MacOS/$APPEX_NAME"

cp "$EXT_SRC/Info.plist" "$APPEX/Contents/Info.plist"
printf 'XPC!????' > "$APPEX/Contents/PkgInfo"

# ---------------------------------------------------------------------------
# 3. Embed into the app
# ---------------------------------------------------------------------------
step "Embedding appex into $APP_NAME.app/Contents/PlugIns"
PLUGINS="$APP/Contents/PlugIns"
mkdir -p "$PLUGINS"
rm -rf "$PLUGINS/$APPEX_NAME.appex"
cp -R "$APPEX" "$PLUGINS/$APPEX_NAME.appex"
EMBEDDED_APPEX="$PLUGINS/$APPEX_NAME.appex"

# ---------------------------------------------------------------------------
# 4. Sign (inside-out)
# ---------------------------------------------------------------------------
IDENTITY="${APPLE_SIGNING_IDENTITY:-}"
if [[ -z "$IDENTITY" ]]; then
  # Use the SHA-1 hash (column 2) so a duplicated common name cannot make codesign fail.
  IDENTITY="$(security find-identity -v -p codesigning 2>/dev/null \
    | awk '/Apple Development/ {print $2; exit}' || true)"
fi
if [[ -z "$IDENTITY" ]]; then
  IDENTITY="-"
  echo "WARNING: no signing identity found; using ad-hoc signing (-)." >&2
  echo "         Set APPLE_SIGNING_IDENTITY or create an Apple Development cert in Xcode > Settings > Accounts." >&2
fi
step "Signing with identity: $IDENTITY"

# An interrupted codesign leaves *.cstemp files that break verification.
find "$APP" -name '*.cstemp' -print -delete

codesign --force --options runtime --timestamp=none \
  --entitlements "$EXT_SRC/RustyCompressFinder.entitlements" \
  -s "$IDENTITY" "$EMBEDDED_APPEX"

# No hardened runtime on the main app: the Tauri webview build was not tested with it.
codesign --force --timestamp=none -s "$IDENTITY" "$APP"

step "Verifying signature"
codesign --verify --deep --strict --verbose=2 "$APP"

# ---------------------------------------------------------------------------
# 5. Optional DMG
# ---------------------------------------------------------------------------
DMG=""
if [[ $MAKE_DMG -eq 1 ]]; then
  VERSION="$(node -p "require('./src-tauri/tauri.conf.json').version")"
  DMG_DIR="$BUNDLE_DIR/dmg"
  DMG="$DMG_DIR/Rusty Compress_${VERSION}_mac.dmg"
  step "Creating DMG: $DMG"
  mkdir -p "$DMG_DIR"
  rm -f "$DMG"
  hdiutil create -volname "$APP_NAME" -srcfolder "$APP" -ov -format UDZO "$DMG"
fi

# ---------------------------------------------------------------------------
# 6. Optional install
# ---------------------------------------------------------------------------
if [[ $INSTALL -eq 1 ]]; then
  INSTALLED="/Applications/$APP_NAME.app"
  step "Installing to $INSTALLED"
  rm -rf "$INSTALLED"
  cp -R "$APP" "$INSTALLED"

  "$LSREGISTER" -f "$INSTALLED"
  pluginkit -a "$INSTALLED/Contents/PlugIns/$APPEX_NAME.appex"
  pluginkit -e use -i "$EXT_BUNDLE_ID"

  if [[ $RESTART_FINDER -eq 1 ]]; then
    step "Restarting Finder"
    killall Finder || true
  fi
fi

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
step "Build complete"
echo "App:       $APP"
echo "Extension: $EMBEDDED_APPEX"
[[ -n "$DMG" ]] && echo "DMG:       $DMG"

cat <<EOF

Next steps:
  - Install and enable (run once):  bash scripts/build-macos.sh --skip-tauri --install
  - Enable in Finder:               System Settings > General > Login Items & Extensions > Finder
                                    (or: pluginkit -e use -i $EXT_BUNDLE_ID)
  - Check registration:             pluginkit -m -i $EXT_BUNDLE_ID   (expect a leading '+')
  - Restart Finder if the menu does not appear:  killall Finder
  - Unsigned/ad-hoc builds on other Macs: see README "macOS Finder Extension".
EOF
