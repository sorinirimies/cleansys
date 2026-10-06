//! Stylesheet served at `/style.css`.
//!
//! Responsive on the same breakpoints as the GUI and the TUI:
//! **wide** ≥ 980 px (300 px sidebar) · **medium** ≥ 700 px (230 px sidebar) ·
//! **narrow** < 700 px (the sidebar becomes a drop-down, buttons shrink to icons).

pub const CSS: &str = r#"
:root { color-scheme: dark; --bg:#14161f; --panel:#1c1f2b; --line:#2b3042; --fg:#d8dcea; --fg2:#a9b1d6; --dim:#8b92aa; --accent:#7aa2f7; --ok:#9ece6a; --warn:#e0af68; --err:#f7768e; --sel:#232840; }
* { box-sizing: border-box; }
html { -webkit-text-size-adjust: 100%; }
body { overflow-x:hidden; }
main > *, .app > *, .pane > *, .rows > *, form.item, label.item, .head > * { min-width:0; }
body { margin:0; background:var(--bg); color:var(--fg); font:14px/1.5 system-ui,-apple-system,"Segoe UI",sans-serif; }
a { color:var(--accent); text-decoration:none; }
a:hover { text-decoration:underline; }
main { max-width:1280px; margin:0 auto; padding:16px 16px 0; display:flex; flex-direction:column; gap:12px; min-height:100vh; }

/* ── header ── */
header.top { display:flex; flex-wrap:wrap; align-items:center; gap:10px 14px; background:var(--panel); border:1px solid var(--line); border-radius:10px; padding:10px 14px; }
header.top h1 { margin:0; font-size:20px; color:var(--fg); white-space:nowrap; }
header.top .grow { flex:1 1 220px; min-width:0; }
header.top form.search { display:flex; gap:6px; }
input[type=search], input[type=text], select { background:var(--bg); color:var(--fg); border:1px solid var(--line); border-radius:6px; padding:7px 10px; font:inherit; min-width:0; }
input[type=search] { width:100%; }
input:focus, select:focus, button:focus-visible, a:focus-visible { outline:2px solid var(--accent); outline-offset:1px; }
.badge { display:inline-block; padding:1px 8px; border-radius:5px; font-size:11px; font-weight:600; color:#fff; background:var(--accent); line-height:1.6; vertical-align:middle; }
.badge.root { background:var(--accent); }
.badge.user { background:var(--ok); color:#10131c; }
.badge.mod { background:var(--warn); color:#10131c; }
.badge.caut { background:var(--err); color:#10131c; }
.btn, button { display:inline-flex; align-items:center; gap:6px; background:var(--panel); color:var(--fg); border:1px solid var(--line); border-radius:7px; padding:7px 12px; font:inherit; cursor:pointer; white-space:nowrap; }
.btn:hover, button:hover { border-color:var(--accent); text-decoration:none; }
.btn.primary, button.primary { background:var(--accent); color:#10131c; border-color:var(--accent); font-weight:600; }
.btn.danger, button.danger { background:var(--err); color:#10131c; border-color:var(--err); font-weight:600; }
.btn[aria-disabled=true], button:disabled { opacity:.45; cursor:not-allowed; pointer-events:none; }

/* ── layout: sidebar + list ── */
.app { display:grid; grid-template-columns:300px minmax(0,1fr); gap:14px; align-items:start; flex:1; }
nav.side { background:var(--panel); border:1px solid var(--line); border-radius:10px; padding:8px; position:sticky; top:12px; max-height:calc(100vh - 150px); overflow:auto; }
nav.side .section { padding:8px 8px 4px; color:var(--dim); font-size:11px; letter-spacing:.06em; text-transform:uppercase; }
nav.side .section small { display:block; text-transform:none; letter-spacing:0; font-size:10px; }
a.cat { display:flex; align-items:center; gap:8px; justify-content:space-between; padding:7px 10px; border-radius:7px; color:var(--fg2); }
a.cat:hover { background:var(--sel); text-decoration:none; }
a.cat.active { background:var(--sel); color:var(--fg); box-shadow:inset 3px 0 var(--accent); }
a.cat .name { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
a.cat .tick { color:var(--accent); font-size:11px; }
a.cat .sz { margin-left:auto; font-variant-numeric:tabular-nums; }
form.catpick { display:none; }

.pane { display:flex; flex-direction:column; gap:10px; min-width:0; }
.pane h2 { margin:0; font-size:20px; }
.pane .sub { color:var(--dim); font-size:12px; margin-top:-6px; }
.pane .head { display:flex; gap:10px; align-items:center; justify-content:space-between; flex-wrap:wrap; }
.pane .head form { display:flex; gap:6px; }
.banner { background:var(--sel); border:1px solid var(--accent); border-radius:8px; padding:8px 12px; }
.banner.warn { border-color:var(--warn); }
.rows { display:flex; flex-direction:column; gap:6px; }
form.item { margin:0; }
label.item { max-width:100%; display:grid; grid-template-columns:auto minmax(0,1fr) auto; gap:4px 12px; align-items:center; background:var(--panel); border:1px solid var(--line); border-radius:9px; padding:10px 12px; cursor:pointer; }
label.item:hover { border-color:var(--accent); }
label.item.sel { background:var(--sel); border-color:var(--accent); }
label.item.off { opacity:.55; cursor:not-allowed; }
label.item input[type=checkbox] { width:18px; height:18px; accent-color:var(--accent); }
.item .title { display:flex; flex-wrap:wrap; gap:6px 8px; align-items:center; font-weight:500; }
.item .desc { color:var(--dim); font-size:12px; grid-column:2; }
.item .path { color:var(--fg2); font-size:11px; grid-column:2; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.item .size { grid-column:3; grid-row:1 / span 3; text-align:right; font-variant-numeric:tabular-nums; white-space:nowrap; }
.item .size b { display:block; font-size:16px; font-weight:600; }
.item .size small { color:var(--dim); }
.size.big b { color:var(--err); }
.size.mid b { color:var(--warn); }
.size.none { color:var(--dim); }
.empty { padding:28px; text-align:center; color:var(--dim); background:var(--panel); border:1px dashed var(--line); border-radius:10px; }

/* ── action bar ── */
.bar { position:sticky; bottom:0; margin:0 -16px; padding:10px 16px 14px; background:linear-gradient(to top, var(--bg) 70%, transparent); z-index:5; }
.bar .inner { display:flex; flex-wrap:wrap; align-items:center; gap:8px 10px; background:var(--panel); border:1px solid var(--line); border-radius:10px; padding:10px 12px; }
.bar .sum { flex:1 1 220px; min-width:0; }
.bar .sum b { font-size:15px; }
.bar .sum small { display:block; color:var(--dim); }
.bar form { display:contents; }
.progress { height:6px; background:var(--line); border-radius:3px; overflow:hidden; margin-top:4px; }
.progress > span { display:block; height:100%; background:var(--accent); }
.ico { display:none; }

/* ── tables / pages ── */
.card { background:var(--panel); border:1px solid var(--line); border-radius:10px; padding:14px 16px; }
.card h2 { margin:0 0 8px; font-size:16px; }
table { width:100%; border-collapse:collapse; }
th, td { padding:6px 10px; text-align:left; border-bottom:1px solid var(--line); vertical-align:top; }
td.num, th.num { text-align:right; font-variant-numeric:tabular-nums; white-space:nowrap; }
td.path { color:var(--dim); font-size:12px; word-break:break-all; }
.ok { color:var(--ok); } .warn { color:var(--warn); } .err { color:var(--err); } .dim { color:var(--dim); }
form.stack { display:grid; grid-template-columns:140px minmax(0,1fr); gap:10px 12px; align-items:center; max-width:560px; }
form.stack .actions { grid-column:1 / -1; display:flex; gap:8px; flex-wrap:wrap; }
footer { color:var(--dim); font-size:12px; padding:6px 0 18px; }

/* ── medium ── */
@media (max-width:980px) {
  .app { grid-template-columns:230px minmax(0,1fr); }
}
/* ── narrow: no sidebar, drop-down navigation, icon buttons ── */
@media (max-width:700px) {
  main { padding:10px 10px 0; }
  .bar { margin:0 -10px; padding:8px 10px 10px; }
  .app { grid-template-columns:minmax(0,1fr); }
  nav.side { display:none; }
  form.catpick { display:block; }
  form.catpick select { width:100%; }
  .lbl { display:none; }
  .ico { display:inline; }
  label.item { grid-template-columns:auto minmax(0,1fr); }
  .item .size { grid-column:2; grid-row:auto; text-align:left; display:flex; gap:8px; align-items:baseline; }
  .item .size b { display:inline; }
  .bar .sum { flex-basis:100%; }
  .bar .primary { flex:1 1 100%; justify-content:center; }
  form.stack { grid-template-columns:minmax(0,1fr); }
}
@media (prefers-reduced-motion: no-preference) { label.item, a.cat, .btn, button { transition:border-color .15s, background .15s; } }
"#;
