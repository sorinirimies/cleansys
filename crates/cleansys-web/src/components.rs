//! Reusable view components.

use cleansys_core::Risk;
use topcoat::{
    Result,
    view::{Child, View, component, view},
};

use crate::state::{CatView, ItemView, Snapshot};
use crate::theme;
use crate::util::{fmt, nav_href, size_class};

/// HTML shell shared by every page. `refresh` (seconds, 0 = off) re-loads the page —
/// used while a scan or a clean is in progress.
#[component]
pub async fn document(
    refresh: u32,
    theme: usize,
    title: String,
    child: Child<'_>,
) -> Result<impl View> {
    let css = theme::css(theme);
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                if refresh > 0 {
                    <meta http-equiv="refresh" content=(refresh.to_string())>
                }
                <title>(title)</title>
                <link rel="stylesheet" href="/style.css">
                <style>(css)</style>
                topcoat::dev::script()
            </head>
            <body><main>(child)</main></body>
        </html>
    })
}

/// Logo, user/root badge, search box, schedule link and theme picker.
#[component]
pub async fn top_bar(
    is_root: bool,
    q: String,
    cat: usize,
    hide: bool,
    theme: usize,
    back: String,
) -> Result<impl View> {
    Ok(view! {
        <header class="top">
            <h1><a href="/" style="color:inherit">"🧹 CleanSys"</a></h1>
            if is_root {
                <span class="badge caut" title="System (root) section">"ROOT"</span>
            } else {
                <span class="badge user">"USER"</span>
            }
            <form class="search grow" method="get" action="/">
                <input type="search" name="q" value=(q) placeholder="Search cleaners…  (browsers, gradle, docker, logs)" aria-label="Search cleaners">
                <input type="hidden" name="cat" value=(cat.to_string())>
                if !hide {
                    <input type="hidden" name="hide" value="0">
                }
                <button type="submit" title="Search"><span class="ico">"🔎"</span><span class="lbl">"Search"</span></button>
            </form>
            <a class="btn" href="/schedule" title="Automatic cleaning"><span class="ico">"⏰"</span><span class="lbl">"⏰ Schedule"</span></a>
            <form method="get" action=(back)>
                <select name="theme" onchange="this.form.submit()" aria-label="Theme">
                    for (i, n) in theme::names().iter().enumerate() {
                        <option value=(*n) selected=(i == theme)>(*n)</option>
                    }
                </select>
            </form>
        </header>
    })
}

fn cat_size(c: &CatView) -> (String, &'static str) {
    match c.bytes {
        Some(b) if b > 0 => (fmt(b), size_class(b)),
        Some(_) => ("—".to_string(), "dim"),
        None => (String::new(), "dim"),
    }
}

/// A small CSS-animated spinner shown instead of numbers that are not ready yet.
#[component]
pub async fn spinner() -> Result<impl View> {
    Ok(view! { <span class="spin" role="status" aria-label="measuring"></span> })
}

/// Sidebar of categories: USER LAND above SYSTEM · ROOT.
#[component]
pub async fn side_nav(
    categories: &[CatView],
    active: usize,
    q: String,
    hide: bool,
    scanning: bool,
) -> Result<impl View> {
    let user: Vec<&CatView> = categories.iter().filter(|c| !c.root && c.visible).collect();
    let root: Vec<&CatView> = categories.iter().filter(|c| c.root && c.visible).collect();
    Ok(view! {
        <nav class="side" aria-label="Categories">
            <div class="section">"User land" <small>"no password needed"</small></div>
            for c in &user {
                <a class=(if c.index == active && q.trim().is_empty() { "cat active" } else { "cat" }) href=(nav_href(c.index, &q, hide))>
                    <span class="name">(c.name.clone())</span>
                    if c.ticked > 0 { <span class="tick">(c.ticked) " ✓"</span> }
                    <span class=(format!("sz {}", cat_size(c).1))>
                        if c.bytes.is_none() && scanning { spinner() } else { (cat_size(c).0) }
                    </span>
                </a>
            }
            if !root.is_empty() {
                <div class="section">"System · root" <small>"needs sudo cleansys-web"</small></div>
                for c in &root {
                    <a class=(if c.index == active && q.trim().is_empty() { "cat active" } else { "cat" }) href=(nav_href(c.index, &q, hide))>
                        <span class="name">(c.name.trim_end_matches(cleansys_core::model::ROOT_SUFFIX).to_string())</span>
                        if c.ticked > 0 { <span class="tick">(c.ticked) " ✓"</span> }
                        <span class=(format!("sz {}", cat_size(c).1))>
                            if c.bytes.is_none() && scanning { spinner() } else { (cat_size(c).0) }
                        </span>
                    </a>
                }
            }
            if scanning {
                <div class="section">"Scanning…"</div>
            }
        </nav>
    })
}

