# Cleansys - Modern Animated System Cleaner for Linux with TUI

[![Crates.io](https://img.shields.io/crates/v/cleansys.svg)](https://crates.io/crates/cleansys)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Cleansys** is a terminal-based utility for Linux that helps you safely clean your system. It provides a modern interface to remove unnecessary files, clean caches, and free up disk space.

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

## Responsive Design

Cleansys features a fully responsive terminal user interface that automatically adapts to different terminal sizes:

- **Dynamic Layout**: Automatically adjusts layout proportions based on terminal width and height
- **Responsive Chart**: Cleanup distribution chart scales, simplifies, or hides based on available space
- **Adaptive Content**: Text and controls adjust for optimal display on narrow or small terminals
- **Real-time Resize**: Handles terminal resize events instantly without losing state
- **Size Indicators**: Shows terminal dimensions on very small screens for debugging
- **Manual Controls**: Press `c` to manually toggle chart visibility

### Responsive Breakpoints

- **Very narrow** (< 60 columns): Chart hidden, minimal UI, essential information only
- **Narrow** (60-79 columns): Compact layout with reduced chart
- **Medium** (80-119 columns): Balanced layout with full chart
- **Wide** (120+ columns): Spacious layout with maximum information density

## Enhanced Features & Layout Improvements

Cleansys now includes significant layout improvements and new features for a professional terminal experience:

### 🎨 Multiple View Modes
- **Standard Mode**: Balanced layout with full feature visibility (default)
- **Compact Mode**: Condensed layout for smaller terminals (<25 rows)
- **Detailed Mode**: Maximum information density with extended statistics
- **Performance Mode**: Focus on operation metrics and real-time monitoring

### 🔧 Layout Fixes
- **Resolved Overlapping Text**: Complete elimination of text overlap issues
- **Smart Spacing**: Proper margins and content separation
- **Dynamic Constraints**: Layout adapts perfectly to any terminal size
- **Content Prioritization**: Essential information always visible

### 📊 Enhanced Progress Display & Pie Chart Canvas
- **Real-time Statistics**: Live operation counters and progress tracking
- **Interactive Pie Charts**: ASCII art pie charts showing item distribution by count or size
- **Chart Type Cycling**: Press 'c' to cycle between Bar Chart, Pie Count, and Pie Size views
- **Responsive Visualization**: Charts adapt automatically to terminal size with side-by-side or stacked layouts
- **Professional Log**: Scrollable operation history with timestamps
- **Detailed Cleaned Items View**: Complete list of every file/directory cleaned with paths, sizes, and timestamps
- **Status Indicators**: Visual icons for different operation states
- **Performance Metrics**: Optional detailed statistics and timing

### ⌨️ Advanced Keyboard Controls
- `c`: Cycle chart types (Count Pie → Size Pie → Bar Chart → Count Pie)
- `m`: Toggle compact mode
- `v`: Cycle view modes (Standard/Compact/Detailed/Performance)
- `l`: Toggle detailed cleaned items list with file paths and sizes
- `p`: Toggle performance statistics
- `s`: Toggle auto-scroll log (during operations)
- `o`: Cycle sort modes
- `f`: Cycle filter modes
- `y`: Toggle confirmation prompts
- `x`: Clear all errors
- `j/k`: Scroll detailed items list (vi-style navigation)
- `Ctrl+Space`: Pause/Resume operations
- `PgUp/PgDn`: Scroll operation log
- `Home/End`: Jump to first/last item

### 🚀 Robustness Features
- **Error Recovery**: Comprehensive error tracking and clearing
- **Detailed Audit Trail**: Complete record of all cleaned files with paths, sizes, and timestamps
- **State Management**: Persistent settings across operations
- **Memory Optimization**: Efficient rendering and bounded log buffers
- **Pause/Resume**: Control over long-running operations

## Installation

### From crates.io

```bash
cargo install cleansys
```

### From source

```bash
git clone https://github.com/sorin/cleansys
cd cleansys
cargo install --path .
```

## Usage

```
# Run terminal UI (default)
cleansys

# Run terminal UI with root privileges
sudo cleansys

# Run terminal UI explicitly
cleansys tui

# Run text-based interactive menu
cleansys menu
cleansys --interactive

# Run specific cleaners
cleansys user  # Run all user-level cleaners
sudo cleansys system  # Run all system-level cleaners

# List all available cleaners
cleansys list

# Run without confirmation prompts
cleansys --yes
sudo cleansys --yes

# Run all cleaners (both user and system, requires root)
sudo cleansys --all

# Show verbose output
cleansys --verbose
```

## Examples

Using the Terminal UI:

```bash
cleansys
# Navigate with arrow keys, select with Space, run with Enter
```

Using the text-based interactive menu:

```bash
cleansys menu
# Then select options by entering numbers (e.g., 1,3,5)
```

Clean user caches without prompts:

```bash
cleansys user --yes
```

Clean system caches with verbose output:

```bash
sudo cleansys system --verbose
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
  - l: Toggle detailed cleaned items list
  - c: Toggle chart visibility

- **Progress Screen**
  - ESC: Cancel operation (while running) or Return to main menu (when completed)
  - q: Cancel current operation or quit application
  - Features animated spinners, progress bar, and elapsed time tracking
  - Visual indicators for success (✓), errors (✗), and running operations (animated spinner)
  - Remains visible after operations complete for review - press ESC to return to menu

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