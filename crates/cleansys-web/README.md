# cleansys-web

Server-rendered local web UI ([Topcoat](https://github.com/tokio-rs/topcoat)) for
[CleanSys](https://github.com/sorinirimies/cleansys) — the same ~190 cleaners, live sizes,
user-land / root separation, scheduler and safety rules as the terminal UI and the desktop GUI.

```sh
cargo run -p cleansys-web                 # http://127.0.0.1:3000 (opens your browser)
cleansys-web --port 8080 --no-open
sudo cleansys-web                         # optional: no password prompt for system (root) cleaners
```

Needs **Rust 1.98+** (Topcoat). No JavaScript bundle is required — every action is a plain HTML
form, and pages refresh themselves while a scan or a clean is running.

| URL | |
|---|---|
| `/` | cleaners (`?cat=2&q=gradle&hide=0&theme=Nord`) |
| `/auth` | sudo password prompt (loopback binds only) |
| `/preview` · `/confirm` · `/progress` | dry-run, confirmation, live progress of a clean |
| `/schedule` | automatic cleaning (systemd/cron, launchd, Task Scheduler) |
| `/api/categories` · `/api/status` · `/api/themes` · `/api/health` | JSON |

**Details & selection.** Each cleaner has a *▸ Details* link that lists every path it would remove
(with sizes) and a checkbox per path, plus *Select all* / *Select none* (`?open=<cleaner-id>` keeps it
expanded). Unticked paths are skipped by both Preview and Clean. The **Idle ≥** drop-down in the action
bar sets how long a project must be untouched before its build output (`target/`, `node_modules`, …)
is offered; it is saved to `engine.json` and triggers a re-scan.

Responsive on the same breakpoints as the GUI and TUI: ≥ 980 px sidebar, ≥ 700 px narrower sidebar,
below that a category drop-down and icon-only buttons.

**Safety.** Binds to localhost by default (warns loudly otherwise), rejects requests whose `Host` is
not the bound address (DNS-rebinding), relies on Topcoat's built-in cross-origin protection for the
state-changing routes, only redirects to same-site paths, and uses the same protected-path rules,
`caution` handling and "skip running apps" guard as every other front-end.
