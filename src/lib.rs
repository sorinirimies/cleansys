//! # CleanSys - Modern System Cleaner for Linux
//!
//! `cleansys` is a terminal-based utility for Linux that helps you safely clean your system.
//! It provides a modern, interactive interface to remove unnecessary files, clean caches,
//! and free up disk space with beautiful animations and real-time progress tracking.
//!
//! ## Overview
//!
//! CleanSys offers both a modern Terminal User Interface (TUI) and a command-line interface
//! for cleaning various system and user-level caches and temporary files. It's designed to be
//! safe by default, never removing files that would break your system.
//!
//! ## Features
//!
//! ### Modern Terminal UI
//! - Beautiful interactive interface built with [Ratatui](https://github.com/ratatui-org/ratatui)
//! - Interactive checkbox-based selection using [tui-checkbox](https://crates.io/crates/tui-checkbox)
//! - Split-view progress screen with detailed status information
//! - Animated loading spinners and progress indicators
//! - Multiple chart types: bar charts, pie charts (by count or size)
//! - Responsive design that adapts to terminal size
//! - Real-time resize handling
//!
//! ### User-Level Cleaning
//! - Browser caches (Firefox, Chrome/Chromium)
//! - Application caches
//! - Thumbnail caches
//! - Temporary files owned by the user
//! - Package manager caches (pip, npm, cargo)
//! - User trash
//!
//! ### System-Level Cleaning (requires root)
//! - Package manager caches (apt, pacman, dnf, etc.)
//! - System logs
//! - System caches
//! - Temporary files
//! - Old kernels (on supported systems)
//! - Crash reports and core dumps
//!
//! ## Quick Start
//!
//! ### As a Binary
//!
//! Install from crates.io:
//!
//! ```bash
//! cargo install cleansys
//! ```
//!
//! Run the interactive TUI:
//!
//! ```bash
//! # User-level cleaning
//! cleansys
//!
//! # System-level cleaning (requires root)
//! sudo cleansys
//! ```
//!
//! ### Command-Line Usage
//!
//! ```bash
//! # Run terminal UI (default)
//! cleansys
//!
//! # Run text-based interactive menu
//! cleansys menu
//!
//! # Run specific cleaners
//! cleansys user          # User-level cleaners
//! sudo cleansys system   # System-level cleaners
//!
//! # List all available cleaners
//! cleansys list
//!
//! # Run without confirmation prompts
//! cleansys --yes
//!
//! # Show verbose output
//! cleansys --verbose
//! ```
//!
//! ## Terminal UI Controls
//!
//! ### Navigation
//! - `↑/↓`: Navigate items
//! - `Tab/Shift+Tab`: Switch categories
//! - `j/k`: Scroll detailed items list (vi-style)
//! - `PgUp/PgDn`: Scroll operation log
//! - `Home/End`: Jump to first/last item
//!
//! ### Actions
//! - `Space`: Toggle selection
//! - `Enter`: Run selected cleaners
//! - `a`: Select all in current category
//! - `n`: Deselect all in current category
//! - `ESC`: Cancel operation or return to menu
//! - `q`: Exit application
//!
//! ### View Controls
//! - `c`: Cycle chart types (Bar → Pie Count → Pie Size)
//! - `m`: Toggle compact mode
//! - `v`: Cycle view modes (Standard/Compact/Detailed/Performance)
//! - `p`: Toggle performance statistics
//! - `s`: Toggle auto-scroll log
//! - `/`: Toggle search in detailed view
//! - `?`: Show/hide help
//!
//! ## Responsive Design
//!
//! CleanSys features a fully responsive terminal user interface with multiple breakpoints:
//!
//! - **Very narrow** (< 60 columns): Chart hidden, minimal UI, essential information only
//! - **Narrow** (60-79 columns): Compact layout with reduced chart
//! - **Medium** (80-119 columns): Balanced layout with full chart
//! - **Wide** (120+ columns): Spacious layout with maximum information density
//!
//! ## Safety
//!
//! CleanSys is designed to be safe by default:
//! - Never removes system-critical files
//! - Confirms before running operations
//! - Provides detailed logs of all actions
//! - Shows exactly what will be cleaned before execution
//! - Allows individual cleaner selection
//!
//! ## Categories
//!
//! This crate falls under the following categories:
//! - **Command-line utilities**: Interactive CLI tool
//! - **Operating systems**: Linux system maintenance
//! - **System cleanup**: Cache and temporary file removal
//! - **System administration**: System maintenance and optimization
//! - **Development tools**: Developer cache cleanup (cargo, npm, pip)
//!
//! ## Architecture
//!
//! The crate is organized into several modules:
//!
//! - `cleaners`: Individual cleaner implementations for different types of files
//! - `ui`: Terminal user interface components (TUI and menu)
//! - `utils`: Utility functions for permissions, formatting, and error handling
//!
//! ## Platform Support
//!
//! Currently, CleanSys supports Linux-based operating systems including:
//! - Ubuntu/Debian (apt-based)
//! - Arch Linux (pacman-based)
//! - Fedora/RHEL (dnf/yum-based)
//! - Other Linux distributions
//!
//! ## Examples
//!
//! ### Running the TUI
//!
//! The default behavior launches the interactive Terminal UI:
//!
//! ```bash
//! cleansys
//! ```
//!
//! Navigate with arrow keys, select cleaners with Space, and run with Enter.
//!
//! ### Running Specific Cleaners
//!
//! ```bash
//! # Clean user caches without prompts
//! cleansys user --yes
//!
//! # Clean system caches with verbose output
//! sudo cleansys system --verbose
//! ```
//!
//! ### Listing Available Cleaners
//!
//! ```bash
//! cleansys list
//! ```
//!
//! ## License
//!
//! This project is licensed under the MIT License.
//!
//! ## Contributing
//!
//! Contributions are welcome! Please feel free to submit a Pull Request.
//!
//! ## Links
//!
//! - [Repository](https://github.com/sorinirimies/cleansys)
//! - [Crates.io](https://crates.io/crates/cleansys)
//! - [Documentation](https://docs.rs/cleansys)

// Only require documentation for the main crate-level docs and public API
// Internal implementation details don't need exhaustive documentation
#![allow(missing_docs)]
#![doc(html_root_url = "https://docs.rs/cleansys/0.2.1")]

/// Cleaner implementations for system and user-level cleanup operations
pub mod cleaners;

/// Menu system for text-based interactive interface
pub mod menu;

/// Terminal user interface components
pub mod ui;

/// Utility functions for permissions, formatting, and error handling
pub mod utils;

/// Re-export commonly used types for convenience
pub use cleaners::{system_cleaners, user_cleaners};
pub use menu::Menu;
pub use utils::{check_root, print_error, print_header};
