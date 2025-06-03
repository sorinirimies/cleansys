# CleanMySystem - Modern Animated System Cleaner for Linux with TUI

[![Crates.io](https://img.shields.io/crates/v/clean_my_system.svg)](https://crates.io/crates/clean_my_system)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**CleanMySystem** is a terminal-based utility for Linux that helps you safely clean your system. It provides a modern interface to remove unnecessary files, clean caches, and free up disk space.

## Features

- **Modern Terminal UI**: Beautiful interactive interface with Ratatui
- **Interactive Selection**: Easy to use checkbox-based selection interface
- **Progress Tracking**: Split-view progress screen with detailed status information
- **Animated Loading**: Real-time loading spinners and progress indicators
- **Detailed Status Icons**: Visual indicators showing operation status (running, success, error)
- **User Land Cleaning**:
  - Browser caches (Firefox, Chrome/Chromium)
  - Application caches
  - Thumbnail caches
  - Temporary files owned by the user
  - Package manager caches (pip, npm, cargo)
  - User trash

- **System-Level Cleaning** (requires root):
  - Package manager caches (apt, pacman, dnf, etc.)
  - System logs
  - System caches
  - Temporary files
  - Old kernels (on supported systems)
  - Crash reports and core dumps

- **Safe by Default**: Never removes files that would break your system
- **Interactive**: Confirms before running each cleaner
- **Verbose**: Shows detailed information about space freed

## Installation

### From crates.io

```bash
cargo install clean_my_system
```

### From source

```bash
git clone https://github.com/sorin/clean_my_system
cd clean_my_system
cargo install --path .
```

## Usage

```
# Run terminal UI (default)
clean_my_system

# Run terminal UI with root privileges
sudo clean_my_system

# Run terminal UI explicitly
clean_my_system tui

# Run text-based interactive menu
clean_my_system menu
clean_my_system --interactive

# Run specific cleaners
clean_my_system user  # Run all user-level cleaners
sudo clean_my_system system  # Run all system-level cleaners

# List all available cleaners
clean_my_system list

# Run without confirmation prompts
clean_my_system --yes
sudo clean_my_system --yes

# Run all cleaners (both user and system, requires root)
sudo clean_my_system --all

# Show verbose output
clean_my_system --verbose
```

## Examples

Using the Terminal UI:

```bash
clean_my_system
# Navigate with arrow keys, select with Space, run with Enter
```

Using the text-based interactive menu:

```bash
clean_my_system menu
# Then select options by entering numbers (e.g., 1,3,5)
```

Clean user caches without prompts:

```bash
clean_my_system user --yes
```

Clean system caches with verbose output:

```bash
sudo clean_my_system system --verbose
```

## Terminal UI Controls

The Terminal UI provides an intuitive interface with dynamic animations and the following controls:

- **Navigation**
  - ↑/↓: Navigate items
  - Tab/Shift+Tab: Switch categories
  - q: Exit application from main menu

- **Actions**
  - Space: Toggle selection
  - Enter: Run selected cleaners
  - a: Select all in current category
  - n: Deselect all in current category
  - d: Toggle detailed view

- **Progress Screen**
  - ESC: Return to main menu after operation
  - q: Cancel current operation
  - Features animated spinners, progress bar, and elapsed time tracking
  - Visual indicators for success (✓), errors (✗), and running operations (animated spinner)

- **Other**
  - ?: Show/hide help
  - q: Exit application

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.