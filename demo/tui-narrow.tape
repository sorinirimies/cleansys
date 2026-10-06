# CleanSys TUI — responsive: a narrow terminal collapses the sidebar into a category bar

Output demo/previews/tui-narrow.gif

Set Width 700
Set Height 800
Set Shell "bash"
Set FontSize 14
Set Padding 12
Set Theme "Catppuccin Mocha"
Set TypingSpeed 45ms

Hide
Type "source demo/fixture.sh >/dev/null 2>&1 && clear"
Enter
Sleep 3s
Show

Type "./target/release/cleansys"
Enter
Sleep 5s

Tab
Sleep 1200ms
Tab
Sleep 1200ms
Down
Sleep 800ms
Down
Sleep 800ms
Space
Sleep 1200ms
Type "r"
Sleep 1500ms
Type "/"
Sleep 400ms
Type "docker"
Sleep 2s
Escape
Sleep 800ms
Type "q"
Sleep 500ms
