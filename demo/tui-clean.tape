# CleanSys TUI — a real clean run (on the synthetic demo HOME): pick paths, confirm, live progress in the main view, summary, rescan

Output demo/previews/tui-clean.gif

Set Shell "nu"
Set FontSize 14
Set Padding 12
Set Theme "Catppuccin Mocha"
Set TypingSpeed 45ms
Set Width 1400
Set Height 800

Hide
Type "source-env demo/fixture.nu; clear"
Enter
Sleep 3s
Show

Type "./target/release/cleansys"
Enter
Sleep 5s

# Project build output: tick Rust target/, then open it (→) and untick one project
Tab
Sleep 600ms
Tab
Sleep 1500ms
Space
Sleep 800ms
Right
Sleep 1800ms
Screenshot demo/target/tui-clean-details.png
Down
Sleep 500ms
Space
Sleep 1200ms
Left
Sleep 800ms

# ...and the JS framework caches
Down
Down
Down
Sleep 600ms
Space
Sleep 1500ms
Screenshot demo/target/tui-clean-1.png

# Run -> confirm. Progress, the outcome and the activity log stay in the main view.
Enter
Sleep 2s
Type "y"
Sleep 2500ms
Screenshot demo/target/tui-clean-2.png
Sleep 5s
Screenshot demo/target/tui-clean-3.png

# Dismiss the summary: sizes were re-measured, freed space is gone
Escape
Sleep 4s
Screenshot demo/target/tui-clean-4.png
Type "q"
Sleep 500ms
