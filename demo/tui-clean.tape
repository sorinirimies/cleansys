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
Sleep 3s
Type "source-env demo/fixture.nu; clear"
Enter
Sleep 8s
Show
Sleep 1s

Type "cleansys"
Enter
Sleep 5s

# Space on a category ticks every cleaner in it; → opens one, its top row is "All paths"
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 1200ms
Space
Sleep 1500ms
Right
Sleep 600ms
Right
Sleep 1500ms
Screenshot demo/target/tui-clean-details.png
Down
Down
Sleep 500ms
Space
Sleep 1200ms
Up
Up
Sleep 500ms
Space
Sleep 1200ms
Left
Sleep 600ms
Left
Sleep 800ms
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
