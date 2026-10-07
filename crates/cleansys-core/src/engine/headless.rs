//! Non-interactive runs: `cleansys scan`, `clean`, `auto` and scheduled jobs.
//!
//! Everything works on the same [`CleanerCategory`] list the TUI and GUI show
//! (so ids, risk levels and the user-land / root split are identical).

use anyhow::{bail, Result};
use serde::Serialize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use super::running::AppRunning;
use super::schedule::{LastRun, Schedule, Scope};
use crate::cleaners::cleaned_item::{CleanerFn, RunOptions};
use crate::model::{load_categories, CleanerCategory, ROOT_SUFFIX};
use crate::utils::{check_root, confirm, format_size, print_error, print_success, print_warning};
use crate::Risk;

/// Which cleaners a headless run targets.
#[derive(Debug, Default, Clone)]
pub struct Selection {
    /// Exact cleaner ids. Explicit ids may include `caution` cleaners.
    pub ids: Vec<String>,
    /// Categories (case-insensitive). Also `user` (all user-land categories)
    /// and `system`/`root` (all categories that need root).
    pub categories: Vec<String>,
    /// Allow `caution`-risk cleaners picked via category / `all`.
    pub include_caution: bool,
    /// Every cleaner (subject to `include_caution`).
    pub all: bool,
    /// The recommended preset: `safe`, user-land cleaners only.
    pub recommended: bool,
    /// With `recommended`: also `moderate` ones.
    pub include_moderate: bool,
}

/// One runnable cleaner.
#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub category: String,
    pub risk: Risk,
    pub requires_root: bool,
    pub function: CleanerFn,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemOut {
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", content = "detail", rename_all = "snake_case")]
pub enum OutcomeStatus {
    Ok,
    /// Not run (needs root, app is open, ...).
    Skipped(String),
    Failed(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub id: String,
    pub name: String,
    pub category: String,
    pub risk: Risk,
    pub requires_root: bool,
    pub bytes: u64,
    pub items: Vec<ItemOut>,
    #[serde(flatten)]
    pub status: OutcomeStatus,
}

/// The system/root side of the UI: the built-in "System Cleaners" category
/// and every engine category that needs root.
fn category_is_root(c: &CleanerCategory) -> bool {
    c.name == "System Cleaners" || c.name.ends_with(ROOT_SUFFIX)
}

fn category_matches(c: &CleanerCategory, wanted: &str) -> bool {
    let w = wanted.trim().to_lowercase();
    let name = c.name.to_lowercase();
    match w.as_str() {
        "user" | "user-land" | "userland" => !category_is_root(c),
        "system" | "root" => category_is_root(c),
        _ => name == w || name.trim_end_matches(&ROOT_SUFFIX.to_lowercase()) == w,
    }
}

/// Resolve a [`Selection`] against `categories` (consumed).
pub fn select(categories: Vec<CleanerCategory>, sel: &Selection) -> Result<Vec<Job>> {
    for id in &sel.ids {
        if !categories
            .iter()
            .flat_map(|c| &c.items)
            .any(|i| &i.id == id)
        {
            bail!("unknown cleaner id '{id}' (see `cleansys list`)");
        }
    }
    for w in &sel.categories {
        if !categories.iter().any(|c| category_matches(c, w)) {
            bail!("unknown category '{w}' (see `cleansys list`)");
        }
    }
    let mut jobs = Vec::new();
    for cat in categories {
        let cat_hit = sel.categories.iter().any(|w| category_matches(&cat, w));
        for item in cat.items {
            let explicit = sel.ids.contains(&item.id);
            let by_cat = cat_hit && (sel.include_caution || item.risk != Risk::Caution);
            let by_all = sel.all && (sel.include_caution || item.risk != Risk::Caution);
            let by_recommended = sel.recommended
                && !item.requires_root
                && (item.risk == Risk::Safe
                    || (sel.include_moderate && item.risk == Risk::Moderate));
            if explicit || by_cat || by_all || by_recommended {
                jobs.push(Job {
                    id: item.id,
                    name: item.name,
                    category: cat.name.clone(),
                    risk: item.risk,
                    requires_root: item.requires_root,
                    function: item.function,
                });
            }
        }
    }
    Ok(jobs)
}

fn run_one(job: &Job, opts: RunOptions) -> Outcome {
    let mut out = Outcome {
        id: job.id.clone(),
        name: job.name.clone(),
        category: job.category.clone(),
        risk: job.risk,
        requires_root: job.requires_root,
        bytes: 0,
        items: Vec::new(),
        status: OutcomeStatus::Ok,
    };
    // A cached sudo password (GUI/TUI/web prompt) lets root cleaners run via `sudo -S`.
    if job.requires_root
        && !opts.dry_run
        && !check_root()
        && crate::auth::cached_sudo_password().is_none()
    {
        out.status = OutcomeStatus::Skipped("needs root (re-run with sudo)".into());
        return out;
    }
    match (job.function)(opts) {
        Ok(res) => {
            out.bytes = res.total_bytes;
            out.items = res
                .items
                .iter()
                .map(|i| ItemOut {
                    path: i.path_str(),
                    bytes: i.size,
                })
                .collect();
        }
        Err(e) => match e.downcast_ref::<AppRunning>() {
            Some(a) => out.status = OutcomeStatus::Skipped(a.to_string()),
            None => out.status = OutcomeStatus::Failed(format!("{e:#}")),
        },
    }
    out
}

/// Run jobs. Previews run on a small thread pool (they only read the
/// filesystem); real runs are sequential so prompts and output stay ordered.
pub fn execute(jobs: &[Job], opts: RunOptions) -> Vec<Outcome> {
    if !opts.dry_run || jobs.len() < 2 {
        return jobs.iter().map(|j| run_one(j, opts)).collect();
    }
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<Outcome>>> = jobs.iter().map(|_| Mutex::new(None)).collect();
    let workers = std::env::var("CLEANSYS_SCAN_THREADS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get().min(8))
                .unwrap_or(4)
        });
    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= jobs.len() {
                    break;
                }
                let r = run_one(&jobs[i], opts);
                *slots[i].lock().unwrap_or_else(|e| e.into_inner()) = Some(r);
            });
        }
    });
    slots
        .into_iter()
        .filter_map(|m| m.into_inner().unwrap_or_else(|e| e.into_inner()))
        .collect()
}

