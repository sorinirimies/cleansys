//! "About" information shared by every front-end: version, author, links, paths.
//!
//! The TUI, GUI and web UI all render [`about_rows`], so they can never drift apart.

/// Product name.
pub const NAME: &str = "CleanSys";

/// Release version (the workspace version, e.g. `0.7.8`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One-line description.
pub const TAGLINE: &str = "Modern system cleaner for Linux, macOS & Windows";

/// Developer name.
pub const AUTHOR: &str = "Sorin Albu-Irimies";

/// GitHub account of the developer.
pub const GITHUB_USER: &str = "sorinirimies";

/// GitHub profile URL.
pub const GITHUB_PROFILE: &str = "https://github.com/sorinirimies";

/// Source repository.
pub const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// License identifier (SPDX).
pub const LICENSE: &str = env!("CARGO_PKG_LICENSE");

/// `v0.7.8` — what the UIs show next to the product name.
pub fn version_label() -> String {
    format!("v{VERSION}")
}

/// `CleanSys v0.7.8`.
pub fn title() -> String {
    format!("{NAME} {}", version_label())
}

/// Bug tracker.
pub fn issues_url() -> String {
    format!("{REPO_URL}/issues")
}

/// Pre-built binaries.
pub fn releases_url() -> String {
    format!("{REPO_URL}/releases")
}

/// `macos aarch64`, `linux x86_64`, …
pub fn platform() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Where the engine settings (`engine.json`) live.
pub fn engine_config_path() -> String {
    crate::engine::EngineConfig::path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "(unknown)".into())
}

/// Where the UI settings (`settings.json`) live.
pub fn settings_path() -> String {
    crate::settings::settings_json_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "(unknown)".into())
}

/// Label/value rows for the About page. Values that are links are listed in [`is_link`].
pub fn about_rows() -> Vec<(&'static str, String)> {
    vec![
        ("Version", version_label()),
        ("Developer", AUTHOR.to_string()),
        ("GitHub", GITHUB_PROFILE.to_string()),
        ("Repository", REPO_URL.to_string()),
        ("Report a bug", issues_url()),
        ("Downloads", releases_url()),
        ("License", LICENSE.to_string()),
        ("Platform", platform()),
        ("Engine config", engine_config_path()),
        ("Settings file", settings_path()),
    ]
}

/// Open `url` in the default browser (best effort; never blocks on the browser).
pub fn open_url(url: &str) -> std::io::Result<()> {
    use std::process::{Command, Stdio};
    // Only ever hand real web links to the OS opener.
    if !is_link(url) {
        return Err(std::io::Error::other("not a web link"));
    }
    #[cfg(target_os = "macos")]
    let mut cmd = Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", ""]);
        c
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let mut cmd = Command::new("xdg-open");
    cmd.arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

/// Whether an About value is a web link (so UIs can render it as one).
pub fn is_link(value: &str) -> bool {
    value.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_matches_the_workspace_manifest() {
        assert!(VERSION.split('.').count() == 3, "{VERSION}");
        assert_eq!(version_label(), format!("v{VERSION}"));
        assert!(title().starts_with("CleanSys v"));
        assert_eq!(GITHUB_PROFILE, format!("https://github.com/{GITHUB_USER}"));
        assert!(REPO_URL.starts_with(GITHUB_PROFILE), "{REPO_URL}");
        assert_eq!(LICENSE, "MIT");
    }

    #[test]
    fn rows_cover_the_essentials_and_flag_links() {
        let rows = about_rows();
        for label in [
            "Version",
            "Developer",
            "GitHub",
            "Repository",
            "License",
            "Platform",
        ] {
            assert!(rows.iter().any(|(l, _)| *l == label), "missing {label}");
        }
        assert!(rows.iter().any(|(_, v)| is_link(v)));
        assert!(!is_link("MIT"));
        assert!(issues_url().ends_with("/issues") && releases_url().ends_with("/releases"));
    }
}
