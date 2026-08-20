#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# CleanSys — Linux packaging: .deb, .rpm
# ──────────────────────────────────────────────────────────────────────────────
# Usage:
#   nu scripts/ci/package_linux.nu <version> <target>
#
# Requires: dpkg-deb (for .deb), alien (for .deb -> .rpm conversion).
# Tools are installed in the CI job that calls this script.
#
# `alien` is used instead of `rpmbuild` to build the .rpm: it derives the
# package metadata directly from the already-built, already-correctly-tagged
# .deb's control file and just repackages the existing files, with no
# host/target architecture compilation-feasibility check to trip over on
# cross-architecture CI runners.
# ──────────────────────────────────────────────────────────────────────────────

# Build the argument list for the `alien` invocation used to convert one
# already-built .deb into a .rpm. Extracted into its own pure function so the
# exact shape of the args can be asserted directly in tests.
export def alien-args [deb_path: string]: nothing -> list<string> {
    ["--to-rpm" "--scripts" $deb_path]
}

# Whether a file name looks like a Linux package artifact (.deb or .rpm).
export def is-linux-package-file [name: string]: nothing -> bool {
    ($name | str ends-with ".deb") or ($name | str ends-with ".rpm")
}

# Build the DEBIAN/control file contents for the desktop GUI .deb package.
#
# Extracted into its own pure function (no side effects, no external
# commands) so its exact text can be asserted in tests.
export def gui-deb-control [version: string, arch: string]: nothing -> string {
    $"Package: cleansys-gui
Version: ($version)
Architecture: ($arch)
Maintainer: Sorin Albu-Irimies <sorinirimies@gmail.com>
Description: CleanSys GUI — desktop system cleaner for Linux \(Iced\)
 A mouse-driven desktop GUI for cleaning caches and temp files, built on Iced.
Homepage: https://github.com/sorinirimies/cleansys
Depends: libxkbcommon0, libwayland-client0, libgl1
"
}

# Build the DEBIAN/control file contents for the terminal UI + CLI .deb package.
export def tui-deb-control [version: string, arch: string]: nothing -> string {
    $"Package: cleansys
Version: ($version)
Architecture: ($arch)
Maintainer: Sorin Albu-Irimies <sorinirimies@gmail.com>
Description: CleanSys — terminal UI and CLI system cleaner for Linux
 A keyboard-driven terminal UI and CLI for cleaning caches and temp files, built on Ratatui.
Homepage: https://github.com/sorinirimies/cleansys
"
}

# Convert an already-built .deb into a .rpm using `alien`, and return the
# path to the produced .rpm file.
#
# `alien` writes its output to the current working directory rather than
# accepting an output-directory flag, so this runs it from a dedicated
# scratch directory and restores the previous working directory afterward.
export def alien-deb-to-rpm [
    deb_path: string   # e.g. dist/cleansys_0.4.0_arm64.deb
    out_dir: string    # e.g. dist/alien-rpm/tui
]: nothing -> string {
    mkdir $out_dir
    let deb_abs = ($deb_path | path expand)
    let prev_dir = (pwd)
    cd $out_dir
    run-external "alien" ...(alien-args $deb_abs)
    cd $prev_dir
    let pattern = ($out_dir + "/*.rpm" | into glob)
    (ls $pattern | first).name
}

def main [
    version: string   # e.g. 0.4.0
    target: string    # e.g. x86_64-unknown-linux-gnu
] {
    let arch = if ($target | str contains "aarch64") { "arm64" } else { "amd64" }
    let rpm_arch = if ($target | str contains "aarch64") { "aarch64" } else { "x86_64" }
    let dist_dir = "dist"

    mkdir $dist_dir

    # ── .deb for cleansys (TUI + CLI, binary name: cleansys) ────────────────
    let tui_deb_root = $"($dist_dir)/deb-tui"
    mkdir $"($tui_deb_root)/DEBIAN"
    mkdir $"($tui_deb_root)/usr/bin"
    mkdir $"($tui_deb_root)/usr/share/doc/cleansys"

    cp $"target/($target)/release/cleansys" $"($tui_deb_root)/usr/bin/cleansys"

    (tui-deb-control $version $arch) | save -f $"($tui_deb_root)/DEBIAN/control"

    $"CleanSys ($version)
Copyright 2024 Sorin Albu-Irimies
MIT License — see /usr/share/common-licenses/MIT
" | save -f $"($tui_deb_root)/usr/share/doc/cleansys/copyright"

    let tui_deb_path = $"($dist_dir)/cleansys_($version)_($arch).deb"
    run-external "dpkg-deb" "--build" $tui_deb_root $tui_deb_path
    print $"✅ Built cleansys_($version)_($arch).deb"

    # ── .deb for cleansys-gui (binary name: cleansys-gui) ───────────────────
    let gui_deb_root = $"($dist_dir)/deb-gui"
    mkdir $"($gui_deb_root)/DEBIAN"
    mkdir $"($gui_deb_root)/usr/bin"
    mkdir $"($gui_deb_root)/usr/share/doc/cleansys-gui"

    cp $"target/($target)/release/cleansys-gui" $"($gui_deb_root)/usr/bin/cleansys-gui"

    (gui-deb-control $version $arch) | save -f $"($gui_deb_root)/DEBIAN/control"

    $"CleanSys GUI ($version)
Copyright 2024 Sorin Albu-Irimies
MIT License — see /usr/share/common-licenses/MIT
" | save -f $"($gui_deb_root)/usr/share/doc/cleansys-gui/copyright"

    let gui_deb_path = $"($dist_dir)/cleansys-gui_($version)_($arch).deb"
    run-external "dpkg-deb" "--build" $gui_deb_root $gui_deb_path
    print $"✅ Built cleansys-gui_($version)_($arch).deb"

    # ── .rpm for both (via alien, from the .deb built above) ────────────────
    let tui_rpm_file = (alien-deb-to-rpm $tui_deb_path $"($dist_dir)/alien-rpm/tui")
    cp $tui_rpm_file $"($dist_dir)/cleansys-($version)-($rpm_arch).rpm"
    print $"✅ Built cleansys-($version)-($rpm_arch).rpm"

    let gui_rpm_file = (alien-deb-to-rpm $gui_deb_path $"($dist_dir)/alien-rpm/gui")
    cp $gui_rpm_file $"($dist_dir)/cleansys-gui-($version)-($rpm_arch).rpm"
    print $"✅ Built cleansys-gui-($version)-($rpm_arch).rpm"

    print ""
    print "📦 Linux packages:"
    ls $dist_dir | where { |it| is-linux-package-file ($it.name | path basename) } | select name size | print
}