// ── presentation ──────────────────────────────────────────────────────

fn risk_tag(r: Risk) -> &'static str {
    match r {
        Risk::Safe => "",
        Risk::Moderate => " ~",
        Risk::Caution => " !",
    }
}

/// Print outcomes grouped by category, biggest first, hiding empty ones.
/// `detail` also lists the biggest paths of each cleaner.
pub fn print_outcomes(outcomes: &[Outcome], dry_run: bool, detail: bool) {
    let verb = if dry_run { "reclaimable" } else { "freed" };
    let mut cats: Vec<&str> = Vec::new();
    for o in outcomes {
        if !cats.contains(&o.category.as_str()) {
            cats.push(&o.category);
        }
    }
    for cat in cats {
        let mut rows: Vec<&Outcome> = outcomes
            .iter()
            .filter(|o| o.category == cat && (o.bytes > 0 || o.status != OutcomeStatus::Ok))
            .collect();
        if rows.is_empty() {
            continue;
        }
        rows.sort_by_key(|r| std::cmp::Reverse(r.bytes));
        let total: u64 = rows.iter().map(|r| r.bytes).sum();
        println!("\n{cat}  —  {}", format_size(total));
        for r in rows {
            match &r.status {
                OutcomeStatus::Ok => {
                    println!(
                        "  {:>10}  {}{}",
                        format_size(r.bytes),
                        r.name,
                        risk_tag(r.risk)
                    );
                    if detail {
                        let mut it: Vec<&ItemOut> = r.items.iter().collect();
                        it.sort_by_key(|i| std::cmp::Reverse(i.bytes));
                        for i in it.iter().take(5) {
                            println!("              {:>10}  {}", format_size(i.bytes), i.path);
                        }
                        if it.len() > 5 {
                            println!("              … and {} more", it.len() - 5);
                        }
                    }
                }
                OutcomeStatus::Skipped(why) => {
                    println!("  {:>10}  {} — skipped: {why}", "-", r.name)
                }
                OutcomeStatus::Failed(why) => println!("  {:>10}  {} — FAILED: {why}", "✗", r.name),
            }
        }
    }
    let total: u64 = outcomes.iter().map(|o| o.bytes).sum();
    println!(
        "\nTotal {verb}: {}   (~ moderate, ! caution)",
        format_size(total)
    );
}

