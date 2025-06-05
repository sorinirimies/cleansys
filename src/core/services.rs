use anyhow::Result;
use log::{debug, info};

use crate::core::models::{CleanerItem, CleanerCategory, CleanerStatus, CleanerMessage};

/// Executes a cleaning operation and returns the bytes cleaned
pub fn execute_cleaner(item: &mut CleanerItem, skip_confirmation: bool) -> Result<CleanerMessage> {
    info!("Executing cleaner: {}", item.name);
    item.status = CleanerStatus::Running;
    
    match (item.function)(skip_confirmation) {
        Ok(bytes) => {
            item.bytes_cleaned = bytes;
            item.status = CleanerStatus::Success;
            debug!("Cleaner {} completed successfully, freed {} bytes", item.name, bytes);
            
            Ok(CleanerMessage {
                cleaner_name: item.name.clone(),
                message: format!("Completed successfully, freed {} bytes", bytes),
                is_error: false,
                bytes_cleaned: Some(bytes),
            })
        },
        Err(err) => {
            item.status = CleanerStatus::Error;
            debug!("Cleaner {} failed: {}", item.name, err);
            
            Ok(CleanerMessage {
                cleaner_name: item.name.clone(),
                message: format!("Error: {}", err),
                is_error: true,
                bytes_cleaned: None,
            })
        }
    }
}

/// Executes all selected cleaners in a category
pub fn execute_category(category: &mut CleanerCategory, skip_confirmation: bool) -> Result<Vec<CleanerMessage>> {
    let mut messages = Vec::new();
    let mut total_cleaned: u64 = 0;
    
    for item in &mut category.items {
        if item.selected {
            match execute_cleaner(item, skip_confirmation) {
                Ok(message) => {
                    if let Some(bytes) = message.bytes_cleaned {
                        total_cleaned += bytes;
                    }
                    messages.push(message);
                },
                Err(err) => {
                    messages.push(CleanerMessage {
                        cleaner_name: item.name.clone(),
                        message: format!("Failed to execute: {}", err),
                        is_error: true,
                        bytes_cleaned: None,
                    });
                }
            }
        }
    }
    
    info!("Category {} completed, total cleaned: {} bytes", category.name, total_cleaned);
    
    Ok(messages)
}

/// Selects all cleaners in a category
pub fn select_all_in_category(category: &mut CleanerCategory) {
    for item in &mut category.items {
        item.selected = true;
    }
}

/// Deselects all cleaners in a category
pub fn deselect_all_in_category(category: &mut CleanerCategory) {
    for item in &mut category.items {
        item.selected = false;
    }
}

/// Calculates the total bytes cleaned in a category
pub fn calculate_category_total(category: &CleanerCategory) -> u64 {
    category.items.iter()
        .filter(|item| item.status == CleanerStatus::Success)
        .map(|item| item.bytes_cleaned)
        .sum()
}

/// Calculates the total bytes cleaned across all categories
pub fn calculate_total_cleaned(categories: &[CleanerCategory]) -> u64 {
    categories.iter()
        .map(calculate_category_total)
        .sum()
}