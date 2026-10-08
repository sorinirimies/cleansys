//! Pages and form routes.

use cleansys_core::engine::EngineConfig;
use cleansys_core::engine::headless::OutcomeStatus;
use cleansys_core::engine::schedule::{
    self, Backend, Frequency, LastRun, Schedule, Scope, WEEKDAYS,
};
use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        content::Form,
        error::{SeeOther, see_other},
        page, query_params, route,
    },
    view::{View, view},
};

use crate::{
    components::{action_bar, cat_pick, document, info_tabs, item_row, side_nav, spinner, top_bar},
    state::{LineStatus, NEEDS_AUTH, Phase, Shared, Snapshot},
    theme,
    util::{fmt, nav_href, safe_back, size_class},
};

fn shared(cx: &Cx) -> Shared {
    app_context::<Shared>(cx).clone()
}

/// Resolve `?theme=` (remembered for the GUI too) against the saved one.
fn theme_for(requested: Option<&str>) -> usize {
    let saved = theme::saved_index();
    let idx = theme::resolve(requested, saved);
    if idx != saved {
        theme::save_index(idx);
    }
    idx
}

#[query_params(error = redirect("?"))]
pub struct HomeParams {
    /// Active category index.
    pub cat: Option<usize>,
    /// Search text (filters across every category).
    pub q: Option<String>,
    /// `0` shows cleaners that have nothing to clean.
    pub hide: Option<String>,
    pub theme: Option<String>,
    /// Id of the cleaner whose details are expanded.
    pub open: Option<String>,
}

#[page("/")]
async fn home(cx: &Cx) -> Result<impl View> {
    let st = shared(cx);
    st.poll();
    let p = query_params::<HomeParams>(cx)?;
    let theme_idx = theme_for(p.theme.as_deref());
    // `?hide=0|1` wins; without it the saved preference decides (default: hide empty).
    let hide = match p.hide.as_deref() {
        Some("0") => false,
        Some(_) => true,
        None => cleansys_core::load_settings()
            .unwrap_or_default()
            .hide_empty(),
    };
    let q = p.q.clone().unwrap_or_default();

    let mut snap = st.snapshot();
    for c in &mut snap.categories {
        c.visible = st.category_visible(c.index, hide);
    }
    let mut active = p
        .cat
        .unwrap_or(0)
        .min(snap.categories.len().saturating_sub(1));
    if !snap.categories.get(active).is_some_and(|c| c.visible) {
        active = snap
            .categories
            .iter()
            .find(|c| c.visible)
            .map_or(0, |c| c.index);
    }
    let searching = !q.trim().is_empty();
    let nav = nav_href(active, &q, hide);
    let open_id = p.open.clone().filter(|o| !o.is_empty());
    // `back` keeps the expanded cleaner open across the toggle redirects.
    let back = match &open_id {
        Some(o) => format!("{nav}&open={}", crate::util::pct(o)),
        None => nav.clone(),
    };
    // The header badge shows the scope you are in: ROOT while a system category is open.
    let root_scope =
        snap.is_root || (!searching && snap.categories.get(active).is_some_and(|c| c.root));

    let rows: Vec<usize> = st
        .visible(active, &q, hide)
        .into_iter()
        .filter_map(|(c, i)| {
            snap.items
                .iter()
                .position(|it| it.category == c && it.idx == i)
        })
        .collect();
    let (title, subtitle) = if searching {
        (
            format!("Search results ({})", rows.len()),
            format!("Matching “{}” in every category", q.trim()),
        )
    } else {
        snap.categories
            .get(active)
            .map_or((String::new(), String::new()), |c| {
                (
                    c.name
                        .trim_end_matches(cleansys_core::model::ROOT_SUFFIX)
                        .to_string(),
                    c.description.clone(),
                )
            })
    };
    let refresh = if snap.scanning || snap.run.phase == Phase::Running {
        2
    } else {
        0
    };
    let banner = run_banner(&snap);

    Ok(view! {
        document(refresh: refresh, theme: theme_idx, title: "CleanSys".to_string(),
            top_bar(is_root: root_scope, q: q.clone(), cat: active, hide: hide, theme: theme_idx, back: back.clone())
            if let Some((cls, msg)) = &banner {
                <div class=(format!("banner {cls}"))>(msg.clone()) " " <a href="/progress">"View"</a></div>
            }
            <div class="app">
                side_nav(categories: &snap.categories, active: active, q: q.clone(), hide: hide, scanning: snap.scanning)
                <section class="pane">
                    cat_pick(categories: &snap.categories, active: active, hide: hide)
                    <div class="head">
                        <div>
                            <h2>(title)</h2>
                            <div class="sub">(subtitle)</div>
                        </div>
                        if !searching {
                            <form method="post" action="/select">
                                <input type="hidden" name="back" value=(back.clone())>
                                <input type="hidden" name="cat" value=(active.to_string())>
                                <button name="op" value="cat_all">"All"</button>
                                <button name="op" value="cat_none">"None"</button>
                            </form>
                        }
                    </div>
                    if rows.is_empty() {
                        <div class="empty">
                            if searching { "No cleaner matches your search." }
                            else if snap.scanning { "Scanning your system…" }
                            else { "Nothing to clean here — already tidy ✨ " <a href=(nav_href(active, &q, false))>"show empty cleaners"</a> }
                        </div>
                    } else {
                        <div class="rows">
                            for i in &rows {
                                item_row(item: &snap.items[*i], back: back.clone(), open: open_id.as_deref() == Some(snap.items[*i].id.as_str()), more_href: if open_id.as_deref() == Some(snap.items[*i].id.as_str()) { nav.clone() } else { format!("{nav}&open={}", crate::util::pct(&snap.items[*i].id)) }, show_category: if searching { snap.categories.get(snap.items[*i].category).map(|c| c.name.clone()) } else { None }, scanning_now: snap.scanning)
                            }
                        </div>
                    }
                </section>
            </div>
            action_bar(snap: &snap, back: back.clone(), cat: active, hide: hide)
            <footer>"Local only · " <a href="/schedule">"Schedule"</a> " · " <a href="/api/categories">"JSON"</a> " · " <a href="/api/status">"status"</a></footer>
        )
    })
}

