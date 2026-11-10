//! "Where did my disk go?" browser (`U`): a drill-down size view with proportional bars.
//!
//! Measuring runs on a background thread (read-only, one drive, bounded — see
//! [`cleansys_core::engine::usage`]); ↑/↓ move, Enter/→ open a folder, ←/Backspace go up.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};

use cleansys_core::engine::config::EngineConfig;
use cleansys_core::engine::trash::{Mode, TRASH_SUPPORTED, delete_entry, warning_for};
use cleansys_core::engine::usage::{Node, disk_usage};
use cleansys_core::format_size;

/// Entries listed per folder.
const TOP: usize = 200;

/// A delete waiting for the user's confirmation.
#[derive(Debug, Clone)]
pub struct PendingDelete {
    pub path: PathBuf,
    pub bytes: u64,
    pub is_dir: bool,
    pub mode: Mode,
    pub warning: Option<&'static str>,
}

#[derive(Debug)]
pub struct UsageView {
    /// Delete confirmation, when asked for.
    pub pending: Option<PendingDelete>,
    /// Result of the last delete (or why it was refused).
    pub message: Option<String>,
    /// Folder being shown (or measured).
    pub path: PathBuf,
    pub node: Option<Node>,
    pub cursor: usize,
    rx: Option<Receiver<Node>>,
    /// Folders above this one: `(measured node, cursor)`, so going up is instant.
    stack: Vec<(Node, usize)>,
}

impl UsageView {
    pub fn open(path: PathBuf) -> Self {
        let mut v = Self {
            pending: None,
            message: None,
            path: path.clone(),
            node: None,
            cursor: 0,
            rx: None,
            stack: Vec::new(),
        };
        v.measure(path);
        v
    }

    fn measure(&mut self, path: PathBuf) {
        let (tx, rx) = channel();
        self.path = path.clone();
        self.node = None;
        self.cursor = 0;
        self.rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(disk_usage(&path, 1, TOP));
        });
    }

    /// Ask to delete the highlighted entry (trash by default, where there is one).
    pub fn request_delete(&mut self) {
        self.message = None;
        let Some(c) = self.node.as_ref().and_then(|n| n.children.get(self.cursor)) else {
            return;
        };
        self.pending = Some(PendingDelete {
            path: c.path.clone(),
            bytes: c.bytes,
            is_dir: c.is_dir,
            mode: if TRASH_SUPPORTED {
                Mode::Trash
            } else {
                Mode::Permanent
            },
            warning: warning_for(&c.path),
        });
    }

    /// Key while the confirmation is open. `y`/Enter confirms, `p` toggles permanent,
    /// anything else cancels.
    pub fn confirm_key(&mut self, key: crossterm::event::KeyCode, cfg: &EngineConfig) {
        use crossterm::event::KeyCode;
        let Some(p) = self.pending.as_mut() else {
            return;
        };
        match key {
            KeyCode::Char('p' | 'P') if TRASH_SUPPORTED => {
                p.mode = if p.mode == Mode::Trash {
                    Mode::Permanent
                } else {
                    Mode::Trash
                };
            }
            KeyCode::Char('y' | 'Y') | KeyCode::Enter => {
                let p = p.clone();
                self.pending = None;
                self.perform(&p, cfg);
            }
            _ => self.pending = None,
        }
    }

    fn perform(&mut self, p: &PendingDelete, cfg: &EngineConfig) {
        match delete_entry(&p.path, p.mode, cfg) {
            Ok(_) => {
                let what = if p.mode == Mode::Trash {
                    "Moved to trash"
                } else {
                    "Deleted"
                };
                self.message = Some(format!(
                    "{what}: {} ({} freed)",
                    p.path.display(),
                    format_size(p.bytes)
                ));
                // Keep the numbers honest without re-measuring.
                if let Some(n) = self.node.as_mut()
                    && let Some(i) = n.children.iter().position(|c| c.path == p.path)
                {
                    n.children.remove(i);
                    n.bytes = n.bytes.saturating_sub(p.bytes);
                }
                for (anc, _) in &mut self.stack {
                    anc.bytes = anc.bytes.saturating_sub(p.bytes);
                }
                self.move_cursor(0);
            }
            Err(e) => self.message = Some(format!("Not deleted: {e}")),
        }
    }

    pub fn loading(&self) -> bool {
        self.node.is_none()
    }

    /// Pick up a finished measurement. `true` if the view changed.
    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.rx else { return false };
        match rx.try_recv() {
            Ok(n) => {
                self.node = Some(n);
                self.rx = None;
                true
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.rx = None;
                self.node = Some(Node {
                    path: self.path.clone(),
                    ..Node::default()
                });
                true
            }
            Err(_) => false,
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let len = self.node.as_ref().map_or(0, |n| n.children.len());
        if len > 0 {
            self.cursor = (self.cursor as i32 + delta).clamp(0, len as i32 - 1) as usize;
        }
    }

    /// Open the highlighted folder.
    pub fn descend(&mut self) {
        let Some(node) = self.node.take() else { return };
        match node.children.get(self.cursor) {
            Some(c) if c.is_dir => {
                let next = c.path.clone();
                self.stack.push((node, self.cursor));
                self.measure(next);
            }
            _ => self.node = Some(node),
        }
    }

    /// Back to the folder above.
    pub fn ascend(&mut self) {
        if let Some((node, cursor)) = self.stack.pop() {
            self.path = node.path.clone();
            self.node = Some(node);
            self.cursor = cursor;
            self.rx = None;
        } else if let Some(parent) = self.path.parent().map(PathBuf::from) {
            self.measure(parent);
        }
    }
}

