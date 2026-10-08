# CleanSys TUI — settings (o) and about (i): idle days, scan depth, switches, folders, exclusions, version, GitHub
# Runs against the synthetic demo HOME (demo/fixture.nu).

Output demo/previews/tui-settings.gif

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

# o = settings: ←/→ change a value (idle days, scan depth), Space flips a switch
Type "o"
Sleep 1500ms
Screenshot demo/target/tui-settings-1.png
Right
Sleep 900ms
Right
Sleep 900ms
Down
Sleep 500ms
Right
Sleep 900ms
Down
Sleep 500ms
Space
Sleep 900ms

# project folders and "never delete" patterns: Enter on ＋ to type, d to remove
Down
Down
Down
Sleep 400ms
Enter
Sleep 400ms
Type "~/Projects/rust-api"
Sleep 800ms
Enter
Sleep 1200ms
Down
Down
Enter
Sleep 400ms
Type "~/Projects/keep-me/**"
Sleep 800ms
Enter
Sleep 1500ms
Screenshot demo/target/tui-settings-2.png

# Tab = About: version, developer, GitHub (Enter opens a link)
Tab
Sleep 1200ms
Down
Down
Sleep 800ms
Screenshot demo/target/tui-about.png
Sleep 1500ms
Escape
Sleep 800ms
Type "q"
Sleep 500ms
