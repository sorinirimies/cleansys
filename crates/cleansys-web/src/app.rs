//! Server setup: options, router construction, the DNS-rebinding guard and the serve loop.

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        Body, Next, Router, RouterBuilderDiscoverExt, error::forbidden, layer, request::headers,
        response::Response,
    },
};

use crate::state::Shared;

/// Where and how to serve.
#[derive(Debug, Clone)]
pub struct ServerOptions {
    /// Interface to bind. Defaults to `127.0.0.1`: this UI can delete files.
    pub host: String,
    pub port: u16,
    /// Open the UI in the default browser once the server is listening.
    pub open_browser: bool,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3000,
            open_browser: false,
        }
    }
}

/// `Host` header values (without port) this server answers to. Anything else is
/// refused, which defeats DNS-rebinding attacks against a localhost server.
#[derive(Debug, Clone)]
pub struct AllowedHosts(pub Vec<String>);

impl AllowedHosts {
    pub fn for_bind(host: &str) -> Self {
        let mut v: Vec<String> = ["localhost", "127.0.0.1", "[::1]", "::1"]
            .map(String::from)
            .to_vec();
        if !matches!(
            host,
            "127.0.0.1" | "localhost" | "::1" | "[::1]" | "0.0.0.0" | "::"
        ) {
            v.push(host.to_string());
        }
        AllowedHosts(v)
    }

    pub fn allows(&self, host_header: &str) -> bool {
        self.0
            .iter()
            .any(|a| a.eq_ignore_ascii_case(strip_port(host_header)))
    }
}

/// `localhost:3000` → `localhost`, `[::1]:3000` → `[::1]`.
pub fn strip_port(host: &str) -> &str {
    if let Some(end) = host.strip_prefix('[').and_then(|_| host.find(']')) {
        return &host[..=end];
    }
    host.rsplit_once(':').map_or(host, |(h, _)| h)
}

#[layer("/")]
async fn host_guard(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let allowed = app_context::<AllowedHosts>(cx);
    let ok = headers(cx)
        .get("host")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| allowed.allows(h));
    if !ok {
        return Err(forbidden().into());
    }
    next.run(cx, body).await
}

/// Build the router around a shared state.
pub fn router(state: Shared, hosts: AllowedHosts) -> Router {
    Router::builder()
        .discover()
        .app_context(state)
        .app_context(hosts)
        .build()
}

/// Bind and serve until the process is killed.
pub async fn serve(opts: ServerOptions) -> std::io::Result<()> {
    let state = Shared::load();
    state.start_scan();

    let listener = tokio::net::TcpListener::bind((opts.host.as_str(), opts.port)).await?;
    let addr = listener.local_addr()?;
    // The sudo password prompt is only offered when the password can't leave this machine.
    let elevation = addr.ip().is_loopback() && cleansys_core::utils::supports_sudo_prompt();
    state.allow_elevation(elevation);
    println!("cleansys-web listening on http://{addr}");
    if !addr.ip().is_loopback() {
        eprintln!(
            "cleansys-web: WARNING — bound to {addr}. There is no authentication: anyone who can reach it can delete your caches."
        );
    }
    if !cleansys_core::check_root() {
        println!(
            "cleansys-web: running as a normal user — system (root) cleaners ask for your sudo password when you run them{}.",
            if elevation {
                ""
            } else {
                " (disabled: not bound to loopback — use `sudo cleansys-web`)"
            }
        );
    }
    if opts.open_browser {
        let url = browser_url(addr);
        if let Err(e) = open_in_browser(&url) {
            eprintln!("cleansys-web: could not open a browser ({e}); visit {url}");
        }
    }
    topcoat::serve(listener, router(state, AllowedHosts::for_bind(&opts.host))).await
}

/// URL to open for a bound address (`0.0.0.0` / `::` are not browsable → localhost).
pub fn browser_url(addr: std::net::SocketAddr) -> String {
    if addr.ip().is_unspecified() {
        format!("http://localhost:{}", addr.port())
    } else if addr.is_ipv6() {
        format!("http://[{}]:{}", addr.ip(), addr.port())
    } else {
        format!("http://{addr}")
    }
}

/// Open `url` with the platform's default handler (no extra dependency).
fn open_in_browser(url: &str) -> std::io::Result<()> {
    use std::process::{Command, Stdio};

    #[cfg(target_os = "macos")]
    let mut cmd = Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", ""]);
        c
    };
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut cmd = Command::new("xdg-open");

    cmd.arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_header_port_stripping() {
        assert_eq!(strip_port("localhost:3000"), "localhost");
        assert_eq!(strip_port("127.0.0.1"), "127.0.0.1");
        assert_eq!(strip_port("[::1]:3000"), "[::1]");
    }

    #[test]
    fn loopback_hosts_are_allowed_others_are_not() {
        let h = AllowedHosts::for_bind("127.0.0.1");
        assert!(h.allows("localhost:3000") && h.allows("127.0.0.1:8080") && h.allows("[::1]:1"));
        assert!(!h.allows("evil.example") && !h.allows("evil.example:3000"));
        let h = AllowedHosts::for_bind("192.168.1.5");
        assert!(h.allows("192.168.1.5:3000"));
    }

    #[test]
    fn browser_urls() {
        assert_eq!(
            browser_url("0.0.0.0:3000".parse().unwrap()),
            "http://localhost:3000"
        );
        assert_eq!(
            browser_url("127.0.0.1:3000".parse().unwrap()),
            "http://127.0.0.1:3000"
        );
        assert_eq!(
            browser_url("[::1]:3000".parse().unwrap()),
            "http://[::1]:3000"
        );
    }
}