fn totals(outcomes: &[Outcome]) -> LastRun {
    LastRun {
        at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        bytes_freed: outcomes.iter().map(|o| o.bytes).sum(),
        items: outcomes.iter().map(|o| o.items.len()).sum(),
        cleaners_run: outcomes
            .iter()
            .filter(|o| o.status == OutcomeStatus::Ok)
            .count(),
        skipped: outcomes
            .iter()
            .filter(|o| matches!(o.status, OutcomeStatus::Skipped(_)))
            .count(),
        failed: outcomes
            .iter()
            .filter(|o| matches!(o.status, OutcomeStatus::Failed(_)))
            .count(),
        scheduled: false,
    }
}

// ── commands ──────────────────────────────────────────────────────────

/// Print every cleaner grouped by category (same order as the UIs).
pub fn print_list() {
    for cat in load_categories() {
        println!("\n{}:", cat.name);
        for i in &cat.items {
            let tag = match i.risk {
                Risk::Safe => "",
                Risk::Moderate => " (moderate)",
                Risk::Caution => " (CAUTION)",
            };
            println!("  • {:<28} {}{}", i.id, i.name, tag);
        }
    }
}

/// `scan` / `clean`.
pub fn run_selection(sel: &Selection, opts: RunOptions, json: bool) -> Result<u64> {
    let jobs = select(load_categories(), sel)?;
    if jobs.is_empty() {
        print_warning(
            "Nothing selected. Try `cleansys auto`, --recommended, --all, --category or --id.",
        );
        return Ok(0);
    }
    if opts.dry_run && !json {
        eprintln!("Scanning {} cleaner(s)…", jobs.len());
    }
    let outcomes = execute(&jobs, opts);
    if json {
        println!("{}", serde_json::to_string_pretty(&outcomes)?);
    } else {
        print_outcomes(&outcomes, opts.dry_run, opts.dry_run);
    }
    Ok(outcomes.iter().map(|o| o.bytes).sum())
}

/// `auto`: scan the recommended set, show what would be freed, ask once, clean.
pub fn run_auto(extra: &Selection, yes: bool, dry_run: bool, json: bool) -> Result<u64> {
    let mut sel = extra.clone();
    sel.recommended = true;
    let jobs = select(load_categories(), &sel)?;
    if !json {
        eprintln!("Scanning {} cleaner(s)…", jobs.len());
    }
    let preview = execute(&jobs, RunOptions::preview());
    let reclaimable: u64 = preview.iter().map(|o| o.bytes).sum();

    if json && dry_run {
        println!("{}", serde_json::to_string_pretty(&preview)?);
        return Ok(reclaimable);
    }
    if !json {
        print_outcomes(&preview, true, false);
    }
    if dry_run {
        return Ok(reclaimable);
    }
    if reclaimable == 0 {
        if !json {
            print_success("Nothing to clean — your system is already tidy ✨");
        }
        return Ok(0);
    }
    if !yes && !confirm(&format!("\nFree {} now?", format_size(reclaimable)), true)? {
        return Ok(0);
    }
    let to_run: Vec<Job> = jobs
        .iter()
        .filter(|j| preview.iter().any(|o| o.id == j.id && o.bytes > 0))
        .cloned()
        .collect();
    let outcomes = execute(&to_run, RunOptions::execute());
    let mut last = totals(&outcomes);
    last.save().ok();
    if json {
        println!("{}", serde_json::to_string_pretty(&outcomes)?);
    } else {
        print_outcomes(&outcomes, false, false);
    }
    last.scheduled = false;
    Ok(last.bytes_freed)
}

