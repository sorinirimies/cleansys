//! End-to-end tests: a real server on an ephemeral port, driven with plain
//! HTTP/1.1 over a `TcpStream` (no client dependency).
//!
//! The environment is pointed at a throw-away sandbox *before* anything reads it,
//! so these tests never see (or touch) the developer's real files or settings.

use std::{net::SocketAddr, path::PathBuf, sync::OnceLock, time::Duration};

use cleansys_web::{AllowedHosts, Shared, router};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

struct Sandbox {
    project: PathBuf,
}

fn sandbox() -> &'static Sandbox {
    static BOX: OnceLock<Sandbox> = OnceLock::new();
    BOX.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("cleansys-web-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let projects = root.join("Projects");
        let project = projects.join("demo-rust");
        std::fs::create_dir_all(project.join("target/debug")).unwrap();
        std::fs::write(project.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();
        std::fs::write(project.join("target/debug/big.bin"), vec![b'x'; 200_000]).unwrap();
        std::fs::create_dir_all(root.join("tmp")).unwrap();
        // SAFETY: runs once, inside `get_or_init`, before any server thread reads the
        // environment (every test calls `sandbox()` first).
        unsafe {
            std::env::set_var("HOME", &root);
            std::env::set_var("USERPROFILE", &root);
            std::env::set_var("TMPDIR", root.join("tmp"));
            std::env::set_var("XDG_CACHE_HOME", root.join("xdg-cache"));
            std::env::set_var("XDG_CONFIG_HOME", root.join("xdg-config"));
            std::env::set_var("XDG_DATA_HOME", root.join("xdg-data"));
            std::env::set_var("APPDATA", root.join("appdata"));
            std::env::set_var("LOCALAPPDATA", root.join("localappdata"));
            std::env::set_var("CLEANSYS_SCAN_ROOTS", &projects);
        }
        // Clean fresh projects too (the default only touches ones idle for 14 days).
        let cfg = cleansys_core::engine::EngineConfig {
            min_age_days: 0,
            ..Default::default()
        };
        cfg.save().unwrap();
        Sandbox { project }
    })
}

async fn start() -> SocketAddr {
    sandbox();
    let state = Shared::load();
    state.set_root(false);
    state.start_scan();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = topcoat::serve(listener, router(state, AllowedHosts::for_bind("127.0.0.1"))).await;
    });
    addr
}

struct Response {
    status: u16,
    headers: String,
    body: String,
}

impl Response {
    fn header(&self, name: &str) -> Option<String> {
        self.headers.lines().find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.eq_ignore_ascii_case(name).then(|| v.trim().to_string())
        })
    }
}