/// A one-line notice about a running / finished clean.
fn run_banner(snap: &Snapshot) -> Option<(&'static str, String)> {
    match snap.run.phase {
        Phase::Running => Some((
            "",
            format!("Cleaning… {}/{}", snap.run.done, snap.run.total),
        )),
        Phase::Done => Some((
            "",
            format!(
                "✓ Last clean freed {} ({} cleaners).",
                fmt(snap.run.freed),
                snap.run.lines.len()
            ),
        )),
        Phase::Idle => None,
    }
}

// ── selection forms ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct ToggleForm {
    id: String,
    back: Option<String>,
}

#[route(POST "/toggle")]
async fn toggle(cx: &Cx, Form(f): Form<ToggleForm>) -> Result<SeeOther> {
    shared(cx).toggle(&f.id);
    Ok(see_other(safe_back(f.back.as_deref().unwrap_or("/"))))
}

#[derive(Deserialize)]
struct EntryForm {
    path: String,
    back: Option<String>,
}

#[route(POST "/toggle-entry")]
async fn toggle_entry(cx: &Cx, Form(f): Form<EntryForm>) -> Result<SeeOther> {
    shared(cx).toggle_entry(&f.path);
    Ok(see_other(safe_back(f.back.as_deref().unwrap_or("/"))))
}

#[derive(Deserialize)]
struct EntriesForm {
    id: String,
    on: String,
    back: Option<String>,
}

#[route(POST "/entries")]
async fn set_entries(cx: &Cx, Form(f): Form<EntriesForm>) -> Result<SeeOther> {
    shared(cx).set_entries(&f.id, f.on == "1");
    Ok(see_other(safe_back(f.back.as_deref().unwrap_or("/"))))
}

#[derive(Deserialize)]
struct MinAgeForm {
    days: u64,
    back: Option<String>,
}

#[route(POST "/min-age")]
async fn set_min_age(cx: &Cx, Form(f): Form<MinAgeForm>) -> Result<SeeOther> {
    // Reject absurd values; the selector only offers the fixed steps.
    let days = f.days.min(3650);
    let _ = shared(cx).set_min_age(days);
    // The old expanded view refers to a scan that was just replaced: drop `open`.
    let back = safe_back(f.back.as_deref().unwrap_or("/"));
    let back = back.split("&open=").next().unwrap_or("/").to_string();
    Ok(see_other(back))
}

#[derive(Deserialize)]
struct SelectForm {
    op: String,
    cat: Option<usize>,
    back: Option<String>,
}

#[route(POST "/select")]
async fn select(cx: &Cx, Form(f): Form<SelectForm>) -> Result<SeeOther> {
    let st = shared(cx);
    st.poll();
    match f.op.as_str() {
        "recommended" => {
            st.select_recommended();
        }
        "none" => st.select_none(),
        "rescan" => st.start_scan(),
        "cat_all" => st.set_category(f.cat.unwrap_or(0), true),
        "cat_none" => st.set_category(f.cat.unwrap_or(0), false),
        _ => {}
    }
    Ok(see_other(safe_back(f.back.as_deref().unwrap_or("/"))))
}