pub fn render(f: &mut Frame, view: &UsageView, area: Rect) {
    let inner = crate::charts::popup(
        f,
        area,
        90,
        90,
        " Disk usage ",
        " ↑/↓ move · Enter/→ open · ←/Bksp up · d delete · Esc close ",
    );
    let t = crate::theme::th();
    let w = inner.width as usize;
    let mut lines: Vec<Line> = Vec::new();
    let head = format!(
        "{}  {}",
        view.node
            .as_ref()
            .map_or_else(|| "measuring…".to_string(), |n| format_size(n.bytes)),
        view.path.display()
    );
    lines.push(Line::from(Span::styled(
        clip(&head, w),
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
    )));
    let Some(node) = &view.node else {
        lines.push(Line::from(Span::styled(
            "Measuring this folder (it can take a moment on big drives)…",
            Style::default().fg(t.muted),
        )));
        f.render_widget(Paragraph::new(lines), inner);
        return;
    };
    if node.children.is_empty() && node.hidden == 0 {
        lines.push(Line::from(Span::styled(
            "Empty (or unreadable).",
            Style::default().fg(t.muted),
        )));
    }
    // Columns: size(10) bar(bar_w) pct(5) name
    let bar_w = (w / 4).clamp(6, 24);
    let name_w = w.saturating_sub(10 + bar_w + 5 + 5);
    let total = node.bytes.max(1);
    let visible = inner.height.saturating_sub(2) as usize;
    let start = view.cursor.saturating_sub(visible.saturating_sub(1));
    for (i, c) in node.children.iter().enumerate().skip(start).take(visible) {
        let frac = c.bytes as f64 / total as f64;
        let filled = ((frac * bar_w as f64).round() as usize).min(bar_w);
        let name = c
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = if c.is_dir { format!("{name}/") } else { name };
        let selected = i == view.cursor;
        let style = if selected {
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.foreground)
        };
        lines.push(Line::from(vec![
            Span::styled(if selected { "▶" } else { " " }, style),
            Span::styled(format!("{:>9} ", format_size(c.bytes)), style),
            Span::styled("█".repeat(filled), Style::default().fg(t.accent)),
            Span::styled("░".repeat(bar_w - filled), Style::default().fg(t.muted)),
            Span::styled(
                format!(" {:>3.0}% ", frac * 100.0),
                Style::default().fg(t.muted),
            ),
            Span::styled(clip(&name, name_w), style),
        ]));
    }
    if node.hidden > 0 {
        lines.push(Line::from(Span::styled(
            format!("  … {} in smaller entries", format_size(node.hidden)),
            Style::default().fg(t.muted),
        )));
    }
    if let Some(m) = &view.message {
        lines.push(Line::from(Span::styled(
            clip(m, w),
            Style::default().fg(t.warning),
        )));
    }
    f.render_widget(Paragraph::new(lines), inner);
    if let Some(p) = &view.pending {
        render_confirm(f, p, area);
    }
}