async fn send(
    addr: SocketAddr,
    method: &str,
    path: &str,
    host: &str,
    body: Option<&str>,
) -> Response {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let body_part = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\nAccept: */*\r\n{}Content-Length: {}\r\n\r\n{body_part}",
        if body.is_some() {
            "Content-Type: application/x-www-form-urlencoded\r\n"
        } else {
            ""
        },
        body_part.len()
    );
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.unwrap();
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split(' ').nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    // Chunked bodies are tolerated: tests only assert on substrings.
    Response {
        status,
        headers: head.to_string(),
        body: body.to_string(),
    }
}

async fn get(addr: SocketAddr, path: &str) -> Response {
    send(addr, "GET", path, &addr.to_string(), None).await
}

async fn post(addr: SocketAddr, path: &str, form: &str) -> Response {
    send(addr, "POST", path, &addr.to_string(), Some(form)).await
}

async fn wait_for_scan(addr: SocketAddr) {
    for _ in 0..300 {
        let r = get(addr, "/api/status").await;
        if r.body.contains("\"scanning\":false") {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("scan did not finish");
}

#[tokio::test]
async fn home_renders_categories_and_the_user_root_split() {
    let addr = start().await;
    let r = get(addr, "/").await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert!(r.body.contains("CleanSys"));
    assert!(r.body.contains("User land"));
    assert!(
        r.body.contains("name=\"viewport\""),
        "responsive viewport meta"
    );
    assert!(r.body.contains("/style.css"));
    wait_for_scan(addr).await;
    let r = get(addr, "/").await;
    assert!(
        r.body.contains("System · root")
            || r.body.contains("System &middot; root")
            || r.body.contains("root")
    );
}

#[tokio::test]
async fn stylesheet_is_responsive() {
    let addr = start().await;
    let r = get(addr, "/style.css").await;
    assert_eq!(r.status, 200);
    assert!(r.body.contains("max-width:980px") && r.body.contains("max-width:700px"));
}

#[tokio::test]
async fn json_api_exposes_categories_status_and_themes() {
    let addr = start().await;
    wait_for_scan(addr).await;
    let cats = get(addr, "/api/categories").await;
    assert_eq!(cats.status, 200);
    assert!(cats.body.contains("\"User Land Cleaners\""));
    assert!(cats.body.contains("\"proj-rust\""));
    let health = get(addr, "/api/health").await;
    assert!(health.body.contains("ok"));
    let themes = get(addr, "/api/themes").await;
    assert!(themes.body.contains("Catppuccin Mocha"));
}

#[tokio::test]
async fn unknown_host_header_is_rejected_dns_rebinding_guard() {
    let addr = start().await;
    let r = send(addr, "GET", "/", "evil.example", None).await;
    assert_eq!(r.status, 403);
    let r = send(addr, "GET", "/api/categories", "evil.example:3000", None).await;
    assert_eq!(r.status, 403);
}

#[tokio::test]
async fn toggling_ticks_a_cleaner_and_redirects_back_safely() {
    let addr = start().await;
    wait_for_scan(addr).await;
    let r = post(addr, "/toggle", "id=proj-rust&back=%2F%3Fcat%3D2").await;
    assert_eq!(r.status, 303);
    assert_eq!(r.header("location").as_deref(), Some("/?cat=2"));
    let s = get(addr, "/api/status").await;
    assert!(s.body.contains("\"selected\":1"), "{}", s.body);

    // open redirects are refused
    let r = post(addr, "/toggle", "id=proj-rust&back=%2F%2Fevil.example").await;
    assert_eq!(r.header("location").as_deref(), Some("/"));
    let s = get(addr, "/api/status").await;
    assert!(s.body.contains("\"selected\":0"));
}

#[tokio::test]
async fn root_cleaners_cannot_be_ticked_without_root() {
    let addr = start().await;
    wait_for_scan(addr).await;
    let before = get(addr, "/api/status").await.body;
    let r = post(addr, "/toggle", "id=core-sys-system-logs&back=%2F").await;
    assert_eq!(r.status, 303);
    assert_eq!(get(addr, "/api/status").await.body, before);
}

#[tokio::test]
async fn search_filters_across_categories() {
    let addr = start().await;
    wait_for_scan(addr).await;
    let r = get(addr, "/?q=Rust+target").await;
    assert_eq!(r.status, 200);
    assert!(r.body.contains("Search results"));
    assert!(r.body.contains("Rust target/ Directories"));
}

#[tokio::test]
async fn confirm_preview_run_and_progress_clean_the_sandbox_project() {
    let addr = start().await;
    wait_for_scan(addr).await;
    let project = &sandbox().project;
    assert!(project.join("target").exists());

    // nothing selected -> confirm says so, run is a no-op
    let r = get(addr, "/confirm").await;
    assert!(r.body.contains("Nothing is selected"));

    post(addr, "/toggle", "id=proj-rust&back=%2F").await;
    let r = get(addr, "/preview").await;
    assert_eq!(r.status, 200);
    assert!(r.body.contains("Nothing has been deleted"));
    assert!(r.body.contains("demo-rust/target"));
    assert!(project.join("target").exists(), "preview must not delete");

    let r = get(addr, "/confirm").await;
    assert!(r.body.contains("Yes, clean now"));

    let r = post(addr, "/run", "").await;
    assert_eq!(r.status, 303);
    assert_eq!(r.header("location").as_deref(), Some("/progress"));
    for _ in 0..200 {
        if get(addr, "/api/status")
            .await
            .body
            .contains("\"phase\":\"done\"")
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!project.join("target").exists(), "target/ should be gone");
    assert!(project.join("Cargo.toml").exists(), "marker file must stay");
    let r = get(addr, "/progress").await;
    assert!(r.body.contains("Done") && r.body.contains("Rust target"));
    let r = post(addr, "/dismiss", "").await;
    assert_eq!(r.status, 303);
}

#[tokio::test]
async fn schedule_page_renders_the_form() {
    let addr = start().await;
    let r = get(addr, "/schedule").await;
    assert_eq!(r.status, 200);
    assert!(r.body.contains("Automatic cleaning"));
    assert!(r.body.contains("name=\"every\"") && r.body.contains("Recommended"));
}

#[tokio::test]
async fn cross_origin_post_is_refused() {
    let addr = start().await;
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let body = "id=proj-rust&back=%2F";
    let req = format!(
        "POST /toggle HTTP/1.1\r\nHost: {addr}\r\nOrigin: https://evil.example\r\nSec-Fetch-Site: cross-site\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).await.unwrap();
    assert!(raw.starts_with("HTTP/1.1 403"), "{raw}");
}

#[tokio::test]
async fn header_badge_switches_to_root_when_a_root_category_is_open() {
    let addr = start().await;
    wait_for_scan(addr).await;
    let cats: serde_json::Value =
        serde_json::from_str(&get(addr, "/api/categories").await.body).unwrap();
    let list = cats["categories"].as_array().unwrap();
    let user_idx = list.iter().position(|c| c["root"] == false).unwrap();
    // `hide=0`: on a CI box the root categories can be empty, and empty ones are hidden by default.
    let root_idx = list.iter().position(|c| c["root"] == true).unwrap();

    let user = get(addr, &format!("/?cat={user_idx}&hide=0")).await;
    assert!(user.body.contains("badge user"), "user land shows USER");
    assert!(!user.body.contains("System (root) section"));

    let root = get(addr, &format!("/?cat={root_idx}&hide=0")).await;
    assert!(
        root.body.contains("System (root) section"),
        "root category shows ROOT"
    );
    assert!(!root.body.contains("badge user"));
}
