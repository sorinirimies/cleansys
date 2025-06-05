pub mod app;
pub mod events;
pub mod tui;
pub mod ui;

use crate::cleaners::{system_cleaners, user_cleaners};
use anyhow::Result;
use app::{App, CleanerCategory, CleanerItem};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::{Event, Events};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::io;
use ui::ui;

pub fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Load cleaners into app
    load_cleaners(&mut app);

    // Event loop with more frequent ticks for smoother animations
    let events = Events::with_config(events::Config {
        tick_rate: std::time::Duration::from_millis(100),
    });

    loop {
        // Draw UI
        terminal.draw(|f| ui::<CrosstermBackend<io::Stdout>>(f, &mut app))?;

        // Handle events
        match events.next()? {
            Event::Input(key) => {
                if app.handle_key(key)? {
                    break;
                }
            }
            Event::Tick => {
                // Update animation frame on tick
                if app.is_running {
                    app.update_animation();
                }
            }
            Event::Resize(width, height) => {
                // Handle terminal resize
                app.handle_resize(width, height);
                // Force immediate redraw on resize
                terminal.draw(|f| ui::<CrosstermBackend<io::Stdout>>(f, &mut app))?;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn load_cleaners(app: &mut App) {
    // Add user cleaners
    let mut user_items = Vec::new();
    for cleaner in user_cleaners::get_cleaners() {
        user_items.push(CleanerItem {
            name: cleaner.name.to_string(),
            description: cleaner.description.to_string(),
            requires_root: false,
            selected: false,
            function: cleaner.function,
            bytes_cleaned: 0,
            status: None,
        });
    }

    // Add system cleaners
    let mut system_items = Vec::new();
    for cleaner in system_cleaners::get_cleaners() {
        system_items.push(CleanerItem {
            name: cleaner.name.to_string(),
            description: cleaner.description.to_string(),
            requires_root: true,
            selected: false,
            function: cleaner.function,
            bytes_cleaned: 0,
            status: None,
        });
    }

    app.categories = vec![
        CleanerCategory {
            name: "User Land Cleaners".to_string(),
            description: "Clean user-specific files and caches".to_string(),
            items: user_items,
        },
        CleanerCategory {
            name: "System Cleaners".to_string(),
            description: "Clean system files and caches (requires root)".to_string(),
            items: system_items,
        },
    ];
}
