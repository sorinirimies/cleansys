# Demo previews

GIFs and screenshots embedded in the main [README](../../README.md). They are
**tracked with git-lfs** (see `.gitattributes`), so clones stay small unless you
fetch LFS objects.

| File | What it shows | Made by |
|------|---------------|---------|
| `tui-overview.gif` | live sizes, sidebar, recommended preset (`r`), preview (`d`), filter (`/`), hide-empty (`e`) | `demo/tui-overview.tape` |
| `tui-system-root.gif` | the user-land / root split and the password prompt | `demo/tui-system-root.tape` |
| `tui-schedule.gif` | the schedule overlay (`S`) | `demo/tui-schedule.tape` |
| `tui-narrow.gif` | responsive layout on a narrow terminal | `demo/tui-narrow.tape` |
| `tui-clean.gif` | a real clean run: confirm → progress → freed → rescan | `demo/tui-clean.tape` |
| `cli.gif` | `auto`, `scan --json`, `schedule` | `demo/cli.tape` |
| `gui-wide/medium/narrow.png` | the three responsive GUI layouts | `scripts/gui-screenshots.nu` |
| `gui-schedule.png` | the GUI schedule dialog | `scripts/gui-screenshots.nu` |
| `web-wide/medium/narrow.png` | the three responsive web layouts | `scripts/web-screenshots.nu` |
| `web-preview.png`, `web-schedule.png` | web dry-run and schedule pages | `scripts/web-screenshots.nu` |
| `web-flow.gif` | web: tick → confirm → progress → done | `scripts/web-screenshots.nu` |
| `web-responsive.gif` | the same page at three widths | `scripts/web-screenshots.nu` |
| `web-api.gif` | the JSON API driven from nushell | `demo/web-api.tape` |

## Everything is synthetic

All of it runs against a throw-away `HOME` created by [`demo/fixture.nu`](../fixture.nu)
(sparse files that *report* gigabytes but use no disk). No real paths,
usernames or projects appear, and nothing outside that directory is touched.

## Regenerating

```bash
just vhs-all             # all GIFs   (needs: vhs, ffmpeg, ttyd)
just gui-screenshots     # GUI PNGs   (macOS: osascript + screencapture)
just web-screenshots     # web PNGs/GIFs (headless Chromium-based browser + ffmpeg)
just vhs-refresh-previews
```

`vhs` needs a few GB of scratch space in `$TMPDIR` while recording.

Keep them small — they are real binary blobs, even with LFS.
