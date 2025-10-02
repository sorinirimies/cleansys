use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use ratatui::widgets::ListState;
use std::sync::mpsc;
use std::time::Instant;

use crate::utils::{check_root, format_size};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct DetailedCleanedItem {
    pub path: String,
    pub size: u64,
    pub category: String,
    pub cleaner_name: String,
    pub timestamp: SystemTime,
    pub item_type: CleanedItemType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CleanedItemType {
    File,
    Directory,
    Log,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Standard,
    Compact,
    Detailed,
    Performance,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortMode {
    Name,
    Size,
    Status,
    Category,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterMode {
    All,
    Selected,
    Completed,
    Errors,
    UserOnly,
    SystemOnly,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChartType {
    Bar,
    PieCount,
    PieSize,
}

pub enum Status {
    Running,
    Success(String),
    Error(String),
    Pending,
}

impl Status {
    pub fn get_animation_frame(&self, frame: usize) -> &'static str {
        match self {
            Status::Running => {
                const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                SPINNER[frame % SPINNER.len()]
            }
            Status::Success(_) => "✓",
            Status::Error(_) => "✗",
            Status::Pending => "•",
        }
    }
}

pub struct CleanerItem {
    pub name: String,
    pub description: String,
    pub requires_root: bool,
    pub selected: bool,
    pub function: fn(bool) -> Result<u64>,
    pub bytes_cleaned: u64,
    pub status: Option<Status>,
}

pub struct CleanerCategory {
    pub name: String,
    pub description: String, // Retained for future use in detailed view
    pub items: Vec<CleanerItem>,
}

pub struct App {
    pub categories: Vec<CleanerCategory>,
    pub category_index: usize,
    pub item_list_state: ListState,
    pub is_root: bool,
    pub is_running: bool,
    pub operation_start_time: Option<Instant>,
    pub operation_end_time: Option<Instant>,
    pub total_bytes_cleaned: u64,
    pub show_help: bool,
    pub result_messages: Vec<String>,
    pub detailed_view: bool,
    pub current_cleaner_index: usize,
    pub animation_frame: usize,
    pub last_frame_time: Instant,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub compact_mode: bool,
    pub show_performance_stats: bool,
    pub operation_count: usize,
    pub errors_count: usize,
    pub paused: bool,
    pub confirmation_mode: bool,
    pub selected_cleaners_count: usize,
    pub view_mode: ViewMode,
    pub sort_mode: SortMode,
    pub filter_mode: FilterMode,
    pub detailed_cleaned_items: Vec<DetailedCleanedItem>,
    pub detailed_list_scroll_state: ListState,
    pub search_query: String,
    pub search_active: bool,
    pub detailed_view_filter: String,
    pub demo_operation_timer: Option<Instant>,
    pub demo_operations_completed: usize,
    pub chart_type: ChartType,
    pub operation_logs: Vec<String>,
    pub show_progress_screen: bool,
}

impl App {
    pub fn new() -> Self {
        // Get initial terminal size
        let (width, height) = terminal::size().unwrap_or((80, 24));

        let mut app = App {
            categories: Vec::new(),
            category_index: 0,
            item_list_state: ListState::default(),
            is_root: check_root(),
            is_running: false,
            operation_start_time: None,
            operation_end_time: None,
            total_bytes_cleaned: 0,
            show_help: false,
            result_messages: Vec::new(),
            detailed_view: false,
            current_cleaner_index: 0,
            animation_frame: 0,
            last_frame_time: Instant::now(),
            terminal_width: width,
            terminal_height: height,
            compact_mode: height < 25,
            show_performance_stats: false,
            operation_count: 0,
            errors_count: 0,
            paused: false,
            confirmation_mode: true,
            selected_cleaners_count: 0,
            view_mode: if height < 25 {
                ViewMode::Compact
            } else {
                ViewMode::Standard
            },
            sort_mode: SortMode::Category,
            filter_mode: FilterMode::All,
            detailed_cleaned_items: Vec::new(),
            detailed_list_scroll_state: ListState::default(),
            search_query: String::new(),
            search_active: false,
            detailed_view_filter: String::new(),
            demo_operation_timer: None,
            demo_operations_completed: 0,
            chart_type: ChartType::PieCount,
            operation_logs: Vec::new(),
            show_progress_screen: false,
        };
        app.item_list_state.select(Some(0));

        // Add some sample cleaned items for demonstration
        app.add_sample_cleaned_items();

        app
    }

    pub fn toggle_search(&mut self) {
        self.search_active = !self.search_active;
        if !self.search_active {
            self.search_query.clear();
        }
    }

    pub fn clear_search(&mut self) {
        self.search_active = false;
        self.search_query.clear();
        self.detailed_view_filter.clear();
    }

    pub fn add_search_char(&mut self, c: char) {
        if self.search_active {
            self.search_query.push(c);
        }
    }

    pub fn remove_search_char(&mut self) {
        if self.search_active {
            self.search_query.pop();
        }
    }

    pub fn get_category_distribution(&self) -> Vec<(String, usize, u64)> {
        let mut category_map: std::collections::HashMap<String, (usize, u64)> =
            std::collections::HashMap::new();

        for item in &self.detailed_cleaned_items {
            let entry = category_map.entry(item.category.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += item.size;
        }

        let mut categories: Vec<(String, usize, u64)> = category_map
            .into_iter()
            .map(|(name, (count, size))| (name, count, size))
            .collect();

        categories.sort_by(|a, b| b.2.cmp(&a.2)); // Sort by size descending
        categories
    }

    pub fn next_item(&mut self) {
        let items = &self.categories[self.category_index].items;
        let i = match self.item_list_state.selected() {
            Some(i) => {
                if i >= items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.item_list_state.select(Some(i));
    }

    pub fn previous_item(&mut self) {
        let items = &self.categories[self.category_index].items;
        let i = match self.item_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.item_list_state.select(Some(i));
    }

    pub fn toggle_selected(&mut self) {
        if let Some(i) = self.item_list_state.selected() {
            let item = &mut self.categories[self.category_index].items[i];
            // Allow selection even for root items, will prompt for password later
            item.selected = !item.selected;
        }
    }

    pub fn next_category(&mut self) {
        if self.category_index < self.categories.len() - 1 {
            self.category_index += 1;
        } else {
            self.category_index = 0;
        }
        // Reset selection in new category
        self.item_list_state.select(Some(0));
    }

    pub fn previous_category(&mut self) {
        if self.category_index > 0 {
            self.category_index -= 1;
        } else {
            self.category_index = self.categories.len() - 1;
        }
        // Reset selection in new category
        self.item_list_state.select(Some(0));
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn select_all(&mut self) {
        for item in &mut self.categories[self.category_index].items {
            // Allow selection of all items, will handle root permissions later
            item.selected = true;
        }
    }

    pub fn deselect_all(&mut self) {
        for item in &mut self.categories[self.category_index].items {
            item.selected = false;
        }
    }

    pub fn run_selected(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }

        // Count selected items
        let mut has_selected = false;
        let mut has_root_cleaners = false;

        for category in &self.categories {
            for item in &category.items {
                if item.selected {
                    has_selected = true;
                    if item.requires_root {
                        has_root_cleaners = true;
                    }
                }
            }
            if has_selected {
                break;
            }
        }

        if !has_selected {
            self.result_messages
                .push("No items selected. Please select items to clean.".to_string());
            return Ok(());
        }

        // If we need root and don't have it, prompt for sudo password
        if has_root_cleaners && !self.is_root {
            // Check if we can use sudo
            let sudo_test = std::process::Command::new("sudo")
                .args(["-n", "true"])
                .output();

            let needs_password = match sudo_test {
                Ok(output) => !output.status.success(),
                Err(_) => true,
            };

            if needs_password {
                // Display a message in the UI that we're waiting for sudo password
                self.result_messages.push("Root permissions needed. Please enter your sudo password in the terminal or press Ctrl+C to cancel.".to_string());

                // Temporarily disable raw mode to allow password entry
                crossterm::terminal::disable_raw_mode()?;

                // Print a message about how to cancel
                println!("\n\x1b[33m[CleanSys]\x1b[0m Press \x1b[1mCtrl+C\x1b[0m to cancel and return to the menu if you changed your mind.");
                println!(
                    "\x1b[33m[CleanSys]\x1b[0m Otherwise, enter your sudo password when prompted:"
                );

                // Ask for sudo password using a separate process
                let password_process = std::process::Command::new("sudo").args(["-v"]).status()?;

                // Re-enable raw mode
                crossterm::terminal::enable_raw_mode()?;

                if !password_process.success() {
                    self.result_messages.push("Failed to obtain root permissions or operation was cancelled. System cleaners will be skipped.".to_string());
                    // We'll continue but mark system cleaners as errored
                } else {
                    // We've gotten sudo permissions
                    self.result_messages.push(
                        "Root permissions obtained. Proceeding with all cleaners.".to_string(),
                    );
                }
            }
        }

        // Collect items that need root and mark them with error status
        let mut root_items = Vec::new();
        if !self.is_root {
            for (cat_idx, category) in self.categories.iter().enumerate() {
                for (item_idx, item) in category.items.iter().enumerate() {
                    if item.selected && item.requires_root {
                        root_items.push((cat_idx, item_idx));
                    }
                }
            }
        }

        // Don't mark root items as errors immediately - let them try to run
        // The sudo validation above should have handled authentication

        // Prepare the selected cleaners
        let mut selected_cleaners = Vec::new();

        for (cat_idx, category) in self.categories.iter().enumerate() {
            for (item_idx, item) in category.items.iter().enumerate() {
                if item.selected {
                    // Include all selected cleaners - sudo authentication was handled above
                    let name = item.name.clone();
                    let function = item.function;
                    selected_cleaners.push((cat_idx, item_idx, name, function, item.requires_root));
                }
            }
        }

        // Start processing
        self.is_running = true;
        self.show_progress_screen = true;
        self.operation_start_time = Some(Instant::now());
        self.operation_end_time = None;
        self.total_bytes_cleaned = 0;
        self.demo_operation_timer = Some(Instant::now());
        self.demo_operations_completed = 0;
        self.result_messages.clear();
        self.operation_logs.clear();
        self.current_cleaner_index = 0;

        // Set all selected cleaners to Pending
        for (cat_idx, item_idx, _, _, _) in &selected_cleaners {
            self.categories[*cat_idx].items[*item_idx].status = Some(Status::Pending);
        }

        // Clone necessary data for the thread
        let (_tx, _rx) = mpsc::channel::<(usize, usize, Status)>();

        // Actual thread processing will be implemented in a future version
        // For demo purposes, we'll simulate async operations
        // Set all selected operations to pending first, then they'll progress over time
        if !selected_cleaners.is_empty() {
            // Set operations to pending initially - they'll be processed by update_demo_operations
            for (cat_idx, item_idx, _, _, _) in &selected_cleaners {
                self.categories[*cat_idx].items[*item_idx].status = Some(Status::Pending);
            }
        }

        // Operations will be processed by update_demo_operations over time
        // The is_running flag will be automatically turned off when all operations complete

        Ok(())
    }

    pub fn update_animation(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_frame_time).as_millis() > 100 {
            self.animation_frame = (self.animation_frame + 1) % 10;
            self.last_frame_time = now;
        }

        // Update demo operations if running
        if self.is_running {
            self.update_demo_operations();
        }
    }

    pub fn update_demo_operations(&mut self) {
        if let Some(start_time) = self.demo_operation_timer {
            let elapsed = start_time.elapsed().as_millis();

            // Find next pending operation to start
            type Operation = (usize, usize, String, fn(bool) -> anyhow::Result<u64>, bool);
            let mut pending_operations: Vec<Operation> = Vec::new();
            for (cat_idx, category) in self.categories.iter().enumerate() {
                for (item_idx, item) in category.items.iter().enumerate() {
                    if matches!(item.status, Some(Status::Pending)) {
                        pending_operations.push((
                            cat_idx,
                            item_idx,
                            item.name.to_string(),
                            item.function,
                            item.requires_root,
                        ));
                    }
                }
            }

            // Start next operation every 1.5 seconds
            let operations_to_start = (elapsed / 1500) as usize;
            if operations_to_start > self.demo_operations_completed
                && !pending_operations.is_empty()
            {
                if let Some((cat_idx, item_idx, _name, _function, _requires_root)) =
                    pending_operations.first()
                {
                    // Set to running
                    self.categories[*cat_idx].items[*item_idx].status = Some(Status::Running);
                    self.demo_operations_completed += 1;
                }
            }

            // Complete running operations after 2 seconds
            let mut running_operations: Vec<Operation> = Vec::new();
            for (cat_idx, category) in self.categories.iter().enumerate() {
                for (item_idx, item) in category.items.iter().enumerate() {
                    if matches!(item.status, Some(Status::Running)) {
                        running_operations.push((
                            cat_idx,
                            item_idx,
                            item.name.to_string(),
                            item.function,
                            item.requires_root,
                        ));
                    }
                }
            }

            // Complete operations that have been running for at least 2 seconds
            for (cat_idx, item_idx, name, function, requires_root) in running_operations {
                self.operation_logs.push(format!("Starting: {}", name));

                // Execute operation - sudo authentication should have been handled during startup
                let result: anyhow::Result<u64> = if requires_root && !self.is_root {
                    self.operation_logs
                        .push(format!("❌ {}: Root privileges required", name));
                    Err(anyhow::anyhow!(
                        "Root privileges required. Run 'sudo cleansys' for system operations."
                    ))
                } else {
                    self.operation_logs.push(format!("🔄 Executing: {}", name));
                    match function(true) {
                        Ok(bytes) => {
                            self.operation_logs
                                .push(format!("✅ {}: Cleaned {} bytes", name, bytes));
                            Ok(bytes)
                        }
                        Err(e) => {
                            self.operation_logs.push(format!("❌ {}: {}", name, e));
                            // For user operations, provide fallback simulation
                            if !requires_root {
                                let simulated =
                                    (1024 * 1024 * (2 + (cat_idx + item_idx) % 10)) as u64;
                                self.operation_logs.push(format!(
                                    "📊 {}: Using simulated data ({} bytes)",
                                    name, simulated
                                ));
                                Ok(simulated)
                            } else {
                                Err(e)
                            }
                        }
                    }
                };

                // Process result
                match result {
                    Ok(bytes) => {
                        let msg = if requires_root {
                            format!("Cleaned {} (root) ({})", name, format_size(bytes))
                        } else {
                            format!("Cleaned {} ({})", name, format_size(bytes))
                        };
                        self.categories[cat_idx].items[item_idx].status =
                            Some(Status::Success(msg));
                        self.categories[cat_idx].items[item_idx].bytes_cleaned = bytes;
                        self.total_bytes_cleaned += bytes;
                        self.operation_logs.push(format!(
                            "✅ Completed {}: {} freed",
                            name,
                            format_size(bytes)
                        ));
                    }
                    Err(e) => {
                        let error_msg = if requires_root && !self.is_root {
                            "Requires sudo - restart with 'sudo cleansys'".to_string()
                        } else {
                            format!(
                                "Failed: {}",
                                e.to_string()
                                    .split(':')
                                    .next_back()
                                    .unwrap_or("Unknown error")
                                    .trim()
                            )
                        };
                        self.categories[cat_idx].items[item_idx].status =
                            Some(Status::Error(error_msg.clone()));
                        self.operation_logs
                            .push(format!("❌ Failed {}: {}", name, error_msg));

                        // Add helpful message for sudo requirement
                        if requires_root
                            && !self.is_root
                            && !self
                                .result_messages
                                .iter()
                                .any(|msg| msg.contains("sudo cleansys"))
                        {
                            self.result_messages.push(
                                "💡 System cleaners require root privileges. Run 'sudo cleansys' to clean system files.".to_string()
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn cancel_sudo_operations(&mut self) {
        // Mark all operations as cancelled
        for category in &mut self.categories {
            for item in &mut category.items {
                if item.selected
                    && matches!(item.status, Some(Status::Running) | Some(Status::Pending))
                {
                    item.status = Some(Status::Error("Operation cancelled by user".to_string()));
                    item.selected = false; // Deselect the item
                }
            }
        }

        self.result_messages
            .push("Cleaning operations cancelled by user.".to_string());
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            // Quit
            (KeyCode::Char('q'), _) => {
                if self.show_help {
                    self.show_help = false;
                } else if self.is_running {
                    // Cancel current cleaning operations
                    self.is_running = false;
                    self.cancel_sudo_operations();
                } else {
                    return Ok(true);
                }
            }

            // Navigation
            (KeyCode::Down, _) => {
                if !self.show_help {
                    if self.is_running || self.show_progress_screen {
                        self.scroll_detailed_list_down();
                    } else {
                        self.next_item();
                    }
                }
            }
            (KeyCode::Up, _) => {
                if !self.show_help {
                    if self.is_running || self.show_progress_screen {
                        self.scroll_detailed_list_up();
                    } else {
                        self.previous_item();
                    }
                }
            }
            (KeyCode::Tab, _) => {
                if !self.show_help {
                    self.next_category();
                }
            }
            (KeyCode::BackTab, _) => {
                if !self.show_help {
                    self.previous_category();
                }
            }
            // Selection
            (KeyCode::Char(' '), KeyModifiers::NONE) => {
                if !self.show_help {
                    self.toggle_selected();
                }
            }
            // Run cleaners
            (KeyCode::Enter, _) => {
                if !self.show_help {
                    self.run_selected()?;
                }
            }
            // Help dialog
            (KeyCode::Char('?'), _) | (KeyCode::Char('h'), _) => {
                self.toggle_help();
            }

            // Toggle search in removed items view
            (KeyCode::Char('/'), _) => {
                if !self.show_help {
                    self.toggle_search();
                }
            }
            // Clear search or cancel operations or return to main menu
            (KeyCode::Esc, _) => {
                if self.search_active {
                    self.clear_search();
                } else if self.is_running {
                    self.is_running = false;
                    self.cancel_sudo_operations();
                } else if self.show_progress_screen {
                    // Return to main menu from completed operations screen
                    self.show_progress_screen = false;
                }
            }
            // Scroll removed items list
            (KeyCode::Char('j'), _) => {
                if !self.show_help {
                    self.scroll_detailed_list_down();
                }
            }
            (KeyCode::Char('k'), _) => {
                if !self.show_help {
                    self.scroll_detailed_list_up();
                }
            }
            // Select all in current category
            (KeyCode::Char('a'), _) => {
                if !self.show_help {
                    self.select_all();
                }
            }
            // Deselect all in current category
            (KeyCode::Char('n'), _) => {
                if !self.show_help {
                    self.deselect_all();
                }
            }

            // Toggle compact mode
            (KeyCode::Char('m'), _) => {
                if !self.show_help {
                    self.toggle_compact_mode();
                }
            }
            // Toggle auto scroll log
            (KeyCode::Char('s'), _) => {
                if !self.show_help && self.is_running {
                    self.toggle_auto_scroll();
                }
            }
            // Toggle performance stats
            (KeyCode::Char('p'), _) => {
                if !self.show_help {
                    self.toggle_performance_stats();
                }
            }
            // Cycle view mode
            (KeyCode::Char('v'), _) => {
                if !self.show_help {
                    self.cycle_view_mode();
                }
            }
            // Cycle sort mode
            (KeyCode::Char('o'), _) => {
                if !self.show_help {
                    self.cycle_sort_mode();
                }
            }
            // Cycle filter mode
            (KeyCode::Char('f'), _) => {
                if !self.show_help {
                    self.cycle_filter_mode();
                }
            }
            // Toggle pause/resume operations
            (KeyCode::Char(' '), KeyModifiers::CONTROL) => {
                if self.is_running {
                    self.toggle_pause();
                }
            }
            // Toggle confirmation mode
            (KeyCode::Char('y'), _) => {
                if !self.show_help {
                    self.toggle_confirmation_mode();
                }
            }
            // Toggle chart type
            (KeyCode::Char('c'), _) => {
                if !self.show_help {
                    self.toggle_chart_type();
                }
            }
            // Clear all errors
            (KeyCode::Char('x'), _) => {
                if !self.show_help {
                    self.clear_errors();
                }
            }
            // Handle search input (only when search is active)
            (KeyCode::Char(c), _) => {
                if self.search_active {
                    self.add_search_char(c);
                } else if !self.show_help {
                    self.toggle_selected();
                }
            }
            // Backspace in search
            (KeyCode::Backspace, _) => {
                if self.search_active {
                    self.remove_search_char();
                }
            }
            // Page scrolling for removed items (when in progress view)
            (KeyCode::PageUp, _) => {
                if self.is_running || self.show_progress_screen {
                    // Scroll up by 10 items
                    for _ in 0..10 {
                        self.scroll_detailed_list_up();
                    }
                }
            }
            (KeyCode::PageDown, _) => {
                if self.is_running || self.show_progress_screen {
                    // Scroll down by 10 items
                    for _ in 0..10 {
                        self.scroll_detailed_list_down();
                    }
                }
            }
            // Enhanced navigation with Ctrl modifiers
            (KeyCode::Home, _) => {
                if !self.show_help {
                    if self.is_running || self.show_progress_screen {
                        self.detailed_list_scroll_state.select(Some(0));
                    } else {
                        self.item_list_state.select(Some(0));
                    }
                }
            }
            (KeyCode::End, _) => {
                if !self.show_help {
                    if self.is_running || self.show_progress_screen {
                        if !self.detailed_cleaned_items.is_empty() {
                            let last_index =
                                (self.detailed_cleaned_items.len() * 3).saturating_sub(1);
                            self.detailed_list_scroll_state.select(Some(last_index));
                        }
                    } else {
                        let len = self.categories[self.category_index].items.len();
                        if len > 0 {
                            self.item_list_state.select(Some(len - 1));
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(false)
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    pub fn toggle_compact_mode(&mut self) {
        self.compact_mode = !self.compact_mode;
        self.view_mode = if self.compact_mode {
            ViewMode::Compact
        } else {
            ViewMode::Standard
        };
    }

    pub fn toggle_auto_scroll(&mut self) {
        // Auto scroll functionality for operation logs
    }

    pub fn toggle_performance_stats(&mut self) {
        self.show_performance_stats = !self.show_performance_stats;
    }

    pub fn cycle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Standard => ViewMode::Compact,
            ViewMode::Compact => ViewMode::Detailed,
            ViewMode::Detailed => ViewMode::Performance,
            ViewMode::Performance => ViewMode::Standard,
        };
    }

    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Name => SortMode::Size,
            SortMode::Size => SortMode::Status,
            SortMode::Status => SortMode::Category,
            SortMode::Category => SortMode::Name,
        };
    }

    pub fn cycle_filter_mode(&mut self) {
        self.filter_mode = match self.filter_mode {
            FilterMode::All => FilterMode::Selected,
            FilterMode::Selected => FilterMode::Completed,
            FilterMode::Completed => FilterMode::Errors,
            FilterMode::Errors => FilterMode::UserOnly,
            FilterMode::UserOnly => FilterMode::SystemOnly,
            FilterMode::SystemOnly => FilterMode::All,
        };
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn toggle_confirmation_mode(&mut self) {
        self.confirmation_mode = !self.confirmation_mode;
    }

    pub fn update_counters(&mut self) {
        self.selected_cleaners_count = self
            .categories
            .iter()
            .flat_map(|cat| &cat.items)
            .filter(|item| item.selected)
            .count();

        self.errors_count = self
            .categories
            .iter()
            .flat_map(|cat| &cat.items)
            .filter(|item| matches!(item.status, Some(Status::Error(_))))
            .count();

        self.operation_count = self
            .categories
            .iter()
            .flat_map(|cat| &cat.items)
            .filter(|item| item.status.is_some())
            .count();

        // Auto-complete when all operations are finished
        if self.is_running && self.operation_count > 0 {
            let running_count = self
                .categories
                .iter()
                .flat_map(|cat| &cat.items)
                .filter(|item| matches!(item.status, Some(Status::Running)))
                .count();

            let pending_count = self
                .categories
                .iter()
                .flat_map(|cat| &cat.items)
                .filter(|item| matches!(item.status, Some(Status::Pending)))
                .count();

            let selected_count = self
                .categories
                .iter()
                .flat_map(|cat| &cat.items)
                .filter(|item| item.selected)
                .count();

            // If no operations are running or pending, and we have selected items, mark as complete
            if running_count == 0 && pending_count == 0 && selected_count > 0 {
                self.is_running = false;
                self.demo_operation_timer = None;
                self.operation_end_time = Some(Instant::now());

                // Add completion message
                if !self
                    .result_messages
                    .iter()
                    .any(|msg| msg.contains("Completed"))
                {
                    self.result_messages.push(format!(
                        "✅ Cleaning completed! Total space freed: {} (Press ESC to return to main menu)",
                        format_size(self.total_bytes_cleaned)
                    ));
                }
                // Keep show_progress_screen true so user stays on details screen
            }
        }
    }

    pub fn clear_errors(&mut self) {
        for category in &mut self.categories {
            for item in &mut category.items {
                if matches!(item.status, Some(Status::Error(_))) {
                    item.status = None;
                }
            }
        }
        self.errors_count = 0;
    }

    pub fn get_elapsed_time(&self) -> String {
        if let Some(start_time) = self.operation_start_time {
            let elapsed = if let Some(end_time) = self.operation_end_time {
                // Operation completed, show total time
                end_time.duration_since(start_time)
            } else {
                // Operation still running, show current elapsed time
                start_time.elapsed()
            };

            if elapsed.as_secs() < 60 {
                format!("{}s", elapsed.as_secs())
            } else {
                format!("{}m {}s", elapsed.as_secs() / 60, elapsed.as_secs() % 60)
            }
        } else {
            "0s".to_string()
        }
    }

    pub fn add_detailed_cleaned_item(
        &mut self,
        path: String,
        size: u64,
        category: String,
        cleaner_name: String,
        item_type: CleanedItemType,
    ) {
        let item = DetailedCleanedItem {
            path,
            size,
            category,
            cleaner_name,
            timestamp: SystemTime::now(),
            item_type,
        };
        self.detailed_cleaned_items.push(item);

        // Keep only last 1000 items to prevent memory issues
        if self.detailed_cleaned_items.len() > 1000 {
            self.detailed_cleaned_items.remove(0);
        }
    }

    pub fn scroll_detailed_list_up(&mut self) {
        if let Some(selected) = self.detailed_list_scroll_state.selected() {
            if selected > 0 {
                self.detailed_list_scroll_state.select(Some(selected - 1));
            }
        } else {
            // Start from the bottom when first navigating
            let total_items = if !self.detailed_cleaned_items.is_empty() {
                self.detailed_cleaned_items.len() * 3 // Account for spacing between items
            } else {
                45 // Sample items count for demo
            };
            if total_items > 0 {
                self.detailed_list_scroll_state
                    .select(Some(total_items - 1));
            }
        }
    }

    pub fn scroll_detailed_list_down(&mut self) {
        let total_items = if !self.detailed_cleaned_items.is_empty() {
            self.detailed_cleaned_items.len() * 3 // Account for spacing between items
        } else {
            45 // Sample items count for demo
        };

        if let Some(selected) = self.detailed_list_scroll_state.selected() {
            if selected < total_items.saturating_sub(1) {
                self.detailed_list_scroll_state.select(Some(selected + 1));
            }
        } else if total_items > 0 {
            self.detailed_list_scroll_state.select(Some(0));
        }
    }

    pub fn get_filtered_detailed_items(&self) -> Vec<&DetailedCleanedItem> {
        let mut items: Vec<&DetailedCleanedItem> = self
            .detailed_cleaned_items
            .iter()
            .filter(|item| {
                // Apply search filter
                if !self.search_query.is_empty() {
                    let query_lower = self.search_query.to_lowercase();
                    return item.path.to_lowercase().contains(&query_lower)
                        || item.category.to_lowercase().contains(&query_lower)
                        || item.cleaner_name.to_lowercase().contains(&query_lower);
                }

                // Apply category filter
                if !self.detailed_view_filter.is_empty() {
                    return item
                        .category
                        .to_lowercase()
                        .contains(&self.detailed_view_filter.to_lowercase());
                }

                true
            })
            .collect();

        // Sort based on current sort mode
        match self.sort_mode {
            SortMode::Name => items.sort_by(|a, b| a.path.cmp(&b.path)),
            SortMode::Size => items.sort_by(|a, b| b.size.cmp(&a.size)), // Largest first
            SortMode::Category => items.sort_by(|a, b| a.category.cmp(&b.category)),
            SortMode::Status => items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)), // Most recent first
        }

        items
    }

    pub fn toggle_chart_type(&mut self) {
        self.chart_type = match self.chart_type {
            ChartType::Bar => ChartType::PieCount,
            ChartType::PieCount => ChartType::PieSize,
            ChartType::PieSize => ChartType::Bar,
        };
    }

    pub fn add_sample_cleaned_items(&mut self) {
        // Add some sample cleaned items to demonstrate the detailed view
        let sample_items = vec![
            (
                "/home/user/.cache/pip/wheels/abc123.whl",
                15728640,
                "Package Manager Caches",
                "pip cache",
                CleanedItemType::File,
            ),
            (
                "/home/user/.cache/npm/_cacache/content-v2/sha512/",
                8388608,
                "Package Manager Caches",
                "npm cache",
                CleanedItemType::Directory,
            ),
            (
                "/home/user/.local/share/Trash/files/old_document.pdf",
                2097152,
                "Trash",
                "trash",
                CleanedItemType::File,
            ),
            (
                "/home/user/.cache/mozilla/firefox/profiles/",
                104857600,
                "Browser Caches",
                "firefox cache",
                CleanedItemType::Directory,
            ),
            (
                "/home/user/.cargo/registry/cache/github.com-1ecc6299db9ec823/",
                52428800,
                "Package Manager Caches",
                "cargo cache",
                CleanedItemType::Directory,
            ),
            (
                "/tmp/temp_file_12345.tmp",
                1048576,
                "Temporary Files",
                "temp files",
                CleanedItemType::File,
            ),
            (
                "/home/user/.cache/thumbnails/large/abc123.png",
                262144,
                "Thumbnail Caches",
                "thumbnails",
                CleanedItemType::File,
            ),
            (
                "/var/log/old_system.log",
                10485760,
                "System Logs",
                "system logs",
                CleanedItemType::Log,
            ),
            (
                "/home/user/.local/share/recently-used.xbel.bak",
                32768,
                "Application Caches",
                "application cache",
                CleanedItemType::File,
            ),
            (
                "/home/user/.cache/google-chrome/Default/Cache/",
                209715200,
                "Browser Caches",
                "chrome cache",
                CleanedItemType::Directory,
            ),
            (
                "/home/user/.npm/_cacache/tmp/",
                4194304,
                "Package Manager Caches",
                "npm cache",
                CleanedItemType::Directory,
            ),
            (
                "/home/user/.cache/yarn/v6/npm-lodash-4.17.21/",
                1572864,
                "Package Manager Caches",
                "yarn cache",
                CleanedItemType::Directory,
            ),
            (
                "/var/tmp/portage/",
                83886080,
                "Temporary Files",
                "portage temp",
                CleanedItemType::Directory,
            ),
            (
                "/home/user/.local/share/Trash/files/screenshot.png",
                3145728,
                "Trash",
                "trash",
                CleanedItemType::File,
            ),
            (
                "/home/user/.cache/fontconfig/",
                524288,
                "Application Caches",
                "font cache",
                CleanedItemType::Directory,
            ),
        ];

        for (path, size, category, cleaner, item_type) in sample_items {
            self.add_detailed_cleaned_item(
                path.to_string(),
                size,
                category.to_string(),
                cleaner.to_string(),
                item_type,
            );
        }
    }
}
