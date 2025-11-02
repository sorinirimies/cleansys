# CleanSys Demo - Modern System Cleaner for Linux
# This VHS tape showcases the features and capabilities of CleanSys

Output demo-showcase.gif

Set FontSize 13
Set Width 1200
Set Height 700
Set Padding 15
Set Theme "Catppuccin Mocha"
Set TypingSpeed 80ms

# Clear and setup
Hide
Type "cd /home/sorin/Projects/cleansys && clear"
Enter
Sleep 500ms
Show

# Title card
Type "# 🧹 CleanSys - Modern System Cleaner for Linux"
Enter
Sleep 1s
Type "# Interactive TUI for cleaning system and user caches"
Enter
Sleep 2s
Type ""
Enter
Sleep 500ms

# Show basic info
Type "# Version:"
Enter
Type "cargo run --release -- --version"
Enter
Sleep 2s

Type ""
Enter
Sleep 500ms

# Show available commands
Type "# Available Commands:"
Enter
Type "cargo run --release -- --help"
Enter
Sleep 4s

Type "clear"
Enter
Sleep 500ms

# List cleaners
Type "# 📋 Available Cleaners:"
Enter
Sleep 500ms
Type "cargo run --release -- list"
Enter
Sleep 5s

Type "clear"
Enter
Sleep 500ms

# Feature highlights
Type "# ✨ Key Features:"
Enter
Sleep 500ms
Type ""
Enter
Type "# 🎨 Interactive TUI"
Enter
Type "#    • Real-time progress tracking"
Enter
Type "#    • Beautiful charts and statistics"
Enter
Type "#    • Responsive design"
Enter
Sleep 1s
Type ""
Enter
Type "# 🔒 Secure Password Prompt"
Enter
Type "#    • System cleaners require authentication"
Enter
Type "#    • Password masked during entry"
Enter
Type "#    • Seamless sudo integration"
Enter
Sleep 1s
Type ""
Enter
Type "# 🐧 Multi-Distribution Support"
Enter
Type "#    • Debian/Ubuntu (apt)"
Enter
Type "#    • Fedora/RHEL (dnf)"
Enter
Type "#    • Arch Linux (pacman)"
Enter
Sleep 1s
Type ""
Enter
Type "# 🧼 Smart Cleaning"
Enter
Type "#    • User cleaners (no root needed)"
Enter
Type "#    • System cleaners (with authentication)"
Enter
Type "#    • Safe kernel cleanup"
Enter
Type "#    • Browser cache removal"
Enter
Sleep 2s

Type "clear"
Enter
Sleep 500ms

# Show clean command examples
Type "# 🚀 Quick Start Examples:"
Enter
Sleep 500ms
Type ""
Enter
Type "# Clean user caches (no sudo needed):"
Enter
Type "cargo run --release -- user --yes"
Enter
Sleep 3s

Type ""
Enter
Type "# Interactive TUI (default):"
Enter
Type "# cargo run --release"
Enter
Sleep 2s

Type ""
Enter
Type "# Text-based menu:"
Enter
Type "# cargo run --release -- menu"
Enter
Sleep 2s

Type "clear"
Enter
Sleep 500ms

# Installation info
Type "# 📦 Installation:"
Enter
Sleep 500ms
Type ""
Enter
Type "cargo install cleansys"
Enter
Sleep 500ms
Type ""
Enter
Type "# Or build from source:"
Enter
Type "git clone https://github.com/sorinirimies/cleansys"
Enter
Type "cd cleansys"
Enter
Type "cargo build --release"
Enter
Sleep 2s

Type "clear"
Enter
Sleep 500ms

# Final screen
Type "# 🎉 CleanSys - Keep Your Linux System Clean!"
Enter
Sleep 500ms
Type ""
Enter
Type "# 🌟 Star us on GitHub:"
Enter
Type "#    https://github.com/sorinirimies/cleansys"
Enter
Sleep 500ms
Type ""
Enter
Type "# 📚 Documentation:"
Enter
Type "#    cargo doc --open"
Enter
Sleep 500ms
Type ""
Enter
Type "# 🐛 Report Issues:"
Enter
Type "#    https://github.com/sorinirimies/cleansys/issues"
Enter
Sleep 500ms
Type ""
Enter
Type "# 💬 Features:"
Enter
Type "#    ✅ Password prompt for system operations"
Enter
Type "#    ✅ Multi-distribution kernel cleanup"
Enter
Type "#    ✅ Real-time progress tracking"
Enter
Type "#    ✅ No demo data - real results only"
Enter
Type "#    ✅ Clean, modern TUI interface"
Enter
Sleep 3s

Type ""
Enter
Type "# Thanks for watching! 🚀"
Enter
Sleep 2s
