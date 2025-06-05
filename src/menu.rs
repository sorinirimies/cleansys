use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::io::{self, Write};

use crate::cleaners::{system_cleaners, user_cleaners};
use crate::utils::{check_root, confirm, print_error, print_header, print_success, print_warning};

pub struct MenuItem {
    id: usize,
    name: String,
    description: String,
    requires_root: bool,
    function: fn(bool) -> Result<u64>,
}

pub struct Menu {
    items: Vec<MenuItem>,
    is_root: bool,
}

impl Menu {
    pub fn new() -> Self {
        let is_root = check_root();
        let mut items = Vec::new();
        let mut id = 1;

        // Add user cleaner items
        for cleaner in user_cleaners::get_cleaners() {
            items.push(MenuItem {
                id,
                name: cleaner.name.to_string(),
                description: cleaner.description.to_string(),
                requires_root: false,
                function: cleaner.function,
            });
            id += 1;
        }

        // Add system cleaner items
        for cleaner in system_cleaners::get_cleaners() {
            items.push(MenuItem {
                id,
                name: cleaner.name.to_string(),
                description: cleaner.description.to_string(),
                requires_root: true,
                function: cleaner.function,
            });
            id += 1;
        }

        Menu { items, is_root }
    }

    pub fn display(&self) -> Result<()> {
        print_header("CLEAN MY SYSTEM");

        println!("Select cleaning options (comma-separated numbers, e.g. 1,3,5):");
        println!(
            "0: [{}] Select all{}",
            "ALL".green(),
            if !self.is_root {
                " (user cleaners only)"
            } else {
                ""
            }
        );

        // Group items by user/system
        println!("\n{}", "USER CLEANERS:".blue().bold());
        for item in &self.items {
            if !item.requires_root {
                println!("{}: [{}] {}", item.id, item.name.green(), item.description);
            }
        }

        println!("\n{}", "SYSTEM CLEANERS:".red().bold());
        for item in &self.items {
            if item.requires_root {
                let status = if self.is_root {
                    item.name.green()
                } else {
                    format!("{} (requires root)", item.name).red()
                };
                println!("{}: [{}] {}", item.id, status, item.description);
            }
        }

        Ok(())
    }

    pub fn run_interactive(&self) -> Result<()> {
        self.display()?;

        print!("\nEnter your choices (or 'q' to quit): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.eq_ignore_ascii_case("q") {
            return Ok(());
        }

        let selections = self.parse_selections(input);
        self.run_selected_cleaners(selections)?;

        Ok(())
    }

    fn parse_selections(&self, input: &str) -> Vec<usize> {
        if input.trim() == "0" {
            // Return all ids that don't require root or all ids if we are root
            return self
                .items
                .iter()
                .filter(|item| !item.requires_root || self.is_root)
                .map(|item| item.id)
                .collect();
        }

        input
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .filter(|&id| id > 0 && id <= self.items.len())
            .collect()
    }

    fn run_selected_cleaners(&self, selections: Vec<usize>) -> Result<()> {
        if selections.is_empty() {
            print_warning("No valid selections made. Exiting.");
            return Ok(());
        }

        let mut total_saved: u64 = 0;
        let mut skipped_items = Vec::new();

        // Create a map of id to items for easier lookup
        let id_map: HashMap<usize, &MenuItem> =
            self.items.iter().map(|item| (item.id, item)).collect();

        for id in selections {
            if let Some(item) = id_map.get(&id) {
                // Skip system cleaners if not root
                if item.requires_root && !self.is_root {
                    skipped_items.push(item.name.clone());
                    continue;
                }

                print_header(&format!("RUNNING: {}", item.name.to_uppercase()));

                if confirm(&format!("Run '{}'?", item.name), true)? {
                    match (item.function)(false) {
                        Ok(bytes) => {
                            total_saved += bytes;
                            print_success(&format!(
                                "{} completed: freed {}",
                                item.name,
                                crate::utils::format_size(bytes)
                            ));
                        }
                        Err(err) => {
                            print_error(&format!("Error in {}: {}", item.name, err));
                        }
                    }
                }
            }
        }

        if !skipped_items.is_empty() {
            print_warning(&format!(
                "The following cleaners were skipped because they require root privileges: {}",
                skipped_items.join(", ")
            ));
        }

        print_header("CLEANING COMPLETE");
        print_success(&format!(
            "Total space freed: {}",
            crate::utils::format_size(total_saved)
        ));

        Ok(())
    }
}
