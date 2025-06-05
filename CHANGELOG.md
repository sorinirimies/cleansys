# Changelog

All notable changes to this project will be documented in this file.

## [0.11.0] - 2023-11-10

### Features
- Renamed section headers to "👤 User Land" and "🔒 Root Cleaners" for clarity
- Added numerous Unicode icons throughout the interface for better visual cues
- Expanded chart size for better visualization of cleaning trends
- Improved visual hierarchy with consistent icon usage
- Added elapsed time information to individual cleaner details
- Enhanced overall UI with more descriptive icons and section titles

## [0.10.0] - 2023-11-05

### Features
- Added ability to cancel sudo operations with Ctrl+C
- Implemented clear visual indicators for cancellation option
- Added user-friendly messages about cancellation in terminal
- Enhanced error handling for cancelled sudo operations
- Improved user experience by allowing exit during password prompt

## [0.9.0] - 2023-11-01

### Features
- Updated System Cleaner items to use consistent color scheme with User Land Cleaners
- Standardized on green color for all percentage indicators and category headers
- Improved visual consistency between different cleaner types
- Enhanced readability with uniform color scheme throughout the interface

## [0.8.0] - 2023-10-30

### Features
- Added sudo password prompt for system cleaners requiring root privileges
- Implemented proper detection and handling of root vs non-root operations
- Added visual indicators for operations requiring root permissions
- Enhanced UI with clear warnings about root requirements
- Improved error handling for unauthorized system operations
- Added ability to handle mixed system and user cleaners in the same session

## [0.7.0] - 2023-10-25

### Features
- Merged running cleaning operations with cleaner details for unified view
- Added percentage indicators showing proportion of total space cleaned by each area
- Enhanced info graph with color-coded percentage breakdown
- Improved operation status with real-time progress indicators
- Added detailed percentage breakdown for each cleaning category
- Added per-item percentage contribution to total space freed
- Combined details and operations into a single scrollable interface
- Enhanced visual feedback with more descriptive status indicators

## [0.6.0] - 2023-10-20

### Features
- Reorganized cleaning information with hierarchical details view
- Added colorful info graph showing cleanup trend
- Moved all cleaned items to scrollable cleaner details section
- Implemented category-based organization with section headers
- Added hierarchical indented view with bullet points for individual items
- Enhanced visual separation with proper spacing between categories
- Improved navigation with clear scrolling indicators
- Added color-coded status messages and progress indicators

## [0.5.0] - 2023-10-15

### Features
- Completely redesigned progress screen with cleaner layout
- Unified cleaning progress view with left-aligned cleaned items list
- Added dedicated cleaner details section showing category information
- Improved progress bar with full-width display
- Enhanced time tracking display at both top and bottom of screen
- Added detailed cleaned items log showing individual operations
- Fixed formatting of status indicators and counters
- Implemented exact match to system aesthetics for consistent appearance

## [0.4.0] - 2023-10-01

### Features
- Renamed "User Cleaners" to "User Land Cleaners" for clarity
- Added split-view progress screen with real-time status information
- Improved cleaning operation feedback with detailed progress tracking
- Added ability to return to main menu after operation completion
- Enhanced UI rendering for better readability during operations
- Added explicit "Exit" option in the main menu
- Added animated loading spinners during cleaning operations
- Added visual progress bar with completion percentage
- Added elapsed time tracking during operations
- Improved status icons for better visibility of operation states
- Added detailed error tracking and reporting

## [0.3.0] - 2023-09-25

### Features
- Added modern terminal user interface (TUI) using Ratatui
- Implemented visual checkbox-based item selection
- Added detailed view for cleaner information
- Added category-based navigation with Tab key
- Added keyboard shortcuts for common operations
- Improved visual feedback during cleaning operations

## [0.2.0] - 2023-09-15

### Features
- Added new unified interactive menu for all cleaners
- Implemented multiple selection functionality for cleaning operations
- Improved user interface with color-coded options
- Made the interactive menu the default interface

## [0.1.0] - 2023-09-01

### Features
- Initial release of CleanSys
- User-level cleaning capabilities for browser caches, application caches, thumbnails, temp files, package caches, and trash
- System-level cleaning capabilities for package manager caches, system logs, system caches, temp files, old kernels, and crash reports
- Interactive CLI with confirmation prompts
- Support for various Linux distributions
- Verbose output mode showing space saved

<!-- generated by git-cliff -->