// ── preview / confirm / run / progress ───────────────────────────────

#[page("/preview")]
async fn preview(cx: &Cx) -> Result<impl View> {
    let st = shared(cx);
    let theme_idx = theme_for(None);
    let st2 = st.clone();
    let outcomes = tokio::task::spawn_blocking(move || st2.preview())
        .await
        .unwrap_or_default();
    let total: u64 = outcomes.iter().map(|o| o.bytes).sum();
    let shown: Vec<_> = outcomes
        .into_iter()
        .filter(|o| o.bytes > 0 || o.status != OutcomeStatus::Ok)
        .collect();
    Ok(view! {
        document(refresh: 0, theme: theme_idx, title: "Preview — CleanSys".to_string(),
            <header class="top"><h1>"🔍 Preview"</h1><span class="grow dim">"Nothing has been deleted."</span><a class="btn" href="/">"← Back"</a></header>
            <div class="card">
                <h2>"Would free " (fmt(total))</h2>
                if shown.is_empty() {
                    <p class="dim">"Nothing to clean — all selected cleaners are already empty."</p>
                }
                for o in shown.iter() {
                    <h3>(o.name.clone()) " — " <span class=(size_class(o.bytes))>(fmt(o.bytes))</span> <span class="dim">" · " (o.items.len()) " item(s)"</span></h3>
                    if let OutcomeStatus::Skipped(w) = &o.status { <p class="warn">"skipped: " (w.clone())</p> }
                    if let OutcomeStatus::Failed(w) = &o.status { <p class="err">"failed: " (w.clone())</p> }
                    <table>
                        for i in o.items.iter().take(8) {
                            <tr><td class="path">(i.path.clone())</td><td class="num">(fmt(i.bytes))</td></tr>
                        }
                        if o.items.len() > 8 {
                            <tr><td class="dim" colspan="2">"… and " (o.items.len() - 8) " more"</td></tr>
                        }
                    </table>
                }
            </div>
            <p><a class="btn primary" href="/confirm">"🧹 Continue to clean"</a></p>
        )
    })
}

#[page("/confirm")]
async fn confirm(cx: &Cx) -> Result<impl View> {
    let st = shared(cx);
    st.poll();
    let theme_idx = theme_for(None);
    let snap = st.snapshot();
    let picked: Vec<_> = snap
        .items
        .iter()
        .filter(|i| i.selected && i.selectable)
        .cloned()
        .collect();
    let risky = picked.iter().any(|i| i.risk != cleansys_core::Risk::Safe);
    let caution = picked
        .iter()
        .any(|i| i.risk == cleansys_core::Risk::Caution);
    let bytes: u64 = picked
        .iter()
        .filter_map(|i| i.scan.as_ref())
        .map(|s| s.bytes)
        .sum();
    Ok(view! {
        document(refresh: 0, theme: theme_idx, title: "Confirm — CleanSys".to_string(),
            <header class="top"><h1>"⚠ Confirm cleaning"</h1><span class="grow"></span></header>
            <div class="card">
                if picked.is_empty() {
                    <p>"Nothing is selected."</p>
                    <a class="btn" href="/">"← Back"</a>
                } else {
                    <p>"This will permanently delete files for " (picked.len()) " cleaner(s) — about " <b>(fmt(bytes))</b> ":"</p>
                    <table>
                        for i in picked.iter() {
                            <tr>
                                <td>(i.name.clone())
                                    if i.risk == cleansys_core::Risk::Moderate { " " <span class="badge mod">"MODERATE"</span> }
                                    if i.risk == cleansys_core::Risk::Caution { " " <span class="badge caut">"CAUTION"</span> }
                                </td>
                                <td class="num">(i.scan.as_ref().map(|s| fmt(s.bytes)).unwrap_or_default())</td>
                            </tr>
                        }
                    </table>
                    if caution {
                        <p class="err">"Includes CAUTION cleaners (model weights, chat history, …) — these are not recoverable."</p>
                    } else if risky {
                        <p class="warn">"Some cleaners are slow to rebuild or re-download."</p>
                    }
                    if snap.needs_auth {
                        <p class="warn">"🔒 System (root) cleaners are ticked — your sudo password is needed to run them."</p>
                        <p>
                            <a class="btn danger" href="/auth">"🔒 Enter sudo password"</a>
                            " "
                            <a class="btn" href="/preview">"🔍 Preview first"</a>
                            " "
                            <a class="btn" href="/">"Cancel"</a>
                        </p>
                    } else {
                        <form method="post" action="/run">
                            <button class="danger" type="submit">"🧹 Yes, clean now"</button>
                            " "
                            <a class="btn" href="/preview">"🔍 Preview first"</a>
                            " "
                            <a class="btn" href="/">"Cancel"</a>
                        </form>
                    }
                }
            </div>
        )
    })
}

