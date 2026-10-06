//! CleanSys web — the same cleaners, sizes, scheduler and safety rules as the TUI and
//! GUI, as a server-rendered local web UI built with
//! [Topcoat](https://github.com/tokio-rs/topcoat).
//!
//! | module | what |
//! |---|---|
//! | [`app`] | [`ServerOptions`], router, DNS-rebinding guard, [`serve`] |
//! | [`state`] | shared state: scan results, selection, the clean run (framework-free) |
//! | [`pages`] | the pages and form routes |
//! | [`components`] | reusable view components |
//! | [`api`] | JSON + static routes |
//! | [`style`] | the responsive stylesheet |
//! | [`theme`] | the core's 43 colour themes as CSS variables |
//! | [`util`] | URL / formatting helpers |
//!
//! It needs no JavaScript bundle: every action is a plain HTML form (progressively
//! enhanced with one-line `onchange` submits), pages refresh themselves while a scan or
//! a clean is running, and Topcoat's built-in cross-origin check plus a `Host` guard
//! protect the destructive routes.

#![allow(clippy::too_many_arguments)]

pub mod api;
pub mod app;
pub mod components;
pub mod pages;
pub mod state;
pub mod style;
pub mod theme;
pub mod util;

pub use app::{AllowedHosts, ServerOptions, router, serve};
pub use state::Shared;