fn render_confirm(f: &mut Frame, p: &PendingDelete, area: Rect) {
    use ratatui::widgets::{Block, BorderType, Borders, Clear, Wrap};
    let t = crate::theme::th();
    let w = area.width.min(70);
    let h = area.height.min(11);
    let rect = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    );
    f.render_widget(Clear, rect);
    let permanent = p.mode == Mode::Permanent;
    let block = Block::default()
        .title(if permanent {
            " Delete permanently? "
        } else {
            " Move to trash? "
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if permanent { t.error } else { t.warning }))
        .style(Style::default().bg(crate::theme::bg(&t)).fg(t.foreground));
    let inner = block.inner(rect);
    f.render_widget(block, rect);
    let mut lines = vec![
        Line::from(Span::styled(
            p.path.display().to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "{} · {}",
            format_size(p.bytes),
            if p.is_dir {
                "folder and everything in it"
            } else {
                "file"
            }
        )),
    ];
    if let Some(w) = p.warning {
        lines.push(Line::from(Span::styled(w, Style::default().fg(t.warning))));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        if permanent {
            "Gone for good — this cannot be undone."
        } else {
            "You can restore it from the trash."
        },
        Style::default().fg(if permanent { t.error } else { t.muted }),
    )));
    let mut keys = String::from("y/Enter confirm · any other key cancels");
    if TRASH_SUPPORTED {
        keys.push_str(if permanent {
            " · p: use trash"
        } else {
            " · p: delete permanently"
        });
    }
    lines.push(Line::from(Span::styled(
        keys,
        Style::default().fg(t.accent),
    )));
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn clip(s: &str, max: usize) -> String {
    if unicode_width::UnicodeWidthStr::width(s) <= max {
        return s.to_string();
    }
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if w + cw + 1 > max {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(v: &mut UsageView) {
        for _ in 0..1000 {
            if v.poll() || !v.loading() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("measuring never finished");
    }

    #[test]
    fn descend_then_ascend_restores_the_parent_and_cursor() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("big")).unwrap();
        std::fs::create_dir_all(d.path().join("small")).unwrap();
        std::fs::write(d.path().join("big/f"), vec![0u8; 2_000_000]).unwrap();
        std::fs::write(d.path().join("small/f"), vec![0u8; 10_000]).unwrap();
        let mut v = UsageView::open(d.path().to_path_buf());
        settle(&mut v);
        assert!(v.node.as_ref().unwrap().children[0].path.ends_with("big"));
        v.move_cursor(1);
        v.descend();
        settle(&mut v);
        assert!(v.path.ends_with("small"));
        v.ascend();
        assert_eq!(v.path, d.path());
        assert_eq!(v.cursor, 1);
        assert!(!v.loading());
    }

    #[test]
    fn delete_needs_confirmation_and_updates_the_numbers() {
        use crossterm::event::KeyCode;
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("big")).unwrap();
        std::fs::write(d.path().join("big/f"), vec![0u8; 2_000_000]).unwrap();
        std::fs::write(d.path().join("small"), vec![0u8; 50_000]).unwrap();
        let cfg = EngineConfig::default();
        let mut v = UsageView::open(d.path().to_path_buf());
        settle(&mut v);
        let before = v.node.as_ref().unwrap().bytes;
        // cancel: nothing happens
        v.request_delete();
        assert!(v.pending.is_some());
        v.confirm_key(KeyCode::Char('n'), &cfg);
        assert!(v.pending.is_none() && d.path().join("big").exists());
        // permanent (never touch the real trash in a test)
        v.request_delete();
        if TRASH_SUPPORTED {
            v.confirm_key(KeyCode::Char('p'), &cfg);
        }
        assert_eq!(v.pending.as_ref().unwrap().mode, Mode::Permanent);
        v.confirm_key(KeyCode::Char('y'), &cfg);
        assert!(!d.path().join("big").exists());
        let n = v.node.as_ref().unwrap();
        assert_eq!(n.children.len(), 1);
        assert!(n.bytes < before);
        assert!(v.message.as_deref().unwrap().starts_with("Deleted"));
    }

    #[test]
    fn excluded_entries_are_refused() {
        use crossterm::event::KeyCode;
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("keep"), vec![0u8; 50_000]).unwrap();
        let cfg = EngineConfig {
            exclude: vec![d.path().join("keep").to_string_lossy().into_owned()],
            ..EngineConfig::default()
        };
        let mut v = UsageView::open(d.path().to_path_buf());
        settle(&mut v);
        v.request_delete();
        if TRASH_SUPPORTED {
            v.confirm_key(KeyCode::Char('p'), &cfg);
        }
        v.confirm_key(KeyCode::Char('y'), &cfg);
        assert!(v.message.as_deref().unwrap().starts_with("Not deleted"));
        assert!(d.path().join("keep").exists());
    }
}