#[route(POST "/run")]
async fn run_clean(cx: &Cx) -> Result<SeeOther> {
    match shared(cx).start_run() {
        Ok(_) => Ok(see_other("/progress")),
        Err(NEEDS_AUTH) => Ok(see_other("/auth")),
        Err(_) => Ok(see_other("/")),
    }
}

// ── sudo password ────────────────────────────────────────────────────

#[query_params(error = redirect("/auth"))]
pub struct AuthParams {
    pub msg: Option<String>,
}

#[page("/auth")]
async fn auth_page(cx: &Cx) -> Result<impl View> {
    let params = query_params::<AuthParams>(cx)?;
    let st = shared(cx);
    let theme_idx = theme_for(None);
    let snap = st.snapshot();
    Ok(view! {
        document(refresh: 0, theme: theme_idx, title: "Sudo password — CleanSys".to_string(),
            <header class="top"><h1>"🔒 Sudo password"</h1><span class="grow"></span><a class="btn" href="/">"← Back"</a></header>
            if let Some(m) = &params.msg { <div class="banner">(m.clone())</div> }
            <div class="card">
                if snap.is_root {
                    <p class="ok">"Already running as root — no password needed."</p>
                } else if !snap.can_elevate {
                    <p class="warn">"Password entry is disabled on this server (it is only offered on a loopback address)."</p>
                    <p class="dim">"Restart with " <code>"sudo cleansys-web"</code> " to run system cleaners."</p>
                } else if snap.sudo_ok {
                    <p class="ok">"Authenticated. The password is forgotten as soon as the clean finishes."</p>
                    <a class="btn primary" href="/confirm">"Continue"</a>
                } else {
                    <p>"System (root) cleaners need administrator rights. The password is checked with " <code>"sudo"</code> ", kept in memory only for this clean, and wiped afterwards."</p>
                    <form class="stack" method="post" action="/auth" autocomplete="off">
                        <label>"Password" <input type="password" name="password" autofocus=(true) required=(true) autocomplete="current-password"></label>
                        <button class="primary" type="submit">"Authenticate"</button>
                    </form>
                    <p class="dim">"Sent over plain HTTP to this machine only (127.0.0.1)."</p>
                }
            </div>
        )
    })
}

#[derive(Deserialize)]
struct AuthForm {
    password: String,
}

#[route(POST "/auth")]
async fn auth_submit(cx: &Cx, Form(f): Form<AuthForm>) -> Result<SeeOther> {
    let st = shared(cx);
    // `sudo -S -v` blocks on a child process: keep it off the async executor.
    let result = tokio::task::spawn_blocking(move || st.authenticate(&f.password))
        .await
        .unwrap_or_else(|_| Err("authentication task failed".into()));
    Ok(match result {
        Ok(true) => see_other("/confirm"),
        Ok(false) => see_other(format!(
            "/auth?msg={}",
            crate::util::pct("Incorrect password. Please try again.")
        )),
        Err(e) => see_other(format!("/auth?msg={}", crate::util::pct(&e))),
    })
}

#[page("/progress")]
async fn progress(cx: &Cx) -> Result<impl View> {
    let st = shared(cx);
    st.poll();
    let theme_idx = theme_for(None);
    let snap = st.snapshot();
    let run = snap.run.clone();
    let running = run.phase == Phase::Running;
    let pct = (run.done * 100).checked_div(run.total).unwrap_or(0);
    Ok(view! {
        document(refresh: if running { 1 } else { 0 }, theme: theme_idx, title: "Cleaning — CleanSys".to_string(),
            <header class="top">
                <h1>if running { "⏳ Cleaning…" } else if run.phase == Phase::Done { "✓ Done" } else { "Nothing running" }</h1>
                <span class="grow"></span>
                <a class="btn" href="/">"← Back"</a>
            </header>
            <div class="card">
                if run.phase == Phase::Idle {
                    <p class="dim">"No clean has been started."</p>
                } else {
                    <p>
                        if running { spinner() " " }
                        <b>(run.done) "/" (run.total)</b> " cleaners · " (pct) "% · freed " <b class="ok">(fmt(run.freed))</b>
                        if let Some(c) = &run.current { <span class="dim">" · running " (c.clone())</span> }
                    </p>
                    <div class="progress"><span style=(format!("width:{pct}%"))></span></div>
                    <table>
                        for l in &run.lines {
                            <tr>
                                <td>
                                    if l.status == LineStatus::Ok { <span class="ok">"✓"</span> }
                                    else if l.status == LineStatus::Skipped { <span class="warn">"•"</span> }
                                    else { <span class="err">"✗"</span> }
                                    " " (l.name.clone())
                                    if !l.detail.is_empty() { <span class="dim">" — " (l.detail.clone())</span> }
                                </td>
                                <td class="num">(fmt(l.bytes))</td>
                                <td class="num dim">(l.items) " item(s)"</td>
                            </tr>
                        }
                    </table>
                }
                if run.phase == Phase::Done {
                    <form method="post" action="/dismiss"><button class="primary">"Back to cleaners"</button></form>
                }
            </div>
        )
    })
}