/// Drop-down navigation shown instead of the sidebar on narrow screens.
#[component]
pub async fn cat_pick(categories: &[CatView], active: usize, hide: bool) -> Result<impl View> {
    Ok(view! {
        <form class="catpick" method="get" action="/">
            if !hide { <input type="hidden" name="hide" value="0"> }
            <select name="cat" onchange="this.form.submit()" aria-label="Category">
                for c in categories.iter().filter(|c| c.visible) {
                    <option value=(c.index.to_string()) selected=(c.index == active)>
                        (c.name.clone()) (if c.bytes.unwrap_or(0) > 0 { format!(" · {}", fmt(c.bytes.unwrap_or(0))) } else { String::new() })
                    </option>
                }
            </select>
        </form>
    })
}

/// One cleaner: the whole row is a label, ticking it posts the change.
#[component]
pub async fn item_row(
    item: &ItemView,
    back: String,
    show_category: Option<String>,
    scanning_now: bool,
) -> Result<impl View> {
    let size = item.scan.as_ref();
    let class = format!(
        "item{}{}",
        if item.selected { " sel" } else { "" },
        if item.selectable { "" } else { " off" }
    );
    Ok(view! {
        <form class="item" method="post" action="/toggle">
            <input type="hidden" name="id" value=(item.id.clone())>
            <input type="hidden" name="back" value=(back)>
            <label class=(class)>
                <input type="checkbox" checked=(item.selected) disabled=(!item.selectable) onchange="this.form.submit()" aria-label=(item.name.clone())>
                <span class="title">
                    (item.name.clone())
                    if item.risk == Risk::Moderate { <span class="badge mod" title="slow to rebuild or re-download">"MODERATE"</span> }
                    if item.risk == Risk::Caution { <span class="badge caut" title="large downloads or user data">"CAUTION"</span> }
                    if item.requires_root { <span class="badge root">"ROOT"</span> }
                    if let Some(c) = &show_category { <span class="dim">"· " (c.clone())</span> }
                </span>
                <span class="desc">(item.description.clone())</span>
                if let Some(top) = size.and_then(|s| s.top_path.clone()) {
                    <span class="path">"📁 " (top)</span>
                }
                <span class=(format!("size {}", match size { Some(s) if s.bytes > 0 => size_class(s.bytes), Some(_) => "none", None => "none" }))>
                    if let Some(s) = size {
                        if s.error.is_some() {
                            <span class="err">"scan failed"</span>
                        } else if s.bytes > 0 {
                            <b>(fmt(s.bytes))</b> <small>(s.items) " item(s)"</small>
                        } else {
                            "nothing to clean"
                        }
                    } else if scanning_now {
                        spinner() " measuring…"
                    } else {
                        "—"
                    }
                </span>
                <noscript><button type="submit">"Toggle"</button></noscript>
            </label>
        </form>
    })
}

/// Sticky bar: selection summary / progress and the action buttons.
#[component]
pub async fn action_bar(
    snap: &Snapshot,
    back: String,
    cat: usize,
    hide: bool,
) -> Result<impl View> {
    let can_run = snap.selected_count > 0 && snap.run.phase != crate::state::Phase::Running;
    let reclaim = snap.selected_bytes;
    Ok(view! {
        <div class="bar">
            <div class="inner">
                <div class="sum">
                    if snap.scanning {
                        <b>spinner() " Scanning your system… " (snap.scan_done) "/" (snap.scan_total)</b>
                        <div class="progress"><span style=(format!("width:{}%", (snap.scan_done * 100).checked_div(snap.scan_total).unwrap_or(0)))></span></div>
                    } else if snap.selected_count == 0 {
                        <b>(fmt(snap.total_bytes)) " can be freed"</b>
                        <small>"tick cleaners or press Recommended"</small>
                    } else {
                        <b>(snap.selected_count) " selected · " (fmt(reclaim)) " to free"</b>
                        <small>(fmt(snap.total_bytes)) " reclaimable in total"</small>
                    }
                </div>
                <form method="post" action="/select">
                    <input type="hidden" name="back" value=(back.clone())>
                    <input type="hidden" name="cat" value=(cat.to_string())>
                    if !hide { <input type="hidden" name="hide" value="0"> }
                    <button name="op" value="recommended" title="Tick only the safe, user-land cleaners that have something to free"><span class="ico">"✨"</span><span class="lbl">"✨ Recommended"</span></button>
                    <button name="op" value="none" title="Untick everything"><span class="ico">"☐"</span><span class="lbl">"Select none"</span></button>
                    <button name="op" value="rescan" title="Measure what every cleaner can free again"><span class="ico">"⟳"</span><span class="lbl">"⟳ Rescan"</span></button>
                </form>
                <a class="btn" href=(if can_run { "/preview" } else { "#" }) aria-disabled=(if can_run { "false" } else { "true" }) title="Show exactly what would be removed (deletes nothing)"><span class="ico">"🔍"</span><span class="lbl">"🔍 Preview"</span></a>
                <a class="btn primary" href=(if can_run { "/confirm" } else { "#" }) aria-disabled=(if can_run { "false" } else { "true" })>
                    if can_run && snap.scanning {
                        "🧹 Clean " (snap.selected_count) " · " spinner() " measuring…"
                    } else if can_run {
                        "🧹 Clean " (snap.selected_count) " · " (fmt(reclaim))
                    } else {
                        "Select cleaners"
                    }
                </a>
            </div>
        </div>
    })
}
