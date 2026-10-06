//! Small pure helpers: URL building/escaping and size formatting.

/// Percent-encode a query-string component.
pub fn pct(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// The home-page URL for a view: `/?cat=2&q=gradle&hide=0`.
pub fn nav_href(cat: usize, q: &str, hide_empty: bool) -> String {
    let mut url = format!("/?cat={cat}");
    if !q.trim().is_empty() {
        url.push_str(&format!("&q={}", pct(q.trim())));
    }
    if !hide_empty {
        url.push_str("&hide=0");
    }
    url
}

/// Only same-site relative targets are allowed as a post-action redirect
/// (`/` or `/?…` / `/path`), never `//host` or absolute URLs.
pub fn safe_back(back: &str) -> String {
    let b = back.trim();
    if b.starts_with('/')
        && !b.starts_with("//")
        && !b.contains('\\')
        && !b.contains('\n')
        && !b.contains('\r')
    {
        b.to_string()
    } else {
        "/".to_string()
    }
}

/// CSS class describing how big a size is (`big` / `mid` / `""`).
pub fn size_class(bytes: u64) -> &'static str {
    const GB: u64 = 1 << 30;
    const MB: u64 = 1 << 20;
    if bytes >= 5 * GB {
        "big"
    } else if bytes >= 500 * MB {
        "mid"
    } else {
        ""
    }
}

pub fn fmt(bytes: u64) -> String {
    cleansys_core::format_size(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pct_encodes_reserved_and_unicode() {
        assert_eq!(pct("a b&c"), "a%20b%26c");
        assert_eq!(pct("café"), "caf%C3%A9");
        assert_eq!(pct("safe-_.~"), "safe-_.~");
    }

    #[test]
    fn nav_href_carries_only_non_default_state() {
        assert_eq!(nav_href(2, "", true), "/?cat=2");
        assert_eq!(nav_href(0, "gra dle", false), "/?cat=0&q=gra%20dle&hide=0");
    }

    #[test]
    fn safe_back_rejects_open_redirects() {
        assert_eq!(safe_back("/?cat=1"), "/?cat=1");
        assert_eq!(safe_back("//evil.example"), "/");
        assert_eq!(safe_back("https://evil.example"), "/");
        assert_eq!(safe_back("/\\evil"), "/");
        assert_eq!(safe_back(""), "/");
    }

    #[test]
    fn size_classes() {
        assert_eq!(size_class(6 << 30), "big");
        assert_eq!(size_class(600 << 20), "mid");
        assert_eq!(size_class(10), "");
    }
}
