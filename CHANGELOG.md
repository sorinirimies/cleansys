# Changelog

All notable changes to this project are documented here. It is generated from
[Conventional Commits](https://www.conventionalcommits.org) by
[git-cliff](https://git-cliff.org) — do not edit it by hand.

## [0.9.3](https://github.com/sorinirimies/cleansys/compare/v0.9.2...v0.9.3) — 2026-11-07

### 🐛 Bug fixes

- **tui:** Drawing a frame no longer advances the spinner; the event loop does, on its tick ([`fbd3aba`](https://github.com/sorinirimies/cleansys/commit/fbd3aba60fc554b65e806f5fa93204eb593725e0))
- **ci:** The nightly downgrade guard compares within a semver series ([`68f4c95`](https://github.com/sorinirimies/cleansys/commit/68f4c958cabd69b0c8f682d0701d3b8957564a6b))

### 🧪 Tests

- **web:** The preview test expects the platform's path separator ([`db733f5`](https://github.com/sorinirimies/cleansys/commit/db733f5514e4f08195a3d5142176e997f7bd4e8d))
- **core:** The root check test holds on an elevated Windows runner ([`9e9e760`](https://github.com/sorinirimies/cleansys/commit/9e9e7607fb63daf06ccc163d4c31522f06734983))
- **core:** Compare artifact paths as paths, not strings with /, so the test holds on Windows ([`9ac3651`](https://github.com/sorinirimies/cleansys/commit/9ac365104be15fd93903684cfb6754ecbdd787be))
- **tui:** Render tests size the app to their screen; the layout followed the real terminal (Windows CI) ([`96c355d`](https://github.com/sorinirimies/cleansys/commit/96c355d1a6f71c258cd2887bd16e398b9e7e0dcf))
- The list test expects the system cleaners of the platform it runs on (Windows has no System Logs) ([`5662aeb`](https://github.com/sorinirimies/cleansys/commit/5662aebb88309c2ba6fdcdf394cfe5e68a5c2d74))

### 📦 Dependencies

- Nightly cargo update 2026-11-07 ([`937a375`](https://github.com/sorinirimies/cleansys/commit/937a375291b328d11d45f1845c3ae450383f88af))
- Nightly cargo update 2026-11-06 ([`704e7be`](https://github.com/sorinirimies/cleansys/commit/704e7beada3a16ac74d47503f7279de5675ebe86))

### 🔄 CI and release

- **gitea:** Check the MSRV on Gitea too, with the same command as GitHub ([`33cc5e4`](https://github.com/sorinirimies/cleansys/commit/33cc5e49db0743d222c4fa1c041366d5c0d5b674))
- **release:** Each install line in the release notes says which binary it is (GUI, TUI, web) ([`2030088`](https://github.com/sorinirimies/cleansys/commit/203008881c15b6023574e6130901872cec0f1a27))
- **packages:** .deb and .rpm package names are valid Debian names (no underscores) ([`fa06154`](https://github.com/sorinirimies/cleansys/commit/fa061544df1bdec20395476e49bffe0fde447755))
- **packages:** AppImages use the downloaded appimagetool; which also matched the script's own command ([`c3d8556`](https://github.com/sorinirimies/cleansys/commit/c3d8556f9da3bf5ef4c27a9f4d35c6e6330114cf))
- **packages:** Alien gets an absolute .deb path, so the .rpm build finds it ([`95cc873`](https://github.com/sorinirimies/cleansys/commit/95cc8733ad1e06445ca4a4a9e32dc25fd4753790))
- Download archives to a file so curl retries a dropped transfer, instead of piping into tar ([`0968aca`](https://github.com/sorinirimies/cleansys/commit/0968acaf05980a5b6eabb27d9c6825398ad6464f))
- The Dependabot sweep asks for a rebase on conflicts and keeps going after a failed merge ([`2e24c02`](https://github.com/sorinirimies/cleansys/commit/2e24c02f45849db5950508807ab41aa2e09ab2f4))
- Merge green Dependabot PRs, majors included, with a nightly sweep; nightly commits skip ignored lockfiles ([`66bee5d`](https://github.com/sorinirimies/cleansys/commit/66bee5dec893f7546d1729867fa55d3b074f1ac3))

### 🔧 Chores

- **deny:** List pin-utils (RUSTSEC-2026-0441) in deny.toml; no upgrade path ([`2651b9b`](https://github.com/sorinirimies/cleansys/commit/2651b9b2976bda12703bc0b3d20599038c39f5ac))

## [0.9.2](https://github.com/sorinirimies/cleansys/compare/v0.9.1...v0.9.2) — 2026-11-05

### ✨ Features

- **tui:** Tab cycles panes, ↑/↓ act on the focused pane, scrollable activity log, ←/→ in/out, O options panel ([`1be9f27`](https://github.com/sorinirimies/cleansys/commit/1be9f27f4436b13ccb0b583b450619fa7913ac91))

### ♻️ Refactor

- Edition 2024, MSRV 1.90 and the shared lints; Debug on every public type (CleanerFn is a newtype); unsafe only where FFI needs it; no Unix-only code on Windows ([`38d4941`](https://github.com/sorinirimies/cleansys/commit/38d4941c78eda671c0ab437cb8fbd96a030fa31d))

### 📚 Documentation

- Update README and CHANGELOG for v0.9.1 ([`deb8558`](https://github.com/sorinirimies/cleansys/commit/deb85581669f79e3ae843ea09aa65f076f21a6e4))

### 🧪 Tests

- Define main after every test ([`1473f91`](https://github.com/sorinirimies/cleansys/commit/1473f91421078b2b6b54f9a2e779f0895e6817d9))

### 📦 Dependencies

- Upgrade dependencies 2026-11-05 ([`cac232e`](https://github.com/sorinirimies/cleansys/commit/cac232e21c87fd382e9c628c2497b4edcc2ea396))
- Nightly dependency upgrade 2026-11-03 ([`fec8216`](https://github.com/sorinirimies/cleansys/commit/fec8216f5de16c4b4f42702a09f13abc902ab166))
- Nightly dependency upgrade 2026-11-02 ([`1826b03`](https://github.com/sorinirimies/cleansys/commit/1826b03e8d192b6a74c262d3056261a33f46abd4))

### 🔄 CI and release

- Create-rust-project --desktop packaging (deb/rpm, AppImage, DMG, installer and AUR from project.nu) and AUTO_RELEASE nightly; Nushell-only, secrets scan, unused-dependency gate ([`103e299`](https://github.com/sorinirimies/cleansys/commit/103e2991e95b7f9ae683bc9737c99cb92aa2144e))

### 🔧 Chores

- Ignore .codegraph index ([`c83d174`](https://github.com/sorinirimies/cleansys/commit/c83d1748100cf1cbc79e02e1078d13e77819289e))

## [0.9.1](https://github.com/sorinirimies/cleansys/compare/v0.9.0...v0.9.1) — 2026-11-01

### ✨ Features

- **tui:** Report header with outcome pills, content-sized dashboard/report, sankey in GiB ([`522a174`](https://github.com/sorinirimies/cleansys/commit/522a17411b9c129162ca72b29b0ee974d8463a20))
- **tui:** Report shows freed vs expected (bullet chart) and a per-cleaner results list instead of a one-slice pie ([`3bc72d1`](https://github.com/sorinirimies/cleansys/commit/3bc72d1ae15fd9b0a9423d58a1003747c5ba7467))
- **tui:** Report sized to its content, disk before/after bar instead of the waterfall ([`90846be`](https://github.com/sorinirimies/cleansys/commit/90846be668c9e5e2ea92edbaa723700696b30a06))
- **tui:** Radial risk rings, category-to-risk sankey and a before/after waterfall (tui-chart) ([`5f8da80`](https://github.com/sorinirimies/cleansys/commit/5f8da80f5d04e29948eb6013b9d223ec87b9592a))
- **tui:** Cleaner charts - ranked bars, compact pie with legend, stacked share bar, slim gauges ([`0e13fa9`](https://github.com/sorinirimies/cleansys/commit/0e13fa9af26b5359a736d69af05345a0d7c3648d))
- **packaging:** Installable PKGBUILD from GitHub, pinned to v0.9.0 ([`9ea356c`](https://github.com/sorinirimies/cleansys/commit/9ea356c68332955a3adfe80cdba82e8f716bffb1))

### 🐛 Bug fixes

- **tui:** Drop wide emoji from the UI (misaligned borders over ssh / Linux terminals) ([`cb59370`](https://github.com/sorinirimies/cleansys/commit/cb5937004b78dbcd47b93543f0891fc211aa40d0))
- **tui:** No auto-popup report after a clean; report only after finishing; no detail charts mid-clean ([`080b89b`](https://github.com/sorinirimies/cleansys/commit/080b89bd74e5b2f613c76c94bfe1457dbb407116))
- **ci:** Build docs on stable (nightly rustc cycle error in zvariant_utils) ([`62f7882`](https://github.com/sorinirimies/cleansys/commit/62f78828bd58dd32e01cb7c13ddeb7744ffe1084))
- **ci:** Skip the AUR deploy when its secrets are not configured ([`99a9ba7`](https://github.com/sorinirimies/cleansys/commit/99a9ba7e4040e527539f269314ae3c69ce76baa7))

### 📚 Documentation

- Update README and CHANGELOG for v0.9.0 ([`967ffcb`](https://github.com/sorinirimies/cleansys/commit/967ffcb4f36f7380f75e84a928fbe8faf7bdb872))

### 🔧 Chores

- **just:** Document run aliases ([`32da07e`](https://github.com/sorinirimies/cleansys/commit/32da07e4c17813081aade7f416290c4138b3cc42))
- **just:** Add tui/gui/web aliases for the run recipes ([`cbe62c9`](https://github.com/sorinirimies/cleansys/commit/cbe62c93c089c83225c244f1e4dbc9f4d9f74891))

## [0.9.0](https://github.com/sorinirimies/cleansys/compare/v0.8.8...v0.9.0) — 2026-11-01

### ✨ Features

- **tui:** GUI-style header, card rows, sidebar and action bar ([`44792b8`](https://github.com/sorinirimies/cleansys/commit/44792b8820ed17a496dee8d4aba6fa301695378d))
- **tui:** Treemap gradients from tui-nuances theme colours ([`a9cd006`](https://github.com/sorinirimies/cleansys/commit/a9cd0069327f39d77c9a6ca6f63ea014f8ffe71c))
- **tui:** Charts and TUI/core updates (WIP from local checkout) ([`9215583`](https://github.com/sorinirimies/cleansys/commit/9215583c04e4615ab6b8a0a0da5e806bcf323d89))
- **tui:** Take theme colours from tui-nuances ([`3d0ee27`](https://github.com/sorinirimies/cleansys/commit/3d0ee278e19b3462bed237a5bb97fb26a40e21cd))

### 🐛 Bug fixes

- **tui:** Clean/scan workers no longer spin forever on a hung or panicking cleaner ([`7c6fbaa`](https://github.com/sorinirimies/cleansys/commit/7c6fbaa95300b41e149f3645ce1fedea345fedad))

### ♻️ Refactor

- **tui:** Drop UiTheme; views read tui-nuances Theme directly ([`5af6de1`](https://github.com/sorinirimies/cleansys/commit/5af6de137932adcc684e6152cecb4f6c13b0a7ff))

### 📦 Dependencies

- Nightly dependency upgrade 2026-11-01 ([`a0eccc9`](https://github.com/sorinirimies/cleansys/commit/a0eccc97f1b2e93b7c47091394945dc0bddb1c32))
- Nightly dependency upgrade 2026-10-31 ([`c16cc84`](https://github.com/sorinirimies/cleansys/commit/c16cc844e1ee5773977c4e4713ffb962ae3986b1))
- Nightly dependency upgrade 2026-10-30 ([`bee5c95`](https://github.com/sorinirimies/cleansys/commit/bee5c95c0015e4ca3c36ee98fe11205bbb960d0b))
- Nightly dependency upgrade 2026-10-29 ([`ee0e211`](https://github.com/sorinirimies/cleansys/commit/ee0e211e0aea27cb49a4a57f8fc12afbfeacb4f3))
- Nightly dependency upgrade 2026-10-28 ([`e81cf41`](https://github.com/sorinirimies/cleansys/commit/e81cf4126a274d1a7248472c28d94a7991fac651))
- Nightly dependency upgrade 2026-10-27 ([`ec92181`](https://github.com/sorinirimies/cleansys/commit/ec921817739dcd2111b0e976b0892c8d0ad9ae0e))
- Bump tui-nuances to 0.1.10 ([`fc0b71a`](https://github.com/sorinirimies/cleansys/commit/fc0b71ab372840cbe2be86abdd5424923d3f541d))
- Nightly dependency upgrade 2026-10-26 ([`288add2`](https://github.com/sorinirimies/cleansys/commit/288add20e28c708e9b6337ca6209162ec6d2a70f))

## [0.8.8](https://github.com/sorinirimies/cleansys/compare/v0.8.7...v0.8.8) — 2026-10-25

### 📚 Documentation

- Update README and CHANGELOG for v0.8.7 ([`cfef772`](https://github.com/sorinirimies/cleansys/commit/cfef77259176d2b54ecc635d9557a4fdd72b400d))

### 📦 Dependencies

- Nightly dependency upgrade 2026-10-25 ([`a80b573`](https://github.com/sorinirimies/cleansys/commit/a80b57343de03c6890ebb834a74665d77f6cfcfb))
- Nightly dependency upgrade 2026-10-24 ([`eb33b42`](https://github.com/sorinirimies/cleansys/commit/eb33b422a56786a5369ffe3f1f03f459014d2dce))
- Nightly dependency upgrade 2026-10-23 ([`0e2d041`](https://github.com/sorinirimies/cleansys/commit/0e2d041b95ec48f4c55b7ead385de7c323eacfd3))
- Nightly dependency upgrade 2026-10-22 ([`9486691`](https://github.com/sorinirimies/cleansys/commit/948669115ecc307fa9212cbae91b630d84c9e5e4))

## [0.8.7](https://github.com/sorinirimies/cleansys/compare/v0.8.6...v0.8.7) — 2026-10-21

### ✨ Features

- ~40 new system/dev cleaners; faster scans (size cache, parallel sizing, slowest-first) ([`ba782bc`](https://github.com/sorinirimies/cleansys/commit/ba782bc4ebe3d525860d7d8d40ffec229c5d9413))
- **ui:** Spinning, greyed-out Rescan button; restyle all buttons ([`16bb051`](https://github.com/sorinirimies/cleansys/commit/16bb05189a1feaa37e18cc42b4aab41c07c28ddd))

### 🐛 Bug fixes

- **test:** Judge glob breadth by full pattern, not literal prefix ([`9b920b1`](https://github.com/sorinirimies/cleansys/commit/9b920b19c10391f483d33d7f7762bb86fe6f5fbe))

### ⚡ Performance

- One shared scan pool (2x cores, configurable), deeper parallel walks, streaming sizing, parallel artifact stat; catch cleaner panics ([`ec1f3d3`](https://github.com/sorinirimies/cleansys/commit/ec1f3d323ef7f65d0e8789dda3f45b591f39d64b))

### 📚 Documentation

- Document scan_threads / CLEANSYS_SCAN_THREADS ([`ee3b366`](https://github.com/sorinirimies/cleansys/commit/ee3b3661030114b88dbfff9ee817c2143b9496aa))

### 🔄 CI and release

- Ignore zbus-split unmaintained advisories (transitive, no upgrade) ([`f29ae7b`](https://github.com/sorinirimies/cleansys/commit/f29ae7bc368bb29b861697ac0bb98742f45ab881))
- Release only when a Cargo.toml requirement changes; lock-only merges without a bump ([`fae303b`](https://github.com/sorinirimies/cleansys/commit/fae303b3e650749adfe7243ebe1cdf53b88f15c6))
- Run cargo audit + fuzz on every push and gate releases on them ([`9cb4f3e`](https://github.com/sorinirimies/cleansys/commit/9cb4f3e15574ecac4a07b327155945a9f2bf34ee))

### 💄 Style

- Highlight action buttons in GUI and web ([`962f1b7`](https://github.com/sorinirimies/cleansys/commit/962f1b7aeacab915aa57e3c6dce246ef43dadebf))

## [0.8.6](https://github.com/sorinirimies/cleansys/compare/v0.8.5...v0.8.6) — 2026-10-17

### 🐛 Bug fixes

- Details paths follow unticked parent after scan; cli --saved, --exclude, rescan, --rescan ([`580d2c9`](https://github.com/sorinirimies/cleansys/commit/580d2c952bcc0fbe236484b58459ad54ac10dfa3))

### 🔧 Chores

- Drop unneeded unsafe in macos menu; test --saved selection ([`977b401`](https://github.com/sorinirimies/cleansys/commit/977b40161ecafc161aca41910d958a4f563f6f70))

## [0.8.5](https://github.com/sorinirimies/cleansys/compare/v0.8.4...v0.8.5) — 2026-10-17

### 📦 Dependencies

- Nightly dependency upgrade 2026-10-17 ([`3897b5f`](https://github.com/sorinirimies/cleansys/commit/3897b5f05263ee0d5ea4f35143c6396e75fe7b92))

## [0.8.4](https://github.com/sorinirimies/cleansys/compare/v0.8.2...v0.8.4) — 2026-10-16

### 📦 Dependencies

- Nightly dependency upgrade 2026-10-16 ([`3cedcd7`](https://github.com/sorinirimies/cleansys/commit/3cedcd76c3eec2901a07ba0133062cfead4e32af))

### 🔄 CI and release

- Put cargo bin on PATH before rustup calls ([`23eb4ea`](https://github.com/sorinirimies/cleansys/commit/23eb4eaae3a6138796593c5a1a21a9c247b68f3a))

### 🔧 Chores

- Sync Cargo.lock ([`3908a8c`](https://github.com/sorinirimies/cleansys/commit/3908a8cc223b76676d9d53b97bc85a38e1fff88c))

## [0.8.2](https://github.com/sorinirimies/cleansys/compare/v0.8.1...v0.8.2) — 2026-10-15

### ✨ Features

- **theme:** Add System theme that follows OS light/dark across core, gui, tui and web ([`6e1edc6`](https://github.com/sorinirimies/cleansys/commit/6e1edc62d11bbf257aaa8fa37cc3effa37451205))

### 🐛 Bug fixes

- **gui:** Doc link to private fn breaks cargo doc -D warnings ([`f607402`](https://github.com/sorinirimies/cleansys/commit/f607402095c0730b0ace25ccf3dc87436d7d51e5))

### 📚 Documentation

- Shorten platform support, add web column ([`6a00e18`](https://github.com/sorinirimies/cleansys/commit/6a00e187d0bc09ad9874812c22b174a5a5d18140))
- Drop GitHub release badge ([`395d407`](https://github.com/sorinirimies/cleansys/commit/395d407d9693aad49445732517d39172a28cb472))
- Put image captions above ([`9a5cc4b`](https://github.com/sorinirimies/cleansys/commit/9a5cc4ba27e6db378b8478b7eef3d112ec8d2135))
- Caption top README images ([`1f9a926`](https://github.com/sorinirimies/cleansys/commit/1f9a926a5eae95be3d25ce194e2eb5fcccec98d2))
- Tighten README highlights, add GUI downloads and GUI preview ([`7c07384`](https://github.com/sorinirimies/cleansys/commit/7c07384c452f6ead7e8878a43ab914d014b8f4a5))
- Update README and CHANGELOG for v0.8.1 ([`412ce27`](https://github.com/sorinirimies/cleansys/commit/412ce27365b447ca9aed0e17497d8aac0f59b696))
- Update README and CHANGELOG for v0.8.0 ([`a51149f`](https://github.com/sorinirimies/cleansys/commit/a51149fd117bf56e31218ca97105ba5b4938db9c))

### 📦 Dependencies

- Nightly dependency upgrade 2026-10-15 ([`56246da`](https://github.com/sorinirimies/cleansys/commit/56246da4b9222954b166f9a335493cc95024d750))
- Nightly dependency upgrade 2026-10-14 ([`dad4191`](https://github.com/sorinirimies/cleansys/commit/dad419144a37da885eb5c69d1a3fde86f1234845))
- Nightly dependency upgrade 2026-10-13 ([`293ac9d`](https://github.com/sorinirimies/cleansys/commit/293ac9d3e7cf498836643aa3797bdf3be0636fda))
- Nightly dependency upgrade 2026-10-12 ([`6af9402`](https://github.com/sorinirimies/cleansys/commit/6af9402d47a6e6dd6bd18966a3473378f7d0bee4))

### 🔄 CI and release

- Ignore unmaintained GUI-stack advisories in deny.toml ([`7b1d2d3`](https://github.com/sorinirimies/cleansys/commit/7b1d2d3bf88a0c8dbfaf88950a90a1a70cfb52bb))
- Prebuilt Nushell, cargo-deny + cargo-audit security workflow ([`bd8b1da`](https://github.com/sorinirimies/cleansys/commit/bd8b1da1be3a586f573bd24fa9e3c5f878efe95c))
- Ignore unmaintained GUI-stack advisories in deny.toml ([`99b2a51`](https://github.com/sorinirimies/cleansys/commit/99b2a51577bbe07094a9964302ee743b92b3af4b))
- Prebuilt Nushell, cargo-deny + cargo-audit security workflow ([`15ce28a`](https://github.com/sorinirimies/cleansys/commit/15ce28a5ad2c15f56ac60eb87b5a725303525aad))
- Split fuzz into own workflow; add audit & fuzz badges to README ([`5428829`](https://github.com/sorinirimies/cleansys/commit/5428829ba56f828283801b7e3e5074cdbf2423e9))
- Add cargo-fuzz targets + fuzz smoke job; fix audit/fuzz findings ([`dbf71d0`](https://github.com/sorinirimies/cleansys/commit/dbf71d0f00facff2a7476b5335f3c5b6f5f2d2a1))
- **security:** Add cargo-audit workflow and just audit recipe ([`68966d7`](https://github.com/sorinirimies/cleansys/commit/68966d78b931df54e3e4e621849eecaa87b86ecd))

### 🔧 Chores

- Refresh stale Cargo.lock ([`faabb46`](https://github.com/sorinirimies/cleansys/commit/faabb464c7de48722f5c0b9a74ecfed87b2c6842))
- **justfile:** Push/pull current branch instead of hardcoded main ([`2b2f00e`](https://github.com/sorinirimies/cleansys/commit/2b2f00e57efd18ec79cd6ed9be6ca4f93b7561f3))
- **justfile:** Skip unconfigured remotes in *-all recipes ([`e6726c8`](https://github.com/sorinirimies/cleansys/commit/e6726c82547ca8471aedc0ee821403830cedf41d))

## [0.8.0](https://github.com/sorinirimies/cleansys/compare/v0.7.9...v0.8.0) — 2026-10-09

### ✨ Features

- **gui:** Activity panel fills the width, resizable by dragging its top edge, Clear lit only when there is content ([`14becd9`](https://github.com/sorinirimies/cleansys/commit/14becd973889fd169c5e6d72c61a3ed4b1ccc606))

### 🐛 Bug fixes

- **gui:** Sudo authentication no longer pins the header badge to ROOT; remove dead code (unused settings fns, icon, message); regression tests ([`c1bcbab`](https://github.com/sorinirimies/cleansys/commit/c1bcbabb8e8edc0a5202da1e4763d24af9ea9bb1))

## [0.7.9](https://github.com/sorinirimies/cleansys/compare/v0.7.8...v0.7.9) — 2026-10-09

### ✨ Features

- **gui:** Round progress ring per cleaner/category while measuring, and on progress labels (canvas); shared ring_arc maths ([`0cd7c70`](https://github.com/sorinirimies/cleansys/commit/0cd7c70b8f8429e31ae3a7783291671c83eee324))
- Same animated progress in TUI, GUI and web — spinner plus a bar with a sweeping highlight (shared cleansys_core::anim) ([`513c1ff`](https://github.com/sorinirimies/cleansys/commit/513c1ffeb0f44763cadbf33925b4f2a14d556278))
- **gui:** Window resizer — presets, grow/shrink, maximise, full screen shortcuts and settings; remember the window size ([`fba0a71`](https://github.com/sorinirimies/cleansys/commit/fba0a7114513a9ed983a53643cab0686139eb751))
- Settings and About in the TUI, GUI and web UI; version always visible; engine/preferences editable in-app ([`cbb3d46`](https://github.com/sorinirimies/cleansys/commit/cbb3d4635b181756fd099f3dadd9e44b3f8cef70))
- **tui:** Show clean progress, outcome and activity log in the main view (worker thread); remove the detailed progress screen ([`d4d14a1`](https://github.com/sorinirimies/cleansys/commit/d4d14a1de8fd70821c52e775a95d1a4f0b3710a3))
- Sudo prompt in web UI; idle-day selector and expandable per-path selection in TUI/GUI/web ([`771e0bc`](https://github.com/sorinirimies/cleansys/commit/771e0bcd85eddf039eb0803ece7657b69939b72c))

### 🐛 Bug fixes

- **gui,macos:** Install the standard Window menu so the system's tiling/zoom shortcuts work (as in tokenburn) ([`a3c0056`](https://github.com/sorinirimies/cleansys/commit/a3c0056a5bca7f213a1e707a2050dc277777bc1e))
- **test,core:** Make the web details test independent of the shared sandbox project; write engine.json atomically ([`d529bd2`](https://github.com/sorinirimies/cleansys/commit/d529bd243b48de50d47e65dceee7421082626124))

### 📚 Documentation

- Re-record TUI tapes and web screenshots for the in-view progress, details and idle-days; silence cleaner output in the TUI; wider footer ([`445eca8`](https://github.com/sorinirimies/cleansys/commit/445eca82781ddb46446890857e0ae2543a7408b6))
- Document details expansion, per-path selection and idle-days selector ([`9bffe7e`](https://github.com/sorinirimies/cleansys/commit/9bffe7ed14b6d11d0c211cb942f2768e7d5802bb))

### 🧪 Tests

- **tui:** Cover the clean worker, progress/summary state and run keys; tidy logging and lints ([`bc43b51`](https://github.com/sorinirimies/cleansys/commit/bc43b5198f6c2755929f3552c62438945215838e))

## [0.7.6](https://github.com/sorinirimies/cleansys/compare/v0.7.7...v0.7.6) — 2026-10-07

### 🐛 Bug fixes

- **core:** Evict stale project index cache; wipe cached sudo password on drop ([`8258d36`](https://github.com/sorinirimies/cleansys/commit/8258d36e173cc9251d9ef0848041596f2930a350))

## [0.7.5](https://github.com/sorinirimies/cleansys/compare/v0.7.1...v0.7.5) — 2026-10-07

### ⚡ Performance

- **core:** Parallel directory deletion, parallel marker-only project index, streaming file filters ([`d099842`](https://github.com/sorinirimies/cleansys/commit/d0998427321224f9d573d7a7bae47278f7329153))
- **core:** Parallel directory sizing (rayon, first 4 levels) — Gradle cache 18s -> 4.5s, full scan 20s -> 7.8s ([`48de9e9`](https://github.com/sorinirimies/cleansys/commit/48de9e9f7f56324721c801dd45199a14f4ed57ba))

### 🧪 Tests

- **web:** Server tests use a small fake category set instead of scanning the real machine (11 parallel scans timed out on slower/busier hosts); 6s -> 0.15s ([`48779e0`](https://github.com/sorinirimies/cleansys/commit/48779e0ea1c1c04b7dd317c8459a744bf7af49da))
- **web:** Root-badge test shows empty categories (hide=0) so it passes on Linux CI ([`f9c4a14`](https://github.com/sorinirimies/cleansys/commit/f9c4a14838de6f01a5f558f8b77c8be73f09cd5a))

### 🔄 CI and release

- Ship cleansys-web with the rest — crates.io publish, raw binaries, Windows zip, .deb/.rpm; check_publish covers it ([`52dc773`](https://github.com/sorinirimies/cleansys/commit/52dc77373988ec6b9f393934f42e352f8b27dec3))
- Fix run 2098 — tests no longer assume non-root (CI runs as root), rustdoc private link, retry apt downloads ([`a69f3d7`](https://github.com/sorinirimies/cleansys/commit/a69f3d7ee64b9bbdddad03f5bed4e6a7dd9c56c8))
- Weekly + manual prune of old release assets (GitHub and Gitea) ([`819398e`](https://github.com/sorinirimies/cleansys/commit/819398eea6c08c743fa34a50f507c9f473b01e39))
- **gitea:** Checkout@v4 in prune job ([`6100c44`](https://github.com/sorinirimies/cleansys/commit/6100c445d91b14ae82f653f3be15d2035658da15))
- Keep only the latest release's uploaded assets ([`18936f5`](https://github.com/sorinirimies/cleansys/commit/18936f5f3e26d822193bc1f11f612acb8516063c))

## [0.7.0](https://github.com/sorinirimies/cleansys/compare/v0.6.22...v0.7.0) — 2026-10-06

### ✨ Features

- **web:** Cleansys-web (Topcoat) + nushell-only tooling, new tapes/GIFs/screenshots (git-lfs) ([`b432dda`](https://github.com/sorinirimies/cleansys/commit/b432dda3794a2a870654c818c2a7a743663dcbaa))
- **tui:** Responsive sidebar layout, live sizes, filter, hide-empty; shared ScanBoard in core ([`13546e2`](https://github.com/sorinirimies/cleansys/commit/13546e224214a0d6a845500322db9af1b6bd6cc3))
- **gui:** Responsive redesign — sidebar nav, background scan with sizes, search, action bar ([`30c0904`](https://github.com/sorinirimies/cleansys/commit/30c090468eb5560387b64da151507a7d565ac852))
- Schedule (systemd/cron/launchd/schtasks), auto command, process guard, recommended preset ([`64134c9`](https://github.com/sorinirimies/cleansys/commit/64134c978cca490475e1dc7b2cbce5dc34714daf))
- **engine:** Command action, sudo deletes, and ~130 more cleaners ([`6fd82f7`](https://github.com/sorinirimies/cleansys/commit/6fd82f7f96462d14b407630a1e36845b33ec07e9))
- **core:** Declarative cleaner engine with dev, build-artifact and AI/LLM cleaners ([`18a5dd2`](https://github.com/sorinirimies/cleansys/commit/18a5dd2a105b13cf07cb0cf9ab33ac3dea54fd43))

### 🐛 Bug fixes

- **gui:** Stop macOS 'Choose Application' dialog after cleaning ([`05d036b`](https://github.com/sorinirimies/cleansys/commit/05d036bda3738a35751c220f4dc79b19e012562b))

### 📚 Documentation

- Responsive redesign docs, new tapes/GIFs/screenshots (git-lfs), synthetic demo fixture ([`2074f67`](https://github.com/sorinirimies/cleansys/commit/2074f6748526e8d16782fb91ea00741728244876))

## [0.6.22](https://github.com/sorinirimies/cleansys/compare/v0.6.21...v0.6.22) — 2026-10-06

### 📚 Documentation

- **readme:** Add TUI and GUI crates.io download badges ([`3528e72`](https://github.com/sorinirimies/cleansys/commit/3528e727f73c277c1095d9906048a4d8b186d4a1))

### 📦 Dependencies

- Nightly dependency upgrade 2026-10-06 ([`e5d5c28`](https://github.com/sorinirimies/cleansys/commit/e5d5c28aa87877c0bc80359bceb6ca59c9244db6))
- Nightly dependency upgrade 2026-10-05 ([`d0d8e9c`](https://github.com/sorinirimies/cleansys/commit/d0d8e9c8ee7e3a922f3eeb47c43b76201843b8f2))

### 🔄 CI and release

- **gitea:** Retry rustup install on flaky network ([`d7a238f`](https://github.com/sorinirimies/cleansys/commit/d7a238f6cf4798c9d1ce201aa1d16eff2fde299f))
- Only cut patch release when a direct dependency changes ([`4942ddb`](https://github.com/sorinirimies/cleansys/commit/4942ddb16583c24c7d805ddcad8831308401a996))

## [0.6.21](https://github.com/sorinirimies/cleansys/compare/v0.6.20...v0.6.21) — 2026-10-04

### 📦 Dependencies

- Nightly dependency upgrade 2026-10-04 ([`7d62c47`](https://github.com/sorinirimies/cleansys/commit/7d62c478aa4f41160794921770a62227a7cfc3c8))

## [0.6.20](https://github.com/sorinirimies/cleansys/compare/v0.6.19...v0.6.20) — 2026-10-03

### 📦 Dependencies

- Nightly dependency upgrade 2026-10-03 ([`fe79135`](https://github.com/sorinirimies/cleansys/commit/fe7913585d4b3856ab7b7d40a3518787b8bb98b8))

## [0.6.19](https://github.com/sorinirimies/cleansys/compare/v0.6.18...v0.6.19) — 2026-10-02

### 📦 Dependencies

- Nightly dependency upgrade 2026-10-02 ([`8ec353e`](https://github.com/sorinirimies/cleansys/commit/8ec353e2129c7207117a8af643d2ac9ef2eff0a0))

## [0.6.18](https://github.com/sorinirimies/cleansys/compare/v0.6.17...v0.6.18) — 2026-10-01

### 📦 Dependencies

- Nightly dependency upgrade 2026-10-01 ([`7b573c9`](https://github.com/sorinirimies/cleansys/commit/7b573c92a7a2eb0f89403ada38150bc95798968d))

## [0.6.17](https://github.com/sorinirimies/cleansys/compare/v0.6.16...v0.6.17) — 2026-09-28

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-28 ([`1e5d1b8`](https://github.com/sorinirimies/cleansys/commit/1e5d1b8df2ba47cad1d8224978040502f7055744))

## [0.6.16](https://github.com/sorinirimies/cleansys/compare/v0.6.15...v0.6.16) — 2026-09-26

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-26 ([`594c432`](https://github.com/sorinirimies/cleansys/commit/594c432db98a78a771eba218875dc08eb417b31a))

## [0.6.15](https://github.com/sorinirimies/cleansys/compare/v0.6.12...v0.6.15) — 2026-09-25

### 🐛 Bug fixes

- **ci:** Disable Gitea nightly deps-update schedule to prevent dual auto-release ([`46e35fe`](https://github.com/sorinirimies/cleansys/commit/46e35fe7e33b419b42016eafc7123884c5eb3dcd))

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-25 ([`648f0a0`](https://github.com/sorinirimies/cleansys/commit/648f0a08c5ebbb73f5992ba90b6fc44d88027796))

## [0.6.12](https://github.com/sorinirimies/cleansys/compare/v0.6.13...v0.6.12) — 2026-09-24

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-24 ([`4a6a342`](https://github.com/sorinirimies/cleansys/commit/4a6a3420e0c9a2923f81737a821faf4c227c6240))

## [0.6.13](https://github.com/sorinirimies/cleansys/compare/v0.6.11...v0.6.13) — 2026-09-23

### 📚 Documentation

- Update README and CHANGELOG for v0.6.8 ([`b01c7d2`](https://github.com/sorinirimies/cleansys/commit/b01c7d2baaac183d91494902a93f6fe03f83f962))

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-23 ([`e237fb6`](https://github.com/sorinirimies/cleansys/commit/e237fb637b2ac5f5276ff015868af587a419bee3))
- Nightly dependency upgrade 2026-09-22 ([`4a5ebf7`](https://github.com/sorinirimies/cleansys/commit/4a5ebf78e94fe67da2e6be1618af9f5029fc9ca7))
- Nightly dependency upgrade 2026-09-20 ([`eb1a2e8`](https://github.com/sorinirimies/cleansys/commit/eb1a2e818b0b9e8126661aa491ec656b279584b6))
- Nightly dependency upgrade 2026-09-19 ([`2e13b55`](https://github.com/sorinirimies/cleansys/commit/2e13b551da5ff88fc3e32958e040885955edce95))
- Nightly dependency upgrade 2026-09-17 ([`dffd1db`](https://github.com/sorinirimies/cleansys/commit/dffd1dbf023b8d017f91e514f93a79c68aa3fef0))

## [0.6.11](https://github.com/sorinirimies/cleansys/compare/v0.6.10...v0.6.11) — 2026-09-23

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-23 ([`6a201af`](https://github.com/sorinirimies/cleansys/commit/6a201af6a81061e145394ec4169da428abd5aec7))

## [0.6.10](https://github.com/sorinirimies/cleansys/compare/v0.6.9...v0.6.10) — 2026-09-22

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-22 ([`cd4e582`](https://github.com/sorinirimies/cleansys/commit/cd4e58294ec968ccd57503da9558dd0e32034504))

## [0.6.9](https://github.com/sorinirimies/cleansys/compare/v0.6.8...v0.6.9) — 2026-09-19

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-19 ([`f7f1aa4`](https://github.com/sorinirimies/cleansys/commit/f7f1aa4642ae953f01df6c1e73822295063217d4))
- Nightly dependency upgrade 2026-09-17 ([`9c67b95`](https://github.com/sorinirimies/cleansys/commit/9c67b950f197aead781d897d170ff44d23076ca7))

## [0.6.8](https://github.com/sorinirimies/cleansys/compare/v0.6.7...v0.6.8) — 2026-09-16

### 🐛 Bug fixes

- **gui:** Activity log (and whole main screen) didn't stretch full width on resize ([`f225831`](https://github.com/sorinirimies/cleansys/commit/f225831ef7c4c1abfb60f0e27f791e09b2689131))
- **docs:** Repair corrupted README demo preview + commit real GIF/PNG assets ([`60a3812`](https://github.com/sorinirimies/cleansys/commit/60a3812ca3048f5f9e869478791ceadb88b850e6))

### 📚 Documentation

- Add real, committed demo previews (GIFs + GUI screenshot) ([`ea37c67`](https://github.com/sorinirimies/cleansys/commit/ea37c6777f55e5b9154fcb722c5435f817684f8f))
- Update README and CHANGELOG for v0.6.7 ([`77ed984`](https://github.com/sorinirimies/cleansys/commit/77ed984a4cc71dd331968e180c0259204c5af55e))

### 🧪 Tests

- **tui:** Add render.rs smoke-test suite; cleanup: finish DRY'ing system_cleaners.rs sudo call sites ([`e2a90e3`](https://github.com/sorinirimies/cleansys/commit/e2a90e381614578c8fbf03d4ee6217722fea27b3))

## [0.6.7](https://github.com/sorinirimies/cleansys/compare/v0.6.6...v0.6.7) — 2026-09-15

### 📚 Documentation

- Update README and CHANGELOG for v0.6.6 ([`a21d396`](https://github.com/sorinirimies/cleansys/commit/a21d396edb1614bb7125235ee3c023d9132f622a))
- Update README and CHANGELOG for v0.6.5 ([`6d0fe8f`](https://github.com/sorinirimies/cleansys/commit/6d0fe8fed5211a56df18d6d30f588b47f8e5de0d))
- Update README and CHANGELOG for v0.6.4 ([`d57049b`](https://github.com/sorinirimies/cleansys/commit/d57049b0b31281b3a40d1f68c1a77f2b426754aa))

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-15 ([`d7d81ba`](https://github.com/sorinirimies/cleansys/commit/d7d81ba7be1ad6a0c0be469af2a3edc055cf62b9))

## [0.6.6](https://github.com/sorinirimies/cleansys/compare/v0.6.5...v0.6.6) — 2026-09-14

### ♻️ Refactor

- **core:** Extract cleaner!/run_sudo_step macros+helper, fix silent error swallowing, docs ([`3ec5442`](https://github.com/sorinirimies/cleansys/commit/3ec5442712d6696a485b561d4f1155e14d0e859d))

## [0.6.5](https://github.com/sorinirimies/cleansys/compare/v0.6.4...v0.6.5) — 2026-09-14

### 🐛 Bug fixes

- **demo:** Demo.tape used bare 'cargo run', broken since the cleansys-tui->cleansys package rename ([`3f68d23`](https://github.com/sorinirimies/cleansys/commit/3f68d2347bc7455646548127592a25452fb7d2f2))
- Deep-analysis pass -- TUI/GUI bugs, dedupe, panic hardening ([`d3a5e4d`](https://github.com/sorinirimies/cleansys/commit/d3a5e4d831a866a50028687149d9310c4b890eff))
- **gui:** System cleaners silently no-op after password entry ([`2bc5489`](https://github.com/sorinirimies/cleansys/commit/2bc5489b8baab1c3049fcf48c3a0ad7ff2edd105))

### 📚 Documentation

- Regenerate CHANGELOG.md after merging Gitea auto-releases ([`5f4d2da`](https://github.com/sorinirimies/cleansys/commit/5f4d2da8e9c261fcd28b32e955ea32692c6b7d5b))
- Regenerate CHANGELOG.md with correct per-version sections ([`5eb57ee`](https://github.com/sorinirimies/cleansys/commit/5eb57eef7b342184ffbfb8c87f2a4f0f734d8dbd))

## [0.6.4](https://github.com/sorinirimies/cleansys/compare/v0.6.3...v0.6.4) — 2026-09-14

### 📚 Documentation

- Update README and CHANGELOG for v0.6.2 ([`d12f66f`](https://github.com/sorinirimies/cleansys/commit/d12f66f4e8680a217c2cd54b59a6f105b763bd94))

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-14 ([`404bddf`](https://github.com/sorinirimies/cleansys/commit/404bddf49fcb97031f2c5429f6eb7aa8868b34e1))
- Nightly dependency upgrade 2026-09-13 ([`aee3f97`](https://github.com/sorinirimies/cleansys/commit/aee3f97ffee1e8920f5555951d4ac58e07616ef8))

## [0.6.3](https://github.com/sorinirimies/cleansys/compare/v0.6.2...v0.6.3) — 2026-09-14

### 📦 Dependencies

- Nightly dependency upgrade 2026-09-14 ([`48253e4`](https://github.com/sorinirimies/cleansys/commit/48253e46589bdef3ab7c5b258ef959a649f58bed))

## [0.6.2](https://github.com/sorinirimies/cleansys/compare/v0.6.1...v0.6.2) — 2026-09-12

### 🐛 Bug fixes

- **ci:** Install aarch64 cross-libc + alien/fakeroot for Gitea Linux release job ([`232fcb9`](https://github.com/sorinirimies/cleansys/commit/232fcb9abe58e3461fd35f4e6d483e95b0268604))

## [0.6.0](https://github.com/sorinirimies/cleansys/compare/v0.5.1...v0.6.0) — 2026-09-11

### 📚 Documentation

- Update README and CHANGELOG for v0.5.1 ([`85e69b2`](https://github.com/sorinirimies/cleansys/commit/85e69b2088cee40f81260644b1fccb38ac3ccd50))

## [0.5.1](https://github.com/sorinirimies/cleansys/compare/v0.5.0...v0.5.1) — 2026-08-22

### 🐛 Bug fixes

- **ci:** Fix macOS DMG build failure blocking the v0.5.0 release ([`0174f10`](https://github.com/sorinirimies/cleansys/commit/0174f10826e132fda5c6aa963b718ae8137cd9b9))

### ♻️ Refactor

- **tui:** Use tui-piechart directly, drop the redundant local wrapper ([`c87b880`](https://github.com/sorinirimies/cleansys/commit/c87b88047e149ae62712e4d5099f8150e83fb749))

### 📚 Documentation

- Update README and CHANGELOG for v0.5.0 ([`cbf220d`](https://github.com/sorinirimies/cleansys/commit/cbf220d0c9aee58b33c08651fbdebef1c634780f))

## [0.5.0](https://github.com/sorinirimies/cleansys/compare/v0.2.6...v0.5.0) — 2026-08-22

### ✨ Features

- **tui:** Add tui-spinner animated loading indicator; fix flaky GUI tests ([`131ff3e`](https://github.com/sorinirimies/cleansys/commit/131ff3e85f9681ff28aac0c38ad058ec622a2e3c))
- **ci:** Nightly dependency upgrade now auto-cuts a patch release ([`c2b31db`](https://github.com/sorinirimies/cleansys/commit/c2b31dba9085fc3b72a7dc30a847d89d64c9e28c))
- Safety, dry-run preview, Windows elevation, macOS dev cleaners, and more ([`b1d5b80`](https://github.com/sorinirimies/cleansys/commit/b1d5b80f6d2557a6ec18498c74a11274d47daa23))
- **gui:** Add complete 43-theme selector, matching GitKraft's theme system ([`d239670`](https://github.com/sorinirimies/cleansys/commit/d23967012fb6e50f34ec43bf103fa92585ebaad4))
- **gui:** Redesign layout with tabs/icons, cross-platform support, packaging, tests ([`8de7cdb`](https://github.com/sorinirimies/cleansys/commit/8de7cdbc88cedfd0122e9f30c1bffa028ff81fcc))
- Restructure into Cargo workspace, add Iced GUI, migrate scripts to Nushell ([`c7eed64`](https://github.com/sorinirimies/cleansys/commit/c7eed6482b04c7456122baf314448992f17289d0))

### 🐛 Bug fixes

- **cleaners:** Real cross-platform cleaning with structured, measured results ([`7925505`](https://github.com/sorinirimies/cleansys/commit/792550541bc176dd457ae0aaa68072c997673018))

### 📚 Documentation

- Update README and CHANGELOG for v0.2.6 ([`035b739`](https://github.com/sorinirimies/cleansys/commit/035b7390a5b93ffe465e5f339b5156f998d96769))

### 📦 Dependencies

- Update to latest crate versions, drop dead dependencies ([`f016596`](https://github.com/sorinirimies/cleansys/commit/f016596be192aa5e1b4616cfc9666c86bae2b923))

### 🔄 CI and release

- Fix git-cliff installation by adding Rust toolchain setup ([`1c324b1`](https://github.com/sorinirimies/cleansys/commit/1c324b1dbc3596d07ae2d36a446e8a7395ff3f60))

### 🔧 Chores

- Add gitea-nexus-lab remote + matching justfile recipes ([`573fd40`](https://github.com/sorinirimies/cleansys/commit/573fd406e67e6e2ebdc2d71290f207a714a8b7f1))

## [0.2.6](https://github.com/sorinirimies/cleansys/compare/v0.2.5...v0.2.6) — 2025-11-02

### ✨ Features

- Add password prompt, fix legend distribution, update categories ([`61f147e`](https://github.com/sorinirimies/cleansys/commit/61f147e72787c25592ef2099cf34ab694d73f3fa))

### 📚 Documentation

- Update README and CHANGELOG for v0.2.5 ([`b6550e4`](https://github.com/sorinirimies/cleansys/commit/b6550e4e68cf969873d6329dd46e7cd7497db227))

## [0.2.4](https://github.com/sorinirimies/cleansys/compare/v0.2.3...v0.2.4) — 2025-11-02

### 📚 Documentation

- Update README and CHANGELOG for v0.2.3 ([`8ac783d`](https://github.com/sorinirimies/cleansys/commit/8ac783df9ee5e1140022afa5b221cba0340cc7f1))

## [0.2.3](https://github.com/sorinirimies/cleansys/compare/v0.2.2...v0.2.3) — 2025-10-31

### 📚 Documentation

- Update README and CHANGELOG for v0.2.2 ([`10eb6e9`](https://github.com/sorinirimies/cleansys/commit/10eb6e9e5c11061f64ebb01b8c70fe22327b83d4))

## [0.2.2](https://github.com/sorinirimies/cleansys/compare/v0.2.1...v0.2.2) — 2025-10-30

### 📚 Documentation

- Update README and CHANGELOG for v0.2.1 ([`47fb751`](https://github.com/sorinirimies/cleansys/commit/47fb7514ecd678f7e854a4dce8d055e3a2850bad))

## [0.2.1](https://github.com/sorinirimies/cleansys/compare/v0.0.11...v0.2.1) — 2025-10-15

### 📚 Documentation

- Update README and CHANGELOG for v0.0.11 ([`aa6bd53`](https://github.com/sorinirimies/cleansys/commit/aa6bd532df732c1f80708c20b415a05468694d42))

## [0.0.11](https://github.com/sorinirimies/cleansys/compare/v0.0.10...v0.0.11) — 2025-10-15

### 📚 Documentation

- Update README and CHANGELOG for v0.0.10 ([`6d614da`](https://github.com/sorinirimies/cleansys/commit/6d614daa03ae0b77630f47b2d914e4eb89fe92d2))

## [0.0.10](https://github.com/sorinirimies/cleansys/compare/v0.0.9...v0.0.10) — 2025-10-06

### 🐛 Bug fixes

- Use default GITHUB_TOKEN for releases with proper permissions ([`eb51aa5`](https://github.com/sorinirimies/cleansys/commit/eb51aa58d152b7281d68c1fc4e09affc9ebbdc31))

### 📚 Documentation

- Update README and CHANGELOG for v0.0.9 ([`87f8f3f`](https://github.com/sorinirimies/cleansys/commit/87f8f3f6e12390448a932ce01cf5680176d71b91))

### 🔧 Chores

- Improve release recipe with error handling ([`8f82bc1`](https://github.com/sorinirimies/cleansys/commit/8f82bc121b4eb71d55a325009f1d7458f0adfd02))

## [0.0.8](https://github.com/sorinirimies/cleansys/compare/v0.0.7...v0.0.8) — 2025-10-06

### 📚 Documentation

- Update README and CHANGELOG for v0.0.7 ([`b10566f`](https://github.com/sorinirimies/cleansys/commit/b10566fc177651e2cef78966e264f87f99b05301))

## [0.0.7](https://github.com/sorinirimies/cleansys/compare/v0.0.6...v0.0.7) — 2025-10-06

### 📚 Documentation

- Update README and CHANGELOG for v0.0.6 ([`f8995d4`](https://github.com/sorinirimies/cleansys/commit/f8995d46ac7902a5852af1d0b2a4432ebb08b035))

## [0.0.1](https://github.com/sorinirimies/cleansys/releases/tag/v0.0.1) — 2025-06-05


