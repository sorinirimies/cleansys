# CleanSys CLI — one command to scan, JSON for scripts, schedule help
# Runs against a synthetic HOME (demo/fixture.nu).

Output demo/previews/cli.gif

Set Width 1400
Set Height 900
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

Type "./target/release/cleansys auto -n | ^head -42"
Enter
Sleep 8s

Type "clear"
Enter
Type "./target/release/cleansys scan -c 'AI & LLM Caches' --include-caution"
Enter
Sleep 5s

Type "clear"
Enter
Type "./target/release/cleansys scan -i proj-rust --json | ^head -22"
Enter
Sleep 4s

Type "clear"
Enter
Type "./target/release/cleansys schedule install --help | ^head -24"
Enter
Sleep 5s
