//! The one system cleaner that is not declared in TOML: the Windows Recycle Bin, which
//! needs a Shell API call. Every other system cleaner lives in
//! `engine/builtin/system*.toml`.

use anyhow::Result;
#[cfg(target_os = "windows")]
use log::warn;

#[cfg(target_os = "windows")]
use crate::cleaners::cleaned_item::CleanedItem;
#[cfg(target_os = "windows")]
use crate::cleaners::cleaned_item::CleaningResult;
use crate::cleaners::cleaned_item::{CleanerFnPtr, RunOptions};
use crate::utils::{confirm, format_size, print_error, print_success};

/// Information about a system cleaner.
#[derive(Debug)]
pub struct CleanerInfo {
    /// The name of the cleaner.
    pub name: &'static str,
    /// A description of what the cleaner does.
    pub description: &'static str,
    /// The function that performs the cleaning operation.
    pub function: CleanerFnPtr,
    /// Whether this specific cleaner needs root/Administrator privileges.
    pub requires_root: bool,
}

/// Lists the built-in (non-TOML) system cleaners with their descriptions.
pub fn list_cleaners() -> Vec<String> {
    get_cleaners()
        .iter()
        .map(|c| format!("{}: {}", c.name, c.description))
        .collect()
}

/// The built-in (non-TOML) system cleaners of this platform: only the Windows Recycle Bin.
pub fn get_cleaners() -> Vec<CleanerInfo> {
    #[cfg(target_os = "windows")]
    {
        vec![CleanerInfo {
            name: "Recycle Bin",
            description: "Empty the Recycle Bin for all drives",
            function: clean_recycle_bin,
            requires_root: false,
        }]
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

/// Runs the built-in (non-TOML) system cleaners; the rest run through the engine.
pub fn run_all(skip_confirmation: bool) -> Result<()> {
    let opts = if skip_confirmation {
        RunOptions::execute()
    } else {
        RunOptions::execute_with_confirmation()
    };
    for cleaner in get_cleaners() {
        if skip_confirmation || confirm(&format!("Run '{}'?", cleaner.name), true)? {
            match (cleaner.function)(opts) {
                Ok(result) => print_success(&format!(
                    "{} completed: freed {} across {} item(s)",
                    cleaner.name,
                    format_size(result.total_bytes),
                    result.item_count()
                )),
                Err(err) => print_error(&format!("Error in {}: {err}", cleaner.name)),
            }
        }
    }
    Ok(())
}

/// Empty the Recycle Bin for all fixed drives via the Shell32
/// `SHEmptyRecycleBinW` API — the same call Explorer's own "Empty Recycle
/// Bin" menu item uses, so no elevated privileges are actually required for
/// the current user's own Recycle Bin (multi-user machines may still
/// restrict other users' bins, in which case the call simply reports 0
/// items freed for those).
#[cfg(target_os = "windows")]
fn clean_recycle_bin(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();

    // There is no cheap, official way to query the Recycle Bin's *current*
    // size before emptying it without walking every drive's hidden
    // `$Recycle.Bin` folder (which requires elevated access to enumerate
    // correctly per-SID); rather than mis-report a size, we surface the
    // fact that this happened via the item's label instead.
    if opts.dry_run {
        result.add_item(CleanedItem::directory(
            std::path::PathBuf::from("Recycle Bin"),
            0,
            "Recycle Bin (size unknown until emptied)",
        ));
        return Ok(result);
    }

    if !opts.skip_confirmation && !confirm("Empty the Recycle Bin?", true)? {
        return Ok(result);
    }

    match windows_shell::empty_recycle_bin() {
        Ok(()) => {
            print_success("Emptied the Recycle Bin");
            result.add_item(CleanedItem::directory(
                std::path::PathBuf::from("Recycle Bin"),
                0,
                "Recycle Bin",
            ));
        }
        Err(e) => warn!("Failed to empty Recycle Bin: {e}"),
    }

    Ok(result)
}

/// Thin wrapper around the Win32 Shell API, isolated so the rest of this
/// module never has to deal with raw FFI directly.
#[cfg(target_os = "windows")]
mod windows_shell {
    use anyhow::Result;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::{
        SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND, SHEmptyRecycleBinW,
    };

    /// Empty the Recycle Bin for every drive, silently (no confirmation
    /// dialog, no progress UI, no sound — `CleanSys` already asked the user).
    #[allow(unsafe_code, reason = "SHEmptyRecycleBinW is a Windows FFI call")]
    pub(crate) fn empty_recycle_bin() -> Result<()> {
        // SAFETY: `SHEmptyRecycleBinW(None, None, flags)` empties the
        // Recycle Bin for *all* drives when both the window handle and root
        // path are null, which is exactly the documented, supported usage.
        let result = unsafe {
            SHEmptyRecycleBinW(
                Some(HWND(std::ptr::null_mut())),
                None,
                SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
            )
        };
        result.map_err(|e| anyhow::anyhow!("SHEmptyRecycleBinW failed: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_cleaners_formats_name_and_description() {
        for line in list_cleaners() {
            assert!(line.contains(": "), "{line}");
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_lists_the_recycle_bin() {
        let names: Vec<_> = get_cleaners().iter().map(|c| c.name).collect();
        assert_eq!(names, ["Recycle Bin"]);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn other_platforms_have_no_builtin_system_cleaners() {
        assert!(get_cleaners().is_empty());
    }
}
