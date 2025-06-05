use anyhow::Result;
use clap::{Parser, Subcommand};
use log::debug;

mod cleaners;
mod menu;
mod ui;
mod utils;

use cleaners::{system_cleaners, user_cleaners};
use menu::Menu;
use utils::{check_root, print_error, print_header};

#[derive(Parser)]
#[command(
    name = "cleansys",
    author,
    version,
    about = "A simple CLI tool to clean your Linux system",
    long_about = "CleanSys is a Rust-based CLI tool that helps you clean your Linux system.
When run with sudo, it cleans system caches and other safe-to-remove items.
When run without sudo, it cleans user directories and caches."
)]
struct Cli {
    /// Run all cleaners without prompting
    #[arg(short, long)]
    yes: bool,

    /// Verbose output mode
    #[arg(short, long)]
    verbose: bool,

    /// Clean both user and system (requires root)
    #[arg(short, long)]
    all: bool,

    /// Use interactive menu (text-based)
    #[arg(short, long)]
    interactive: bool,

    /// Use terminal UI interface (default)
    #[arg(short, long, default_value = "true")]
    tui: bool,

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
    /// Interactive menu to select specific cleaners (text-based)
    Menu,
    /// Interactive terminal UI
    Tui,
}

fn setup_logger(verbose: bool) {
    let env = env_logger::Env::default()
        .filter_or("CLEANSYS_LOG", if verbose { "debug" } else { "info" });
    env_logger::Builder::from_env(env)
        .format_timestamp(None)
        .init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logger(cli.verbose);
    debug!(
        "Starting CleanSys with arguments: {:?}",
        std::env::args().collect::<Vec<_>>()
    );

    let is_root = check_root();

    // If interactive flag is passed, override command with Menu
    // If no command is specified and tui is true, use Tui command
    let command = if cli.interactive {
        Some(Commands::Menu)
    } else if cli.command.is_none() && cli.tui {
        Some(Commands::Tui)
    } else {
        cli.command
    };

    match command {
        Some(Commands::User { yes }) => {
            print_header("USER CLEANER");
            user_cleaners::run_all(yes)?;
        }
        Some(Commands::System { yes }) => {
            print_header("SYSTEM CLEANER");
            if !is_root {
                print_error("System cleaning requires root privileges. Please run with sudo.");
                return Ok(());
            }
            system_cleaners::run_all(yes)?;
        }
        Some(Commands::List) => {
            print_header("AVAILABLE CLEANERS");
            println!("\nUser cleaners (no root required):");
            for cleaner in user_cleaners::list_cleaners() {
                println!("  • {}", cleaner);
            }

            println!("\nSystem cleaners (root required):");
            for cleaner in system_cleaners::list_cleaners() {
                println!("  • {}", cleaner);
            }
        }
        Some(Commands::Menu) => {
            let menu = Menu::new();
            menu.run_interactive()?;
        }
        Some(Commands::Tui) => {
            ui::run_tui()?;
        }
        None => {
            // Default behavior - show terminal UI
            ui::run_tui()?;
        }
    }

    Ok(())
}
