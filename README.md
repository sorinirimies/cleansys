# CleanSys - Modern Terminal-Based System Cleaner for Linux

[![Crates.io](https://img.shields.io/crates/v/cleansys.svg)](https://crates.io/crates/cleansys)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**CleanSys** is a modern, terminal-based utility for Linux that helps you safely clean your system. It provides a beautiful interactive TUI to remove unnecessary files, clean caches, and free up disk space with real-time animations and detailed progress tracking.

## ✨ Features

### 🎨 Modern Terminal UI
- **Beautiful Interface**: Built with [Ratatui](https://github.com/ratatui-org/ratatui) for a smooth, modern experience
- **Interactive Checkboxes**: Easy selection using [tui-checkbox](https://crates.io/crates/tui-checkbox) library
- **Multiple Chart Types**: Toggle between Bar Chart, Pie Chart (by count), and Pie Chart (by size)
- **Split-View Progress**: Detailed status information with real-time updates
- **Animated Indicators**: Loading spinners, progress bars, and status icons
- **Responsive Design**: Automatically adapts to any terminal size
- **Real-time Resize**: Handles terminal resizing without losing state

### 🧹 User-Level Cleaning
- Browser caches (Firefox, Chrome/Chromium)
- Application caches
- Thumbnail caches
- Temporary files
- Package manager caches (pip, npm, cargo)
- User trash

### 🔧 System-Level Cleaning (requires root)
- Package manager caches (apt, pacman, dnf, etc.)
- System logs
- System caches
- Temporary files
- Old kernels (on supported systems)
- Crash reports and core dumps

### 🛡️ Safe by Default
- Never removes system-critical files
- Confirms before running operations
- Detailed logs of all actions
- Shows exactly what will be cleaned
- Individual cleaner selection

## 📦 Installation

### From crates.io

```bash
cargo install cleansys
```

### From source

```bash
git clone https://github.com/sorinirimies/cleansys
cd cleansys
cargo install --path .
```

## 🚀 Usage

### Interactive TUI (Default)

Simply run CleanSys to launch the interactive terminal interface:

```bash
# User-level cleaning
cleansys

# System-level cleaning (requires root)
sudo cleansys
```

### Command-Line Interface

```bash
# Run terminal UI explicitly
cleansys tui

# Run text-based interactive menu
cleansys menu

# Run user-level cleaners with confirmation
cleansys user

# Run user-level cleaners without prompts
cleansys user --yes

# Run system-level cleaners (requires root)
sudo cleansys system

# Run system cleaners without prompts
sudo cleansys system --yes

# List all available cleaners
cleansys list

# Show verbose output
cleansys --verbose
```

## ⌨️ Terminal UI Controls

### Navigation
- `↑/↓` or `j/k`: Navigate items
- `Tab/Shift+Tab`: Switch between categories
- `j/k`: Scroll detailed items list (vi-style)
- `PgUp/PgDn`: Scroll operation log
- `Home/End`: Jump to first/last item

### Actions
- `Space`: Toggle selection
- `Enter`: Run selected cleaners
- `a`: Select all in current category
- `n`: Deselect all in current category
- `ESC`: Cancel operation or return to menu
- `q`: Exit application

### View Controls
- `c`: Cycle chart types (Bar → Pie Count → Pie Size)
- `m`: Toggle compact mode
- `v`: Cycle view modes (Standard/Compact/Detailed/Performance)
- `p`: Toggle performance statistics
- `s`: Toggle auto-scroll log
- `/`: Toggle search in detailed view
- `?`: Show/hide help

## 📱 Responsive Design

CleanSys features a fully responsive terminal interface with multiple breakpoints:

| Terminal Width | Layout Features |
|---------------|----------------|
| < 60 columns | Minimal UI, chart hidden, essential information only |
| 60-79 columns | Compact layout with reduced chart |
| 80-119 columns | Balanced layout with full chart |
| 120+ columns | Spacious layout with maximum information density |

## 🎯 View Modes

- **Standard Mode**: Balanced layout with full feature visibility (default)
- **Compact Mode**: Condensed layout for smaller terminals (<25 rows)
- **Detailed Mode**: Maximum information density with extended statistics
- **Performance Mode**: Focus on operation metrics and real-time monitoring

## 📊 Chart Visualization

Press `c` to cycle through different chart types:

1. **Bar Chart**: Traditional vertical bar chart showing cleaned items
2. **Pie Chart (Count)**: Distribution by number of items cleaned
3. **Pie Chart (Size)**: Distribution by bytes cleaned

All charts automatically adapt to terminal size and include:
- Percentages
- Legends
- Color coding
- Smart label positioning

## 🔍 Detailed View

After cleaning operations, view comprehensive details:
- Complete list of cleaned files and directories
- Full file paths
- Individual file sizes
- Timestamps
- Scrollable with `j/k` or `PgUp/PgDn`
- Search functionality with `/`

## 📝 Examples

### Interactive TUI

```bash
# Launch TUI (default behavior)
cleansys

# Navigate with arrow keys
# Select cleaners with Space
# Press Enter to run
```

### Quick Clean

```bash
# Clean user caches without prompts
cleansys user --yes

# Clean system caches with verbose output
sudo cleansys system --verbose --yes
```

### List Available Cleaners

```bash
cleansys list
```

Output:
```
AVAILABLE CLEANERS

User cleaners (no root required):
  • Browser Caches
  • Application Caches
  • Thumbnail Caches
  ...

System cleaners (root required):
  • Package Manager Caches
  • System Logs
  • System Caches
  ...
```

## 🏗️ Architecture

CleanSys is organized into clean, modular components:

```
src/
├── cleaners/          # Individual cleaner implementations
│   ├── user_cleaners.rs
│   └── system_cleaners.rs
├── utils/             # Utility functions (permissions, formatting)
├── app.rs             # Application state and logic
├── events.rs          # Event handling (keyboard, resize)
├── render.rs          # UI rendering logic
├── pie_chart.rs       # Chart visualization component
├── menu.rs            # Text-based interactive menu
├── main.rs            # Entry point and TUI setup
└── lib.rs             # Public API and documentation
```

## 🖥️ Platform Support

CleanSys supports Linux-based operating systems including:
- Ubuntu/Debian (apt-based)
- Arch Linux (pacman-based)
- Fedora/RHEL (dnf/yum-based)
- Other Linux distributions

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test module
cargo test --test integration_tests
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔗 Links

- [Repository](https://github.com/sorinirimies/cleansys)
- [Crates.io](https://crates.io/crates/cleansys)
- [Documentation](https://docs.rs/cleansys)

## 🙏 Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [tui-checkbox](https://crates.io/crates/tui-checkbox) - Checkbox widget library
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation

---

**Note**: Always review what will be cleaned before running system-level operations. While CleanSys is designed to be safe, it's good practice to understand what's being removed from your system.