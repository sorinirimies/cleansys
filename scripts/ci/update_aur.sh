#!/usr/bin/env bash
set -euo pipefail
# Usage: ./scripts/ci/update_aur.sh <version>
# Updates the AUR PKGBUILD with the new version and regenerates .SRCINFO.
# Requires: makepkg (Arch Linux) or just updates the version strings.

VERSION="$1"
PKGBUILD="packaging/aur/cleansys/PKGBUILD"
SRCINFO="packaging/aur/cleansys/.SRCINFO"

# Download tarball and compute sha256
URL="https://github.com/sorinirimies/cleansys/archive/refs/tags/v${VERSION}.tar.gz"
echo "📥 Downloading release tarball for sha256 computation..."
SHA256=$(curl -sL "$URL" | sha256sum | cut -d' ' -f1)
echo "SHA256: $SHA256"

# Update pkgver and sha256sums in PKGBUILD
sed -i "s/^pkgver=.*/pkgver=${VERSION}/" "$PKGBUILD"
sed -i "s/sha256sums=('[^']*')/sha256sums=('${SHA256}')/" "$PKGBUILD"

# Update .SRCINFO version references
sed -i "s/pkgver = .*/pkgver = ${VERSION}/" "$SRCINFO"
sed -i "s/v[0-9]*\.[0-9]*\.[0-9]*/v${VERSION}/g" "$SRCINFO"

echo "✅ Updated PKGBUILD and .SRCINFO to v${VERSION}"
echo ""
echo "Next steps to publish to AUR:"
echo "  1. Clone your AUR repo: git clone ssh://aur@aur.archlinux.org/cleansys.git"
echo "  2. Copy packaging/aur/cleansys/* into it"
echo "  3. Commit and push to AUR"