#[route(POST "/dismiss")]
async fn dismiss(cx: &Cx) -> Result<SeeOther> {
    shared(cx).clear_run();
    Ok(see_other("/"))
}

// ── settings & about ─────────────────────────────────────────────────

#[query_params(error = redirect("/settings"))]
pub struct SettingsParams {
    pub msg: Option<String>,
}

fn settings_back(msg: &str) -> SeeOther {
    see_other(format!("/settings?msg={}", crate::util::pct(msg)))
}

#[page("/settings")]
async fn settings_page(cx: &Cx) -> Result<impl View> {
    let params = query_params::<SettingsParams>(cx)?;
    let theme_idx = theme_for(None);
    let cfg = EngineConfig::load();
    let prefs = cleansys_core::load_settings().unwrap_or_default();
    let roots = cfg.roots_for_editing();
    let auto_roots = cfg.scan_roots.is_empty();
    let ages = cleansys_core::engine::config::MIN_AGE_CHOICES;
    let depths = cleansys_core::engine::config::MAX_DEPTH_CHOICES;
    let age_listed = ages.contains(&cfg.min_age_days);
    let depth_listed = depths.contains(&cfg.max_depth);
    Ok(view! {
        document(refresh: 0, theme: theme_idx, title: "Settings — CleanSys".to_string(),
            <header class="top"><h1>"⚙ Settings"</h1><small class="ver">(cleansys_core::appinfo::version_label())</small><span class="grow"></span><a class="btn" href="/">"← Back"</a></header>
            info_tabs(active: "settings".to_string())
            if let Some(m) = &params.msg { <div class="banner">(m.clone())</div> }
            <div class="card">
                <h2>"Scanning"</h2>
                <form class="stack" method="post" action="/min-age">
                    <input type="hidden" name="back" value="/settings">
                    <label for="days">"Project idle for at least"</label>
                    <select id="days" name="days" onchange="this.form.submit()">
                        for d in ages.iter() {
                            <option value=(d.to_string()) selected=(*d == cfg.min_age_days)>(cleansys_core::engine::config::min_age_label(*d))</option>
                        }
                        if !age_listed { <option value=(cfg.min_age_days.to_string()) selected=(true)>(cleansys_core::engine::config::min_age_label(cfg.min_age_days))</option> }
                    </select>
                    <noscript><span class="actions"><button type="submit">"Save"</button></span></noscript>
                </form>
                <form class="stack" method="post" action="/settings/depth">
                    <label for="depth">"Scan depth"</label>
                    <select id="depth" name="depth" onchange="this.form.submit()">
                        for d in depths.iter() {
                            <option value=(d.to_string()) selected=(*d == cfg.max_depth)>(*d)</option>
                        }
                        if !depth_listed { <option value=(cfg.max_depth.to_string()) selected=(true)>(cfg.max_depth)</option> }
                    </select>
                    <noscript><span class="actions"><button type="submit">"Save"</button></span></noscript>
                </form>
                <p class="dim">"Build output (target/, node_modules, …) is only offered once its project has been untouched this long. Changing a value re-measures everything."</p>
            </div>
            <div class="card">
                <h2>"Behaviour"</h2>
                <form class="switch" method="post" action="/settings/prefs">
                    <label><input type="checkbox" name="hide_empty" value="1" checked=(prefs.hide_empty()) onchange="this.form.submit()"> " Hide cleaners with nothing to clean"</label>
                    <label><input type="checkbox" name="confirm" value="1" checked=(prefs.confirm_before_run()) onchange="this.form.submit()"> " Ask before cleaning (the confirmation page)"</label>
                    <noscript><button type="submit">"Save"</button></noscript>
                </form>
            </div>
            <div class="card">
                <h2>"Project folders scanned for build output"</h2>
                if auto_roots { <p class="dim">"Auto-detected (~/Projects, ~/dev, ~/src, …). Add or remove one to set your own list."</p> }
                <ul class="plain">
                    for (i, r) in roots.iter().enumerate() {
                        <li><code>(r.clone())</code>
                            <form method="post" action="/settings/root-remove" class="inline">
                                <input type="hidden" name="idx" value=(i.to_string())>
                                <button type="submit" title="Remove this folder">"Remove"</button>
                            </form>
                        </li>
                    }
                    if roots.is_empty() { <li class="dim">"No project folders found."</li> }
                </ul>
                <form class="add" method="post" action="/settings/root-add">
                    <input type="text" name="path" placeholder="Add a folder, e.g. ~/code" aria-label="Folder to scan" required=(true)>
                    <button type="submit">"Add"</button>
                </form>
            </div>
            <div class="card">
                <h2>"Never delete (glob patterns)"</h2>
                <ul class="plain">
                    for (i, p) in cfg.exclude.iter().enumerate() {
                        <li><code>(p.clone())</code>
                            <form method="post" action="/settings/exclude-remove" class="inline">
                                <input type="hidden" name="idx" value=(i.to_string())>
                                <button type="submit" title="Remove this pattern">"Remove"</button>
                            </form>
                        </li>
                    }
                    if cfg.exclude.is_empty() { <li class="dim">"Nothing excluded."</li> }
                </ul>
                <form class="add" method="post" action="/settings/exclude-add">
                    <input type="text" name="pattern" placeholder="Add a path or pattern, e.g. ~/work/keep/**" aria-label="Pattern to exclude" required=(true)>
                    <button type="submit">"Add"</button>
                </form>
            </div>
        )
    })
}

