//! JSON and static routes.

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        content::{Css, Json},
        route,
    },
};

use crate::{
    state::{Phase, Shared},
    style::CSS,
};

/// Every category and cleaner with its scan result: `/api/categories`.
#[route(GET "/api/categories")]
async fn categories(cx: &Cx) -> Result<Json<serde_json::Value>> {
    let st = app_context::<Shared>(cx).clone();
    st.poll();
    let snap = st.snapshot();
    let cats: Vec<_> = snap
        .categories
        .iter()
        .map(|c| {
            let items: Vec<_> = snap
                .items
                .iter()
                .filter(|i| i.category == c.index)
                .map(|i| {
                    serde_json::json!({
                        "id": i.id,
                        "name": i.name,
                        "description": i.description,
                        "risk": i.risk,
                        "requires_root": i.requires_root,
                        "selected": i.selected,
                        "bytes": i.scan.as_ref().map(|s| s.bytes),
                        "items": i.scan.as_ref().map(|s| s.items),
                        "error": i.scan.as_ref().and_then(|s| s.error.clone()),
                    })
                })
                .collect();
            serde_json::json!({
                "name": c.name,
                "description": c.description,
                "root": c.root,
                "bytes": c.bytes,
                "cleaners": items,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "categories": cats })))
}

/// Scan / selection / run status: `/api/status`.
#[route(GET "/api/status")]
async fn status(cx: &Cx) -> Result<Json<serde_json::Value>> {
    let st = app_context::<Shared>(cx).clone();
    st.poll();
    let s = st.snapshot();
    Ok(Json(serde_json::json!({
        "root": s.is_root,
        "scanning": s.scanning,
        "scan_done": s.scan_done,
        "scan_total": s.scan_total,
        "reclaimable_bytes": s.total_bytes,
        "selected": s.selected_count,
        "selected_bytes": s.selected_bytes,
        "run": {
            "phase": match s.run.phase { Phase::Idle => "idle", Phase::Running => "running", Phase::Done => "done" },
            "done": s.run.done,
            "total": s.run.total,
            "freed_bytes": s.run.freed,
        }
    })))
}

/// Liveness probe.
#[route(GET "/api/health")]
async fn health() -> Result<Json<&'static str>> {
    Ok(Json("ok"))
}

/// Every colour theme with its palette (the same ones the TUI and GUI offer).
#[route(GET "/api/themes")]
async fn themes() -> Result<Json<serde_json::Value>> {
    Ok(Json(crate::theme::catalogue()))
}

#[route(GET "/style.css")]
async fn style() -> Result<Css<&'static str>> {
    Ok(Css(CSS))
}
