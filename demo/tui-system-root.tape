# CleanSys TUI — the user-land / root separation (system cleaners ask for your password)
# Runs against a synthetic HOME (demo/fixture.nu).

Output demo/previews/tui-system-root.gif

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

# Shift+Tab jumps to the last category: the SYSTEM · ROOT section
Shift+Tab
Sleep 1500ms
Shift+Tab
Sleep 1500ms
Down
Sleep 700ms
Down
Sleep 700ms
Space
Sleep 1200ms

# Run it: confirm, then CleanSys asks for the password (root cleaners only)
Enter
Sleep 2s
Type "y"
Sleep 2500ms
Escape
Sleep 1500ms
Type "q"
Sleep 500ms
