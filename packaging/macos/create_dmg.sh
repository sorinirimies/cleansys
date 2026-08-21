#!/usr/bin/env bash
set -euo pipefail
# Usage: ./packaging/macos/create_dmg.sh <version>
# Creates a macOS DMG containing both CleanSys binaries.
# Requires: create-dmg (brew install create-dmg)

VERSION="$1"
DIST="dist"
APP_NAME="CleanSys"
BUNDLE="${DIST}/${APP_NAME}.app"

mkdir -p "$DIST"

# ── Create .app bundle for the GUI ───────────────────────────────────────────
mkdir -p "${BUNDLE}/Contents/MacOS"
mkdir -p "${BUNDLE}/Contents/Resources"

# Copy universal binary (lipo-merged) or individual arch binary
if [ -f "target/universal-apple-darwin/release/cleansys-gui" ]; then
    cp "target/universal-apple-darwin/release/cleansys-gui" "${BUNDLE}/Contents/MacOS/cleansys-gui"
elif [ -f "target/aarch64-apple-darwin/release/cleansys-gui" ]; then
    cp "target/aarch64-apple-darwin/release/cleansys-gui" "${BUNDLE}/Contents/MacOS/cleansys-gui"
else
    cp "target/x86_64-apple-darwin/release/cleansys-gui" "${BUNDLE}/Contents/MacOS/cleansys-gui"
fi
chmod +x "${BUNDLE}/Contents/MacOS/cleansys-gui"

# Copy TUI binary alongside the .app for terminal usage
if [ -f "target/universal-apple-darwin/release/cleansys" ]; then
    cp "target/universal-apple-darwin/release/cleansys" "${DIST}/cleansys"
elif [ -f "target/aarch64-apple-darwin/release/cleansys" ]; then
    cp "target/aarch64-apple-darwin/release/cleansys" "${DIST}/cleansys"
else
    cp "target/x86_64-apple-darwin/release/cleansys" "${DIST}/cleansys"
fi

# Info.plist
cat > "${BUNDLE}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>cleansys-gui</string>
    <key>CFBundleIdentifier</key>
    <string>com.sorinirimies.cleansys</string>
    <key>CFBundleName</key>
    <string>CleanSys</string>
    <key>CFBundleDisplayName</key>
    <string>CleanSys</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
EOF

echo "✅ Created ${APP_NAME}.app bundle"

# ── Create DMG ────────────────────────────────────────────────────────────────
OUTPUT="${DIST}/cleansys-${VERSION}-macos.dmg"

# The DMG volume icon is optional artwork (see cleansys.icns.txt) that is
# NOT committed to the repo. Only pass --volicon when it actually exists so
# create-dmg never hard-fails on a missing icon file -- same fallback pattern
# as the Windows installer's bundled fallback .ico.
ICNS="packaging/macos/cleansys.icns"
VOLICON_ARGS=()
if [ -f "$ICNS" ]; then
    VOLICON_ARGS=(--volicon "$ICNS")
else
    echo "info: $ICNS not found - building DMG without a custom volume icon."
fi

if command -v create-dmg &>/dev/null; then
    create-dmg \
        --volname "CleanSys ${VERSION}" \
        "${VOLICON_ARGS[@]}" \
        --window-pos 200 120 \
        --window-size 660 400 \
        --icon-size 128 \
        --icon "CleanSys.app" 160 185 \
        --hide-extension "CleanSys.app" \
        --app-drop-link 500 185 \
        --no-internet-enable \
        "$OUTPUT" \
        "$DIST/" \
    2>/dev/null || true
else
    # Fallback: plain hdiutil DMG (no fancy layout)
    hdiutil create -volname "CleanSys ${VERSION}" \
        -srcfolder "$DIST" \
        -ov -format UDZO \
        "$OUTPUT"
fi

echo "✅ Built ${OUTPUT}"