#[derive(Deserialize)]
struct DepthForm {
    depth: usize,
}

#[route(POST "/settings/depth")]
async fn settings_depth(cx: &Cx, Form(f): Form<DepthForm>) -> Result<SeeOther> {
    let depth = f.depth.clamp(1, 20);
    Ok(
        match shared(cx).edit_engine(true, |c| {
            c.max_depth = depth;
            Ok(())
        }) {
            Ok(()) => settings_back(&format!("Scan depth set to {depth}.")),
            Err(e) => settings_back(&e),
        },
    )
}

#[derive(Deserialize)]
struct PrefsForm {
    hide_empty: Option<String>,
    confirm: Option<String>,
}

#[route(POST "/settings/prefs")]
async fn settings_prefs(cx: &Cx, Form(f): Form<PrefsForm>) -> Result<SeeOther> {
    let _ = shared(cx); // (state is untouched: preferences live in settings.json)
    let hide = f.hide_empty.is_some();
    let ask = f.confirm.is_some();
    Ok(
        match cleansys_core::update_settings(|s| {
            s.hide_empty = Some(hide);
            s.confirm_before_run = Some(ask);
        }) {
            Ok(_) => settings_back("Saved."),
            Err(e) => settings_back(&format!("Could not save: {e}")),
        },
    )
}

#[derive(Deserialize)]
struct PathForm {
    path: String,
}

#[route(POST "/settings/root-add")]
async fn settings_root_add(cx: &Cx, Form(f): Form<PathForm>) -> Result<SeeOther> {
    Ok(
        match shared(cx).edit_engine(true, |c| c.add_scan_root(&f.path)) {
            Ok(()) => settings_back("Folder added — re-measuring."),
            Err(e) => settings_back(&e),
        },
    )
}

#[derive(Deserialize)]
struct PatternForm {
    pattern: String,
}

#[route(POST "/settings/exclude-add")]
async fn settings_exclude_add(cx: &Cx, Form(f): Form<PatternForm>) -> Result<SeeOther> {
    Ok(
        match shared(cx).edit_engine(false, |c| c.add_exclude(&f.pattern)) {
            Ok(()) => settings_back("Pattern added."),
            Err(e) => settings_back(&e),
        },
    )
}

#[derive(Deserialize)]
struct IdxForm {
    idx: usize,
}

#[route(POST "/settings/root-remove")]
async fn settings_root_remove(cx: &Cx, Form(f): Form<IdxForm>) -> Result<SeeOther> {
    Ok(
        match shared(cx).edit_engine(true, |c| c.remove_scan_root(f.idx)) {
            Ok(()) => settings_back("Folder removed — re-measuring."),
            Err(e) => settings_back(&e),
        },
    )
}

