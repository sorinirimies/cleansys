use anyhow::Result;
use clap::{Parser, Subcommand};
use log::debug;
use std::io;

use cleansys_core::utils::elevate_if_needed;
use cleansys_core::{check_root, print_error, print_header, system_cleaners, user_cleaners};
use cleansys_tui::app::App;
use cleansys_tui::events::{Config, Event, Events};
use cleansys_tui::menu::Menu;
use cleansys_tui::render::ui;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};

#[derive(Parser)]
#[command(
    name = "cleansys",
    author,
    version,
    about = "A modern terminal-based Linux system cleaner",
    long_about = "CleanSys is a Rust-based TUI tool that helps you clean your Linux system.
It provides an interactive terminal interface to select and clean user or system files.
System cleaners require root privileges."
)]
struct Cli {
    /// Verbose output mode
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Clean user-specific files and caches
    User {
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// Clean system files and caches (requires root)
    System {
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// List all available cleaners
    List,
    /// Preview what dev/AI/build cleaners would free (deletes nothing)
    Scan(SelectArgs),
    /// Run dev/AI/build cleaners non-interactively (BleachBit-style)
    Clean {
        #[command(flatten)]
        select: SelectArgs,
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
        /// Only report what would be removed
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// Interactive menu to select specific cleaners (text-based)
    Menu,
    /// Interactive terminal UI (default)
    Tui,
}

#[derive(clap::Args, Clone)]
struct SelectArgs {
    /// Cleaner ids (see `cleansys list`), e.g. proj-rust dev-gradle-caches
    #[arg(short, long = "id", value_name = "ID")]
    ids: Vec<String>,
    /// Whole category, e.g. "Project Build Artifacts" or "AI & LLM Caches"
    #[arg(short, long = "category", value_name = "NAME")]
    categories: Vec<String>,
    /// Every cleaner (caution-risk ones only with --include-caution)
    #[arg(short, long)]
    all: bool,
    /// Also include caution-risk cleaners (models, session histories)
    #[arg(long)]
    include_caution: bool,
}

impl From<SelectArgs> for cleansys_core::engine::headless::Selection {
    fn from(a: SelectArgs) -> Self {
        Self {
            ids: a.ids,
            categories: a.categories,
            include_caution: a.include_caution,
            all: a.all,
        }
    }
}

fn setup_logger(verbose: bool) {
    let env = env_logger::Env::default()
        .filter_or("CLEANSYS_LOG", if verbose { "debug" } else { "info" });
    env_logger::Builder::from_env(env)
        .format_timestamp(None)
        .init();
}

fn load_cleaners(app: &mut App) {
    app.categories = cleansys_core::load_categories();
}

fn run_tui() -> Result<()> {
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

    // Event loop with frequent ticks for smooth animations
    let events = Events::with_config(Config {
        tick_rate: std::time::Duration::from_millis(100),
    });

    let result = loop {
        // Draw UI
        if let Err(e) = terminal.draw(|f| ui(f, &mut app)) {
            break Err(e.into());
        }

        // Handle events
        match events.next() {
            Ok(Event::Input(key)) => match app.handle_key(key) {
                Ok(should_quit) => {
                    if should_quit {
                        break Ok(());
                    }
                }
                Err(e) => break Err(e),
            },
            Ok(Event::Tick) => {
                // Update animation frame on tick
                if app.is_running {
                    app.update_animation();
                }
            }
            Ok(Event::Resize(width, height)) => {
                // Handle terminal resize
                app.handle_resize(width, height);
                // Force immediate redraw on resize
                if let Err(e) = terminal.draw(|f| ui(f, &mut app)) {
                    break Err(e.into());
                }
            }
            Err(e) => break Err(e),
        }
    };

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logger(cli.verbose);
    debug!(
        "Starting CleanSys with arguments: {:?}",
        std::env::args().collect::<Vec<_>>()
    );

    let is_root = check_root();

    match cli.command {
        Some(Commands::User { yes }) => {
            print_header("USER CLEANER");
            user_cleaners::run_all(yes)?;
        }
        Some(Commands::System { yes }) => {
            print_header("SYSTEM CLEANER");
            if !is_root {
                // Prompt for elevation
                if !elevate_if_needed()? {
                    print_error("Cannot proceed without root privileges.");
                    return Ok(());
                }
                // After elevation, check if we now have root
                if !check_root() {
                    print_error("Elevation was approved but system cleaners still require sudo.");
                    println!("Please run: sudo cleansys system");
                    return Ok(());
                }
            }
            system_cleaners::run_all(yes)?;
        }
        Some(Commands::List) => {
            print_header("AVAILABLE CLEANERS");
            println!("\nUser cleaners (no root required):");
            for cleaner in user_cleaners::list_cleaners() {
                println!("  • {}", cleaner);
            }

            println!("\nSystem cleaners (some require root/admin):");
            for cleaner in system_cleaners::list_cleaners() {
                println!("  • {}", cleaner);
            }

            println!("\nDeveloper / build / AI cleaners (use with `scan` / `clean --id`):");
            cleansys_core::engine::headless::print_list();
        }
        Some(Commands::Scan(args)) => {
            print_header("SCAN (preview)");
            cleansys_core::engine::headless::run_selection(
                &args.into(),
                cleansys_core::RunOptions::preview(),
            )?;
        }
        Some(Commands::Clean {
            select,
            yes,
            dry_run,
        }) => {
            print_header(if dry_run { "CLEAN (dry run)" } else { "CLEAN" });
            let opts = if dry_run {
                cleansys_core::RunOptions::preview()
            } else if yes {
                cleansys_core::RunOptions::execute()
            } else {
                cleansys_core::RunOptions::execute_with_confirmation()
            };
            cleansys_core::engine::headless::run_selection(&select.into(), opts)?;
        }
        Some(Commands::Menu) => {
            let menu = Menu::new();
            menu.run_interactive()?;
        }
        Some(Commands::Tui) | None => {
            // Default behavior - show terminal UI
            run_tui()?;
        }
    }

    Ok(())
}