/// Selection for a saved schedule.
pub fn selection_for(schedule: &Schedule) -> Selection {
    match schedule.scope {
        Scope::Recommended => Selection {
            recommended: true,
            ..Default::default()
        },
        Scope::Extended => Selection {
            recommended: true,
            include_moderate: true,
            ..Default::default()
        },
        Scope::Selected => Selection {
            ids: schedule.ids.clone(),
            ..Default::default()
        },
    }
}

/// Entry point of the OS-scheduled job (`cleansys auto --scheduled`).
/// Never prompts; cleans per the saved schedule; records [`LastRun`].
pub fn run_scheduled() -> Result<u64> {
    let Some(schedule) = Schedule::load() else {
        bail!("no schedule configured — run `cleansys schedule install` first");
    };
    let mut sel = selection_for(&schedule);
    // Unattended runs never touch root cleaners that need a password.
    let jobs: Vec<Job> = select(load_categories(), &sel)?
        .into_iter()
        .filter(|j| !j.requires_root || check_root())
        .collect();
    sel.ids.clear();
    println!(
        "[cleansys] scheduled run — {} — {} cleaner(s)",
        schedule.describe(),
        jobs.len()
    );
    let outcomes = execute(&jobs, RunOptions::execute());
    for o in &outcomes {
        match &o.status {
            OutcomeStatus::Ok if o.bytes > 0 => {
                println!("[cleansys] {}: freed {}", o.name, format_size(o.bytes))
            }
            OutcomeStatus::Skipped(w) => println!("[cleansys] {}: skipped ({w})", o.name),
            OutcomeStatus::Failed(w) => {
                print_error(&format!("{}: {w}", o.name));
            }
            _ => {}
        }
    }
    let mut last = totals(&outcomes);
    last.scheduled = true;
    last.save().ok();
    println!(
        "[cleansys] done — freed {} in {} item(s); {} skipped, {} failed",
        format_size(last.bytes_freed),
        last.items,
        last.skipped,
        last.failed
    );
    Ok(last.bytes_freed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::load_categories;

    #[test]
    fn recommended_is_safe_user_land_only() {
        let jobs = select(
            load_categories(),
            &Selection {
                recommended: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!jobs.is_empty());
        assert!(jobs
            .iter()
            .all(|j| j.risk == Risk::Safe && !j.requires_root));
    }

    #[test]
    fn caution_needs_explicit_id_or_flag() {
        let cats = load_categories();
        let caution: Vec<String> = cats
            .iter()
            .flat_map(|c| &c.items)
            .filter(|i| i.risk == Risk::Caution)
            .map(|i| i.id.clone())
            .collect();
        if let Some(id) = caution.first() {
            let all = select(
                load_categories(),
                &Selection {
                    all: true,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(all.iter().all(|j| j.risk != Risk::Caution));
            let one = select(
                load_categories(),
                &Selection {
                    ids: vec![id.clone()],
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(one.len(), 1);
        }
    }

    #[test]
    fn user_and_system_aliases_partition_categories() {
        let u = select(
            load_categories(),
            &Selection {
                categories: vec!["user".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(u
            .iter()
            .all(|j| !j.requires_root && j.category != "System Cleaners"));
        let r = select(
            load_categories(),
            &Selection {
                categories: vec!["system".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!r.is_empty());
        assert!(r
            .iter()
            .all(|j| j.category == "System Cleaners" || j.category.ends_with(ROOT_SUFFIX)));
    }

    #[test]
    fn unknown_id_and_category_error() {
        assert!(select(
            load_categories(),
            &Selection {
                ids: vec!["nope".into()],
                ..Default::default()
            }
        )
        .is_err());
        assert!(select(
            load_categories(),
            &Selection {
                categories: vec!["nope".into()],
                ..Default::default()
            }
        )
        .is_err());
    }

    #[test]
    fn scope_maps_to_selection() {
        let mut s = Schedule::default();
        assert!(selection_for(&s).recommended && !selection_for(&s).include_moderate);
        s.scope = Scope::Extended;
        assert!(selection_for(&s).include_moderate);
        s.scope = Scope::Selected;
        s.ids = vec!["a".into()];
        assert_eq!(selection_for(&s).ids, vec!["a".to_string()]);
    }
}