#[route(POST "/settings/exclude-remove")]
async fn settings_exclude_remove(cx: &Cx, Form(f): Form<IdxForm>) -> Result<SeeOther> {
    Ok(
        match shared(cx).edit_engine(false, |c| c.remove_exclude(f.idx)) {
            Ok(()) => settings_back("Pattern removed."),
            Err(e) => settings_back(&e),
        },
    )
}

#[page("/about")]
async fn about_page(cx: &Cx) -> Result<impl View> {
    let _ = shared(cx);
    let theme_idx = theme_for(None);
    let rows = cleansys_core::appinfo::about_rows();
    Ok(view! {
        document(refresh: 0, theme: theme_idx, title: "About — CleanSys".to_string(),
            <header class="top"><h1>"ℹ About"</h1><small class="ver">(cleansys_core::appinfo::version_label())</small><span class="grow"></span><a class="btn" href="/">"← Back"</a></header>
            info_tabs(active: "about".to_string())
            <div class="card">
                <h2>"🧹 " (cleansys_core::appinfo::title())</h2>
                <p class="dim">(cleansys_core::appinfo::TAGLINE)</p>
                <table class="kv">
                    for (label, value) in rows.iter() {
                        <tr>
                            <th>(*label)</th>
                            <td>
                                if cleansys_core::appinfo::is_link(value) {
                                    <a href=(value.clone()) target="_blank" rel="noopener noreferrer">(value.clone())</a>
                                } else {
                                    (value.clone())
                                }
                            </td>
                        </tr>
                    }
                </table>
            </div>
        )
    })
}

// ── schedule ─────────────────────────────────────────────────────────

#[query_params(error = redirect("/schedule"))]
pub struct ScheduleParams {
    pub msg: Option<String>,
}

#[page("/schedule")]
async fn schedule_page(cx: &Cx) -> Result<impl View> {
    let params = query_params::<ScheduleParams>(cx)?;
    let theme_idx = theme_for(None);
    let current = Schedule::load();
    let installed = schedule::installed().map(|j| j.backend.to_string());
    let last = LastRun::load();
    let d = current.clone().unwrap_or_default();
    let cfg = EngineConfig::load();
    let roots: Vec<String> = cfg
        .effective_roots()
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    let linux = cfg!(all(unix, not(target_os = "macos")));
    Ok(view! {
        document(refresh: 0, theme: theme_idx, title: "Schedule — CleanSys".to_string(),
            <header class="top"><h1>"⏰ Automatic cleaning"</h1><span class="grow"></span><a class="btn" href="/">"← Back"</a></header>
            if let Some(m) = &params.msg { <div class="banner">(m.clone())</div> }
            <div class="card">
                <h2>"Status"</h2>
                if let Some(b) = &installed {
                    <p class="ok">"ACTIVE via " (b.clone()) " — " (d.describe()) " · " (d.scope_label())</p>
                } else {
                    <p class="dim">"Not scheduled."</p>
                }
                if let Some(l) = &last {
                    <p class="dim">"Last run: " (l.ago()) " — freed " (fmt(l.bytes_freed)) " (" (l.cleaners_run) " cleaners" if l.scheduled { ", scheduled" } ")"</p>
                }
            </div>
            <div class="card">
                <h2>"Schedule"</h2>
                <form class="stack" method="post" action="/schedule/install">
                    <label for="every">"Repeat"</label>
                    <select id="every" name="every">
                        <option value="daily" selected=(d.frequency == Frequency::Daily)>"Daily"</option>
                        <option value="weekly" selected=(d.frequency == Frequency::Weekly)>"Weekly"</option>
                        <option value="monthly" selected=(d.frequency == Frequency::Monthly)>"Monthly"</option>
                    </select>
                    <label for="weekday">"Weekday (weekly)"</label>
                    <select id="weekday" name="weekday">
                        for (i, n) in WEEKDAYS.iter().enumerate() {
                            <option value=(i.to_string()) selected=(i as u8 == d.weekday)>(*n)</option>
                        }
                    </select>
                    <label for="day">"Day of month (monthly)"</label>
                    <select id="day" name="day">
                        for n in 1..=28u8 {
                            <option value=(n.to_string()) selected=(n == d.day_of_month)>(n)</option>
                        }
                    </select>
                    <label for="hour">"Time"</label>
                    <span>
                        <select id="hour" name="hour">
                            for h in 0..24u8 { <option value=(h.to_string()) selected=(h == d.hour)>(format!("{h:02}"))</option> }
                        </select>
                        " : "
                        <select name="minute" aria-label="Minute">
                            for m in (0..60u8).step_by(5) { <option value=(m.to_string()) selected=(m == d.minute)>(format!("{m:02}"))</option> }
                        </select>
                    </span>
                    <label for="scope">"Cleans"</label>
                    <select id="scope" name="scope">
                        <option value="recommended" selected=(d.scope == Scope::Recommended)>"Recommended — safe caches only"</option>
                        <option value="extended" selected=(d.scope == Scope::Extended)>"Extended — safe + moderate"</option>
                        <option value="selected" selected=(d.scope == Scope::Selected)>"Selected — the cleaners ticked on the main page"</option>
                    </select>
                    if linux {
                        <label for="backend">"Backend"</label>
                        <select id="backend" name="backend">
                            <option value="auto" selected=(d.backend == Backend::Auto)>"Auto (systemd timer, else cron)"</option>
                            <option value="systemd" selected=(d.backend == Backend::Systemd)>"systemd user timer"</option>
                            <option value="cron" selected=(d.backend == Backend::Cron)>"cron (crontab)"</option>
                        </select>
                    }
                    <div class="actions">
                        <button class="primary" type="submit">if installed.is_some() { "Update schedule" } else { "Enable schedule" }</button>
                    </div>
                </form>
                if installed.is_some() {
                    <form method="post" action="/schedule/remove" style="margin-top:10px"><button class="danger" type="submit">"Remove schedule"</button></form>
                }
                <p class="dim">"Unattended runs only touch user-land cleaners, skip apps that are open, and never include caution-risk cleaners unless you picked them."</p>
            </div>
            <div class="card">
                <h2>"Project scan roots"</h2>
                if roots.is_empty() {
                    <p class="dim">"None found — add one with " <code>"cleansys config add-root ~/code"</code> "."</p>
                }
                <ul>for r in &roots { <li><code>(r.clone())</code></li> }</ul>
                <p class="dim">"Build output is only cleaned when untouched for " (cfg.min_age_days) " days (" <code>"cleansys config min-age N"</code> ")."</p>
            </div>
        )
    })
}

