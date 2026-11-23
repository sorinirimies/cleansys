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
Sleep 3s
Type "source-env demo/fixture.nu; clear"
Enter
Sleep 8s
Show
Sleep 1s

Type "cleansys"
Enter
Sleep 5s

# Walk down the category list into the SYSTEM · ROOT section, tick "All system"
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Sleep 700ms
Down
Sleep 1200ms
Screenshot demo/target/tui-system-root-1.png
Space
Sleep 1500ms
Right
Sleep 800ms
Screenshot demo/target/tui-system-root-2.png
Left
Sleep 800ms

# Run it: confirm, then CleanSys asks for the password (root cleaners only)
Enter
Sleep 2s
Type "y"
Sleep 2500ms
Escape
Sleep 1500ms
Type "q"
Sleep 500ms
