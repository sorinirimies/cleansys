# macOS Packaging

## DMG Creation

The `create_dmg.sh` script creates a macOS disk image (`.dmg`) containing:
- `CleanSys.app` — the desktop GUI as a proper macOS application bundle
- `cleansys` — the terminal binary (for manual install to `/usr/local/bin`)

### Building locally

1. Install create-dmg: `brew install create-dmg`
2. Build for both architectures:
   ```bash
   cargo build --release -p cleansys-gui -p cleansys-tui --target aarch64-apple-darwin
   cargo build --release -p cleansys-gui -p cleansys-tui --target x86_64-apple-darwin
   ```
3. Create universal binaries:
   ```bash
   ./scripts/ci/build_universal_macos.sh
   ```
4. Create the DMG:
   ```bash
   ./packaging/macos/create_dmg.sh 0.4.0
   ```

## Code Signing (optional)

For distribution outside the App Store, sign with:
```bash
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name (TEAM_ID)" \
  dist/CleanSys.app

# Notarize
xcrun notarytool submit dist/cleansys-*.dmg \
  --apple-id your@apple.id \
  --team-id TEAM_ID \
  --password APP_SPECIFIC_PASSWORD \
  --wait
```
