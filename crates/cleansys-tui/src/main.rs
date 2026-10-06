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
    /// Preview what cleaners would free (deletes nothing)
    Scan {
        #[command(flatten)]
        select: SelectArgs,
        /// Machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
    /// One command: scan the recommended (safe) cleaners, show the result, ask once, clean
    Auto {
        #[command(flatten)]
        select: SelectArgs,
        /// Don't ask for confirmation
        #[arg(short, long)]
        yes: bool,
        /// Only report; delete nothing
        #[arg(short = 'n', long)]
        dry_run: bool,
        /// Machine-readable JSON output
        #[arg(long)]
        json: bool,
        /// Used by the OS scheduler: follow the saved schedule, never prompt
        #[arg(long, hide = true)]
        scheduled: bool,
    },
    /// Automatic cleaning on a schedule (systemd/cron, launchd, Task Scheduler)
    Schedule {
        #[command(subcommand)]
        action: ScheduleCmd,
    },
    /// Show or change engine settings (project scan roots, exclusions, ...)
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
    /// Run selected cleaners non-interactively
    Clean {
        #[command(flatten)]
        select: SelectArgs,
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
        /// Only report what would be removed
        #[arg(short = 'n', long)]
        dry_run: bool,
        /// Machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
    /// Interactive menu to select specific cleaners (text-based)
    Menu,
    /// Interactive terminal UI (default)
    Tui,
}

#[derive(Subcommand)]
enum ScheduleCmd {
    /// Show the current schedule and last run
    Show,
    /// Create or update the schedule (unspecified options keep their saved/default value)
    Install {
        /// daily | weekly | monthly
        #[arg(long, value_parser = ["daily", "weekly", "monthly"])]
        every: Option<String>,
        /// Time of day, HH:MM (24h)
        #[arg(long)]
        at: Option<String>,
        /// Weekly: weekday (mon, tue, ...); monthly: day of month (1-28)
        #[arg(long)]
        day: Option<String>,
        /// recommended (safe) | extended (safe + moderate) | selected (use --id)
        #[arg(long, value_parser = ["recommended", "extended", "selected"])]
        scope: Option<String>,
        /// Cleaner ids for --scope selected
        #[arg(short, long = "id", value_name = "ID")]
        ids: Vec<String>,
        /// auto | systemd | cron (Linux only)
        #[arg(long, value_parser = ["auto", "systemd", "cron"])]
        backend: Option<String>,
    },
    /// Remove the schedule
    Remove,
    /// Run the scheduled job right now
    RunNow,
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Print the current engine configuration
    Show,
    /// Add a directory to scan for projects (target/, build/, node_modules/ ...)
    AddRoot { path: String },
    /// Stop scanning a directory
    RemoveRoot { path: String },
    /// Never delete anything matching this glob (e.g. '~/work/keep-me/**')
    Exclude { pattern: String },
    /// Only clean project build output untouched for N days (0 = always)
    MinAge { days: u64 },
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
    /// The recommended preset: safe, user-land cleaners
    #[arg(short, long)]
    recommended: bool,
    /// With --recommended: also moderate-risk cleaners
    #[arg(long)]
    moderate: bool,
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
            recommended: a.recommended,
            include_moderate: a.moderate,
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

fn run_schedule_cmd(action: ScheduleCmd) -> Result<()> {
    use cleansys_core::engine::schedule::{
        self as sch, Backend, Frequency, LastRun, Schedule, Scope,
    };
    match action {
        ScheduleCmd::Show => {
            print_header("SCHEDULE");
            match (Schedule::load(), sch::installed()) {
                (Some(s), job) => {
                    println!("  When:     {}", s.describe());
                    println!("  Cleans:   {}", s.scope_label());
                    match job {
                        Some(j) => println!("  Active:   yes ({})", j.backend),
                        None => println!("  Active:   NO — saved but not installed; run `cleansys schedule install`"),
                    }
                    println!("  Log:      {}", sch::log_path().display());
                }
                (None, _) => {
                    println!("No schedule yet. Create one, e.g.:");
                    println!("  cleansys schedule install --every weekly --day sun --at 03:30");
                }
            }
            if let Some(l) = LastRun::load() {
                println!(
                    "  Last run: {} — freed {} ({} cleaners{})",
                    l.ago(),
                    cleansys_core::format_size(l.bytes_freed),
                    l.cleaners_run,
                    if l.scheduled { ", scheduled" } else { "" }
                );
            }
        }
        ScheduleCmd::Install {
            every,
            at,
            day,
            scope,
            ids,
            backend,
        } => {
            let mut s = Schedule::load().unwrap_or_default();
            if let Some(e) = every {
                s.frequency = match e.as_str() {
                    "daily" => Frequency::Daily,
                    "monthly" => Frequency::Monthly,
                    _ => Frequency::Weekly,
                };
            }
            if let Some(t) = at {
                (s.hour, s.minute) = sch::parse_time(&t)?;
            }
            if let Some(d) = day {
                match s.frequency {
                    Frequency::Weekly => s.weekday = sch::parse_weekday(&d)?,
                    Frequency::Monthly => s.day_of_month = d.parse()?,
                    Frequency::Daily => {}
                }
            }
            if let Some(sc) = scope {
                s.scope = match sc.as_str() {
                    "extended" => Scope::Extended,
                    "selected" => Scope::Selected,
                    _ => Scope::Recommended,
                };
            }
            if !ids.is_empty() {
                s.ids = ids;
                s.scope = Scope::Selected;
            }
            if let Some(b) = backend {
                s.backend = match b.as_str() {
                    "systemd" => Backend::Systemd,
                    "cron" => Backend::Cron,
                    _ => Backend::Auto,
                };
            }
            let job = sch::install(&s)?;
            cleansys_core::utils::print_success(&format!(
                "Scheduled: {} — {} (via {})",
                s.describe(),
                s.scope_label(),
                job.backend
            ));
            println!("  Test it now with: cleansys schedule run-now");
        }
        ScheduleCmd::Remove => {
            sch::remove()?;
            cleansys_core::utils::print_success("Schedule removed.");
        }
        ScheduleCmd::RunNow => {
            cleansys_core::engine::headless::run_scheduled()?;
        }
    }
    Ok(())
}

fn run_config_cmd(action: ConfigCmd) -> Result<()> {
    use cleansys_core::engine::EngineConfig;
    let mut cfg = EngineConfig::load();
    match action {
        ConfigCmd::Show => {
            print_header("ENGINE CONFIG");
            let roots = cfg.effective_roots();
            println!(
                "Project scan roots ({}):",
                if cfg.scan_roots.is_empty() {
                    "auto-detected"
                } else {
                    "configured"
                }
            );
            if roots.is_empty() {
                println!("  (none found — add one: cleansys config add-root ~/code)");
            }
            for r in roots {
                println!("  • {}", r.display());
            }
            println!("Max depth:     {}", cfg.max_depth);
            println!("Min age (days): {}", cfg.min_age_days);
            println!(
                "Excluded:      {}",
                if cfg.exclude.is_empty() {
                    "-".to_string()
                } else {
                    cfg.exclude.join(", ")
                }
            );
            println!("File:          {}", EngineConfig::path()?.display());
            return Ok(());
        }
        ConfigCmd::AddRoot { path } => {
            if cfg.scan_roots.is_empty() {
                // Start from what is auto-detected so adding one doesn't drop the rest.
                cfg.scan_roots = cfg
                    .effective_roots()
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
            }
            if !cfg.scan_roots.contains(&path) {
                cfg.scan_roots.push(path);
            }
        }
        ConfigCmd::RemoveRoot { path } => {
            if cfg.scan_roots.is_empty() {
                cfg.scan_roots = cfg
                    .effective_roots()
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
            }
            cfg.scan_roots.retain(|r| r != &path);
        }
        ConfigCmd::Exclude { pattern } => {
            if !cfg.exclude.contains(&pattern) {
                cfg.exclude.push(pattern);
            }
        }
        ConfigCmd::MinAge { days } => cfg.min_age_days = days,
    }
    cfg.save()?;
    cleansys_core::utils::print_success("Saved. See it with `cleansys config show`.");
    Ok(())
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

            println!("\nAll cleaners (ids work with `scan`/`clean --id`, ~ moderate, ! caution):");
            cleansys_core::engine::headless::print_list();
        }
        Some(Commands::Scan { select, json }) => {
            if !json {
                print_header("SCAN (preview)");
            }
            cleansys_core::engine::headless::run_selection(
                &select.into(),
                cleansys_core::RunOptions::preview(),
                json,
            )?;
        }
        Some(Commands::Clean {
            select,
            yes,
            dry_run,
            json,
        }) => {
            if !json {
                print_header(if dry_run { "CLEAN (dry run)" } else { "CLEAN" });
            }
            let opts = if dry_run {
                cleansys_core::RunOptions::preview()
            } else if yes {
                cleansys_core::RunOptions::execute()
            } else {
                cleansys_core::RunOptions::execute_with_confirmation()
            };
            cleansys_core::engine::headless::run_selection(&select.into(), opts, json)?;
        }
        Some(Commands::Auto {
            select,
            yes,
            dry_run,
            json,
            scheduled,
        }) => {
            if scheduled {
                cleansys_core::engine::headless::run_scheduled()?;
            } else {
                if !json {
                    print_header("AUTO CLEAN");
                }
                cleansys_core::engine::headless::run_auto(&select.into(), yes, dry_run, json)?;
            }
        }
        Some(Commands::Schedule { action }) => run_schedule_cmd(action)?,
        Some(Commands::Config { action }) => run_config_cmd(action)?,
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
