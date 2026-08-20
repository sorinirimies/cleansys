#!/usr/bin/env bash
set -euo pipefail
# Usage: ./scripts/ci/build_universal_macos.sh
# Merges x86_64 and aarch64 macOS binaries into universal binaries using lipo.

mkdir -p target/universal-apple-darwin/release

for BIN in cleansys cleansys-gui; do
    ARM="target/aarch64-apple-darwin/release/${BIN}"
    X86="target/x86_64-apple-darwin/release/${BIN}"
    OUT="target/universal-apple-darwin/release/${BIN}"

    if [ -f "$ARM" ] && [ -f "$X86" ]; then
        echo "🔗 Creating universal binary for ${BIN}..."
        lipo -create "$ARM" "$X86" -output "$OUT"
        echo "✅ ${OUT} (universal)"
    elif [ -f "$ARM" ]; then
        echo "⚠️  x86_64 not found — using arm64 only for ${BIN}"
        cp "$ARM" "$OUT"
    elif [ -f "$X86" ]; then
        echo "⚠️  arm64 not found — using x86_64 only for ${BIN}"
        cp "$X86" "$OUT"
    else
        echo "❌ Neither arm64 nor x86_64 binary found for ${BIN}"
        exit 1
    fi
done

echo ""
echo "📦 Universal binaries:"
ls -lh target/universal-apple-darwin/release/cleansys*
