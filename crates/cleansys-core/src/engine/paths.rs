//! Path templating (`~`, `$VAR`, `${VAR:-default}`, `%VAR%`) and glob expansion.

use std::path::PathBuf;

/// Look up an environment variable with sensible per-platform fallbacks for
/// the XDG / Windows well-known directories. Empty values count as unset.
pub fn lookup_env(name: &str) -> Option<String> {
    for candidate in [name.to_string(), name.to_ascii_uppercase()] {
        if let Ok(v) = std::env::var(&candidate) {
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    let home = crate::cleaners::platform::home_dir;
    let under = |rel: &str| home().map(|h| h.join(rel).to_string_lossy().into_owned());
    match name.to_ascii_uppercase().as_str() {
        "HOME" | "USERPROFILE" => home().map(|h| h.to_string_lossy().into_owned()),
        "XDG_CACHE_HOME" => under(".cache"),
        "XDG_STATE_HOME" => under(".local/state"),
        "XDG_DATA_HOME" => under(".local/share"),
        "XDG_CONFIG_HOME" => under(".config"),
        "LOCALAPPDATA" => under("AppData/Local"),
        "APPDATA" => under("AppData/Roaming"),
        "SYSTEMROOT" | "WINDIR" => Some("C:\\Windows".to_string()),
        "SYSTEMDRIVE" => Some("C:".to_string()),
        "PROGRAMDATA" | "ALLUSERSPROFILE" => Some("C:\\ProgramData".to_string()),
        "PROGRAMFILES" => Some("C:\\Program Files".to_string()),
        "PROGRAMFILES(X86)" => Some("C:\\Program Files (x86)".to_string()),
        "TEMP" | "TMP" => Some(std::env::temp_dir().to_string_lossy().into_owned()),
        _ => None,
    }
}

/// Expand a path template using `env` for variable lookups.
///
/// Unknown variables expand to the empty string, which makes the resulting
/// path obviously invalid; callers drop results that are not absolute.
pub fn expand(input: &str, env: &dyn Fn(&str) -> Option<String>) -> String {
    let mut s = input.to_string();
    if s == "~" || s.starts_with("~/") || s.starts_with("~\\") {
        s = format!("{}{}", env("HOME").unwrap_or_default(), &s[1..]);
    }

    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
            if let Some(end) = chars[i + 2..].iter().position(|&c| c == '}') {
                let inner: String = chars[i + 2..i + 2 + end].iter().collect();
                let (name, default) = match inner.split_once(":-") {
                    Some((n, d)) => (n.to_string(), Some(d.to_string())),
                    None => (inner, None),
                };
                match env(&name) {
                    Some(v) => out.push_str(&v),
                    None => {
                        if let Some(d) = default {
                            out.push_str(&expand(&d, env));
                        }
                    }
                }
                i += end + 3;
                continue;
            }
        } else if c == '$' && i + 1 < chars.len() && is_name_char(chars[i + 1]) {
            let mut j = i + 1;
            while j < chars.len() && is_name_char(chars[j]) {
                j += 1;
            }
            let name: String = chars[i + 1..j].iter().collect();
            out.push_str(&env(&name).unwrap_or_default());
            i = j;
            continue;
        } else if c == '%' {
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == '%') {
                let name: String = chars[i + 1..i + 1 + end].iter().collect();
                if !name.is_empty()
                    && name
                        .chars()
                        .all(|c| is_name_char(c) || c == '(' || c == ')')
                {
                    out.push_str(&env(&name).unwrap_or_default());
                    i += end + 2;
                    continue;
                }
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Expand a template and resolve glob patterns to existing absolute paths
/// (symlinks are returned as-is, never followed). Sorted and de-duplicated.
pub fn resolve(template: &str, env: &dyn Fn(&str) -> Option<String>) -> Vec<PathBuf> {
    let expanded = expand(template, env);
    let path = PathBuf::from(&expanded);
    if !path.is_absolute() {
        return Vec::new();
    }

    let mut found: Vec<PathBuf> = if expanded.contains(['*', '?', '[']) {
        glob::glob(&expanded)
            .map(|it| it.flatten().collect())
            .unwrap_or_default()
    } else if std::fs::symlink_metadata(&path).is_ok() {
        vec![path]
    } else {
        Vec::new()
    };
    found.sort();
    found.dedup();
    found
}

/// Locate an executable on `PATH` (or accept an absolute path).
pub fn find_program(program: &str) -> Option<PathBuf> {
    let direct = PathBuf::from(program);
    if direct.is_absolute() {
        return direct.is_file().then_some(direct);
    }
    let exts: Vec<String> = if cfg!(windows) {
        vec![
            "".into(),
            ".exe".into(),
            ".cmd".into(),
            ".bat".into(),
            ".com".into(),
        ]
    } else {
        vec!["".into()]
    };
    std::env::split_paths(&std::env::var_os("PATH")?).find_map(|dir| {
        exts.iter().find_map(|e| {
            let candidate = dir.join(format!("{program}{e}"));
            candidate.is_file().then_some(candidate)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(name: &str) -> Option<String> {
        match name {
            "HOME" => Some("/home/u".into()),
            "FOO" => Some("/foo".into()),
            "WIN" => Some("C:\\w".into()),
            _ => None,
        }
    }

    #[test]
    fn tilde_and_vars() {
        assert_eq!(expand("~/.cache", &env), "/home/u/.cache");
        assert_eq!(expand("$FOO/bar", &env), "/foo/bar");
        assert_eq!(expand("${FOO}/bar", &env), "/foo/bar");
        assert_eq!(expand("%WIN%\\x", &env), "C:\\w\\x");
    }

    #[test]
    fn default_syntax() {
        assert_eq!(expand("${NOPE:-~/.cargo}/git", &env), "/home/u/.cargo/git");
        assert_eq!(expand("${FOO:-/other}", &env), "/foo");
    }

    #[test]
    fn unknown_var_makes_path_relative_and_dropped() {
        assert!(resolve("$NOPE/x", &env).is_empty());
    }

    #[test]
    fn percent_literal_is_kept() {
        assert_eq!(expand("100%/x", &env), "100%/x");
    }

    #[test]
    fn resolve_globs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.cache.v1"), "x").unwrap();
        std::fs::write(dir.path().join("a.cache.v2"), "x").unwrap();
        std::fs::write(dir.path().join("b.txt"), "x").unwrap();
        let root = dir.path().to_string_lossy().into_owned();
        let got = resolve(&format!("{root}/a.cache.*"), &env);
        assert_eq!(got.len(), 2);
    }
}
