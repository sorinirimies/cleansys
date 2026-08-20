#!/usr/bin/env bash
set -euo pipefail
# Usage: ./scripts/ci/package_windows.sh <version>
# Builds the Windows NSIS installer. Run on a Windows runner with NSIS installed.

VERSION="$1"
NSI="packaging/windows/installer.nsi"
DIST="dist"
ICON="packaging/windows/cleansys.ico"

mkdir -p "$DIST"

# The NSIS script (MUI_ICON / MUI_UNICON) requires a real .ico file to exist
# at build time, but it is deliberately not committed to the repo (see
# packaging/windows/cleansys.ico.txt). Generate a simple fallback icon here
# if one hasn't been provided, so the installer build never hard-fails on a
# missing file. Embed a pre-built, verified-valid classic 16x16 32bpp ICO
# (BMP-encoded, not the Vista+ PNG-compressed format) as a base64 blob and
# decode it directly.
if [ ! -f "$ICON" ]; then
    echo "ℹ️  $ICON not found — writing a bundled fallback icon."
    base64 -d > "$ICON" <<'ICO_BASE64'
AAABAAEAEBAAAAEAIABoBAAAFgAAACgAAAAQAAAAIAAAAAEAIAAAAAAAQAQAAAAAAAAAAAAAAAAA
AAAAAAAbGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/
GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8b
GBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsY
GP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP88TOf/PEzn/zxM5/88TOf/PEzn
/zxM5/88TOf/PEzn/zxM5/88TOf/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/PEzn/zxM5/88TOf/
PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/PEzn/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/zxM5/88
TOf/PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/PEzn/zxM5/8bGBj/GxgY/xsYGP8bGBj/GxgY/xsY
GP88TOf/PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/GxgY/xsYGP8bGBj/GxgY
/xsYGP8bGBj/PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/PEzn/xsYGP8bGBj/
GxgY/xsYGP8bGBj/GxgY/zxM5/88TOf/PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/PEzn/zxM5/8b
GBj/GxgY/xsYGP8bGBj/GxgY/xsYGP88TOf/PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/PEzn/zxM
5/88TOf/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/PEzn
/zxM5/88TOf/PEzn/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/zxM5/88TOf/PEzn/zxM5/88TOf/
PEzn/zxM5/88TOf/PEzn/zxM5/8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP88TOf/PEzn/zxM5/88
TOf/PEzn/zxM5/88TOf/PEzn/zxM5/88TOf/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsY
GP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY
/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/
GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8bGBj/GxgY/xsYGP8b
GBj/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
AAAAAAAAAAAAAA==
ICO_BASE64
    if [ ! -f "$ICON" ] || [ ! -s "$ICON" ]; then
        echo "❌ Fallback icon generation failed — $ICON still missing or empty."
        exit 1
    fi
    echo "✅ Wrote fallback icon at $ICON ($(wc -c < "$ICON" | tr -d ' ') bytes)"
fi

# The .nsi script is compiled from a versioned copy at dist/installer_versioned.nsi,
# not from its own committed location, and NSIS resolves every relative path
# used by File/LicenseData/Icon-style commands relative to the *compiled
# script's own directory*. The .nsi script re-anchors all of its relative
# paths to the real repository root via `!cd "@REPO_ROOT_ABS@"`; resolve
# that absolute Windows path here using Git Bash's `pwd -W` (native
# drive-letter path), then double backslashes before using it in sed's
# replacement text.
REPO_ROOT_ABS="$(pwd -W | tr '/' '\\')"
echo "ℹ️  Resolved repo root to absolute path: $REPO_ROOT_ABS"
REPO_ROOT_ABS_SED="${REPO_ROOT_ABS//\\/\\\\}"

# Substitute placeholders
sed -e "s/@VERSION@/${VERSION}/g" \
    -e "s#@REPO_ROOT_ABS@#${REPO_ROOT_ABS_SED}#g" \
    "$NSI" > "$DIST/installer_versioned.nsi"

echo "🔨 Building Windows installer with NSIS..."
# makensis is installed by choco to a fixed path not automatically on bash PATH
'/c/Program Files (x86)/NSIS/makensis.exe' "$DIST/installer_versioned.nsi"

echo "✅ Built dist/cleansys-${VERSION}-windows-x86_64-setup.exe"
