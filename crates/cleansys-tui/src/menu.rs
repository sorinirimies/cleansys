use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::io::{self, Write};

use cleansys_core::utils::{print_success, print_warning};
use cleansys_core::{CleanerFn, check_root, confirm, format_size, print_error, print_header};

#[derive(Debug)]
pub struct MenuItem {
    id: usize,
    name: String,
    description: String,
    category: String,
    requires_root: bool,
    function: CleanerFn,
}

#[derive(Debug)]
pub struct Menu {
    items: Vec<MenuItem>,
    is_root: bool,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    /// Every cleaner of this platform (built-in Rust, TOML, detected app caches), numbered
    /// from 1, user cleaners first.
    pub fn new() -> Self {
        Self::from_categories(cleansys_core::load_categories(), check_root())
    }

    fn from_categories(categories: Vec<cleansys_core::CleanerCategory>, is_root: bool) -> Self {
        let mut seen = std::collections::HashSet::new();
        let mut all: Vec<MenuItem> = categories
            .into_iter()
            .flat_map(|c| {
                let category = c.name;
                c.items.into_iter().map(move |i| (category.clone(), i))
            })
            .filter(|(_, i)| seen.insert(i.id.clone()))
            .map(|(category, i)| MenuItem {
                id: 0,
                name: i.name,
                description: i.description,
                category,
                requires_root: i.requires_root,
                function: i.function,
            })
            .collect();
        // User cleaners first, keeping the category order within each side.
        all.sort_by_key(|i| i.requires_root);
        for (n, item) in all.iter_mut().enumerate() {
            item.id = n + 1;
        }
        Menu {
            items: all,
            is_root,
        }
    }

    pub fn display(&self) -> Result<()> {
        print_header("CLEAN MY SYSTEM");

        println!("Select cleaning options (comma-separated numbers, e.g. 1,3,5):");
        println!(
            "0: [{}] Select all{}",
            "ALL".green(),
            if self.is_root {
                ""
            } else {
                " (user cleaners only)"
            }
        );

        // Group items by user/system
        println!("\n{}", "USER CLEANERS:".blue().bold());
        self.print_side(false);
        println!("\n{}", "SYSTEM CLEANERS:".red().bold());
        self.print_side(true);

        Ok(())
    }

    fn print_side(&self, root: bool) {
        let mut last = "";
        for item in self.items.iter().filter(|i| i.requires_root == root) {
            if item.category != last {
                println!("  {}", item.category.bold());
                last = &item.category;
            }
            let name = if root && !self.is_root {
                format!("{} (requires root)", item.name).red()
            } else {
                item.name.green()
            };
            println!("{}: [{}] {}", item.id, name, item.description);
        }
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
                    match (item.function)(cleansys_core::RunOptions::execute_with_confirmation()) {
                        Ok(result) => {
                            total_saved += result.total_bytes;
                            print_success(&format!(
                                "{} completed: freed {} across {} item(s)",
                                item.name,
                                format_size(result.total_bytes),
                                result.item_count()
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
        print_success(&format!("Total space freed: {}", format_size(total_saved)));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu(is_root: bool) -> Menu {
        Menu::from_categories(cleansys_core::load_categories(), is_root)
    }

    #[test]
    fn lists_toml_cleaners_once_user_side_first() {
        let m = menu(false);
        assert!(m.items.len() > 20, "TOML cleaners are listed");
        let ids: Vec<_> = m.items.iter().map(|i| i.id).collect();
        assert_eq!(ids, (1..=m.items.len()).collect::<Vec<_>>());
        let first_root = m.items.iter().position(|i| i.requires_root);
        if let Some(p) = first_root {
            assert!(m.items[p..].iter().all(|i| i.requires_root));
        }
        let mut names: Vec<_> = m.items.iter().map(|i| (&i.category, &i.name)).collect();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "no cleaner twice");
    }

    #[test]
    fn zero_selects_what_this_user_may_run() {
        let user = menu(false);
        let all = user.parse_selections("0");
        assert!(all.iter().all(|id| !user.items[id - 1].requires_root));
        assert_eq!(menu(true).parse_selections("0").len(), user.items.len());
    }

    #[test]
    fn selections_ignore_junk_and_out_of_range() {
        let m = menu(false);
        let n = m.items.len();
        assert_eq!(
            m.parse_selections(&format!("1, x, 0, {}, 2", n + 1)),
            vec![1, 2]
        );
    }
}
