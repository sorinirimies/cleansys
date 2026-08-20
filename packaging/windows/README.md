# Windows Packaging

## NSIS Installer

The `installer.nsi` script builds a standard Windows installer (`.exe`) using
[NSIS](https://nsis.sourceforge.io/).

### Building locally

1. Install NSIS: https://nsis.sourceforge.io/Download
2. Build the binaries:
   ```
   cargo build --release -p cleansys-gui -p cleansys-tui --target x86_64-pc-windows-msvc
   ```
3. Replace `@VERSION@` and `@REPO_ROOT_ABS@` in the `.nsi` file (see
   `scripts/ci/package_windows.sh` for how CI does this)
4. Run: `makensis packaging\windows\installer.nsi`

The installer will be created at `dist\cleansys-<version>-windows-x86_64-setup.exe`.

### Path resolution (`!cd`)

NSIS resolves every relative path used by `File`, `LicenseData`,
`MUI_ICON`/`MUI_UNICON`, and similar commands relative to the *compiled
script's own directory*, not the process's working directory. In CI this
script is compiled from a versioned copy at `dist\installer_versioned.nsi`
(see `scripts/ci/package_windows.sh`), so left unhandled, relative paths
like `LICENSE` or `target\...\release\cleansys-gui.exe` would be looked up
under `dist\`, where they don't exist.

`installer.nsi` opens with `!cd "@REPO_ROOT_ABS@"`, which
`package_windows.sh` substitutes with the actual repository root
(resolved via Git Bash's `pwd -W`) before invoking `makensis`. This
re-anchors every relative path in the script to the repo root in one
place, so `LICENSE`, `packaging\windows\cleansys.ico`, and the `File`
commands all resolve exactly as if the script were compiled in place.

## Icon

The installer expects `packaging\windows\cleansys.ico` to exist at build time.
See [`cleansys.ico.txt`](cleansys.ico.txt) for instructions on providing it.

If no icon is committed, `scripts/ci/package_windows.sh` writes a small
bundled fallback icon (a pre-built, verified-valid classic 16x16 32bpp ICO,
embedded as a base64 blob and decoded with `base64 -d`) before invoking
`makensis`, so the installer build never hard-fails on a missing icon file.
Replace it with real branded artwork at any time using the same path — the
fallback is only used when the file is absent.
