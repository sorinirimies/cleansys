# Demo previews

These are the **committed** GIF/PNG assets referenced by the main
[README.md](../../README.md)'s Demo section — unlike `demo/target/` (where
`just vhs`/`just vhs-all` write their output by default), files in this
directory are tracked by git so they actually render on crates.io/GitHub.

## Regenerating

```bash
just vhs-all
cp demo/target/demo.gif               demo/previews/demo.gif
cp demo/target/userland-cleaners.gif  demo/previews/userland-cleaners.gif
cp demo/target/system-cleaners.gif    demo/previews/system-cleaners.gif
```

`gui.png` is a manual screenshot of `cleansys-gui` (VHS only records
terminal sessions, not native GUI windows) — retake it whenever the GUI's
layout/theme changes meaningfully and replace the file directly.

Keep these reasonably small (a few MB at most) — they're real, committed
binary blobs in git history, not build artifacts.
