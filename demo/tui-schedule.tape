# CleanSys TUI — schedule automatic cleaning (S)
# Nothing is installed in this demo: the overlay is closed without pressing Enter.

Output demo/previews/tui-schedule.gif

Set Width 1400
Set Height 800
Set Shell "nu"
Set FontSize 14
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
Sleep 5s

Type "S"
Sleep 2s

# Repeat: weekly -> monthly -> daily
Right
Sleep 1s
Left
Sleep 500ms
Left
Sleep 1200ms

# Hour 03 -> 06, minute 30 -> 40
Down
Sleep 500ms
Right
Right
Right
Sleep 1200ms
Down
Sleep 500ms
Right
Right
Sleep 1200ms

# Scope: recommended -> extended
Down
Sleep 500ms
Right
Sleep 2s

Escape
Sleep 800ms
Type "q"
Sleep 500ms
