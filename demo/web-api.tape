# CleanSys web — start the server, ask it for status and the cleaner list (JSON API)
# Runs against the synthetic demo HOME (demo/fixture.nu).

Output demo/previews/web-api.gif

Set Shell "nu"
Set FontSize 14
Set Padding 12
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set Width 1300
Set Height 800

Hide
Type "source-env demo/fixture.nu; clear"
Enter
Sleep 3s
Show

Type "./target/release/cleansys-web --list-themes | lines | first 6"
Enter
Sleep 2500ms

Type "clear"
Enter
Type "job spawn { ./target/release/cleansys-web --port 3971 --no-open | ignore }"
Enter
Sleep 1s
Type "sleep 12sec; http get http://127.0.0.1:3971/api/status | to json"
Enter
Sleep 15s

Type "clear"
Enter
Type "http get http://127.0.0.1:3971/api/categories | get categories | select name root bytes | first 8"
Enter
Sleep 4s

Type "http get http://127.0.0.1:3971/api/categories | get categories.0.cleaners | select id name risk bytes | first 5"
Enter
Sleep 4s