#[derive(Deserialize)]
struct ScheduleForm {
    every: String,
    hour: u8,
    minute: u8,
    weekday: Option<u8>,
    day: Option<u8>,
    scope: String,
    backend: Option<String>,
}

fn msg_redirect(msg: &str) -> SeeOther {
    see_other(format!("/schedule?msg={}", crate::util::pct(msg)))
}

#[route(POST "/schedule/install")]
async fn schedule_install(cx: &Cx, Form(f): Form<ScheduleForm>) -> Result<SeeOther> {
    let st = shared(cx);
    let mut s = Schedule::load().unwrap_or_default();
    s.frequency = match f.every.as_str() {
        "daily" => Frequency::Daily,
        "monthly" => Frequency::Monthly,
        _ => Frequency::Weekly,
    };
    s.hour = f.hour;
    s.minute = f.minute;
    if let Some(w) = f.weekday {
        s.weekday = w.min(6);
    }
    if let Some(d) = f.day {
        s.day_of_month = d.clamp(1, 28);
    }
    s.scope = match f.scope.as_str() {
        "extended" => Scope::Extended,
        "selected" => Scope::Selected,
        _ => Scope::Recommended,
    };
    if s.scope == Scope::Selected {
        s.ids = st
            .snapshot()
            .items
            .iter()
            .filter(|i| i.selected && !i.requires_root)
            .map(|i| i.id.clone())
            .collect();
        if s.ids.is_empty() {
            return Ok(msg_redirect(
                "Scope “Selected” needs ticked cleaners — tick some on the main page first.",
            ));
        }
    }
    s.backend = match f.backend.as_deref() {
        Some("systemd") => Backend::Systemd,
        Some("cron") => Backend::Cron,
        _ => Backend::Auto,
    };
    let desc = s.describe();
    let result = tokio::task::spawn_blocking(move || schedule::install(&s))
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| r.map_err(|e| format!("{e:#}")));
    Ok(match result {
        Ok(job) => msg_redirect(&format!("✓ Scheduled: {desc} via {}", job.backend)),
        Err(e) => msg_redirect(&format!("✗ {e}")),
    })
}

#[route(POST "/schedule/remove")]
async fn schedule_remove() -> Result<SeeOther> {
    let r = tokio::task::spawn_blocking(schedule::remove)
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| r.map_err(|e| format!("{e:#}")));
    Ok(match r {
        Ok(()) => msg_redirect("✓ Schedule removed."),
        Err(e) => msg_redirect(&format!("✗ {e}")),
    })
}
