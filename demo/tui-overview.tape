# CleanSys TUI — overview: live sizes, sidebar, recommended preset, filter, preview
# Everything runs against a synthetic HOME (demo/fixture.nu) — no real paths shown.

Output demo/previews/tui-overview.gif

Set Shell "nu"
Set FontSize 14
Set Width 1400
Set Height 800
Set Padding 12
Set Theme "Catppuccin Mocha"
Set TypingSpeed 45ms

Hide
Type "source-env demo/fixture.nu; clear"
Enter
Sleep 3s
Show

Type "./target/release/cleansys"
Enter
Sleep 4s
Screenshot demo/target/tui-1.png

# Sidebar sizes appear as the background scan finishes; user-land above, root below
Sleep 1500ms
Tab
Sleep 1200ms
Tab
Sleep 1200ms
Tab
Sleep 1500ms

# One key: tick the recommended (safe, user-land) cleaners
Type "r"
Sleep 2s

# Preview exactly what would go — deletes nothing
Type "d"
Sleep 3s
Screenshot demo/target/tui-2-preview.png
Escape
Sleep 800ms

# Search every category at once
Type "/"
Sleep 400ms
Type "gradle"
Sleep 2500ms
Screenshot demo/target/tui-3-filter.png
Escape
Sleep 1200ms

# Hide / show cleaners with nothing to clean
Type "e"
Sleep 1500ms

Type "?"
Sleep 3s
Escape
Sleep 500ms
Type "q"
Sleep 500ms
