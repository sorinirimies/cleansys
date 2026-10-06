//! Non-interactive helpers for the CLI (`cleansys scan` / `cleansys clean`).

use anyhow::{bail, Result};

use super::exec::run_spec;
use super::registry::load_specs;
use super::spec::{CleanerSpec, Risk};
use crate::cleaners::cleaned_item::RunOptions;
use crate::utils::{format_size, print_error, print_success, print_warning};

/// Which cleaners a headless run should target.
#[derive(Debug, Default, Clone)]
pub struct Selection {
    /// Exact cleaner ids (`proj-rust`, `ai-huggingface`, ...). Explicit ids
    /// may include `caution` cleaners.
    pub ids: Vec<String>,
    /// Whole categories (case-insensitive), e.g. "Project Build Artifacts".
    pub categories: Vec<String>,
    /// Allow `caution`-risk cleaners picked via category / `all`.
    pub include_caution: bool,
    /// Select every cleaner (subject to `include_caution`).
    pub all: bool,
}

/// Resolve a [`Selection`] against the loaded specs.
pub fn select(specs: &[CleanerSpec], sel: &Selection) -> Result<Vec<CleanerSpec>> {
    for id in &sel.ids {
        if !specs.iter().any(|s| &s.id == id) {
            bail!("unknown cleaner id '{id}' (see `cleansys list`)");
        }
    }
    for c in &sel.categories {
        if !specs.iter().any(|s| s.category.eq_ignore_ascii_case(c)) {
            bail!("unknown category '{c}' (see `cleansys list`)");
        }
    }
    Ok(specs
        .iter()
        .filter(|s| {
            let explicit = sel.ids.contains(&s.id);
            let bulk = sel.all
                || sel
                    .categories
                    .iter()
                    .any(|c| s.category.eq_ignore_ascii_case(c));
            explicit || (bulk && (sel.include_caution || s.risk != Risk::Caution))
        })
        .cloned()
        .collect())
}

/// Print every available declarative cleaner grouped by category.
pub fn print_list() {
    let mut last = String::new();
    for s in load_specs() {
        if s.category != last {
            println!("\n{}:", s.category);
            last = s.category.clone();
        }
        let tag = match s.risk {
            Risk::Safe => "",
            Risk::Moderate => " (moderate)",
            Risk::Caution => " (CAUTION)",
        };
        println!("  • {:<26} {}{}", s.id, s.name, tag);
    }
}

/// Run the selected cleaners; returns total bytes freed (or reclaimable when
/// `opts.dry_run`).
pub fn run_selection(sel: &Selection, opts: RunOptions) -> Result<u64> {
    let specs = load_specs();
    let chosen = select(&specs, sel)?;
    if chosen.is_empty() {
        print_warning("Nothing selected. Use --all, --category <name> or --id <id>.");
        return Ok(0);
    }
    let verb = if opts.dry_run { "reclaimable" } else { "freed" };
    let mut total = 0u64;
    for spec in chosen {
        if spec.requires_root && !crate::utils::check_root() {
            print_warning(&format!("Skipping '{}' (needs root)", spec.name));
            continue;
        }
        match run_spec(&spec, opts) {
            Ok(res) => {
                total += res.total_bytes;
                if res.item_count() > 0 || !opts.dry_run {
                    print_success(&format!(
                        "{}: {} {verb} across {} item(s)",
                        spec.name,
                        format_size(res.total_bytes),
                        res.item_count()
                    ));
                }
                if opts.dry_run {
                    for item in res.items.iter().take(50) {
                        println!("      {:>10}  {}", format_size(item.size), item.path_str());
                    }
                    if res.items.len() > 50 {
                        println!("      … and {} more", res.items.len() - 50);
                    }
                }
            }
            Err(e) => print_error(&format!("{}: {e:#}", spec.name)),
        }
    }
    print_success(&format!("Total {verb}: {}", format_size(total)));
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::registry::load_all_unfiltered;

    #[test]
    fn caution_excluded_from_bulk_but_allowed_explicitly() {
        let specs = load_all_unfiltered(None);
        let bulk = select(
            &specs,
            &Selection {
                all: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(bulk.iter().all(|s| s.risk != Risk::Caution));
        let one = select(
            &specs,
            &Selection {
                ids: vec!["ai-huggingface".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(one.len(), 1);
    }

    #[test]
    fn unknown_id_errors() {
        let specs = load_all_unfiltered(None);
        assert!(select(
            &specs,
            &Selection {
                ids: vec!["nope".into()],
                ..Default::default()
            }
        )
        .is_err());
    }
}
