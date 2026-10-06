# CleanSys TUI — a real clean run (on the synthetic demo HOME): confirm, progress, freed space, rescan

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

# Project build output: tick Rust target/ and the JS framework caches
Tab
Sleep 600ms
Tab
Sleep 1500ms
Space
Sleep 800ms
Down
Down
Down
Sleep 600ms
Space
Sleep 1500ms
Screenshot demo/target/tui-clean-1.png

# Run -> confirm
Enter
Sleep 2s
Type "y"
Sleep 3s
Screenshot demo/target/tui-clean-2.png
Sleep 3s

# Back to the list: sizes re-measured, freed space gone
Escape
Sleep 5s
Screenshot demo/target/tui-clean-3.png
Type "q"
Sleep 500ms
