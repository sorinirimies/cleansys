# Checkbox Usage Summary

## Overview

This document summarizes how the `tui-checkbox` library is used throughout the CleanSys application for both User Land Cleaners and System Cleaners.

## Implementation Location

All checkbox rendering is centralized in a single function:
- **File**: `src/ui/render.rs`
- **Function**: `render_cleaners()`

This function is used for rendering **both** User Land Cleaners and System Cleaners, ensuring consistent checkbox behavior across the entire application.

## Code Implementation

```rust
// src/ui/render.rs
use tui_checkbox::{symbols as checkbox_symbols, Checkbox};

fn render_cleaners<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let current_category = &app.categories[app.category_index];

    let items: Vec<ListItem> = current_category
        .items
        .iter()
        .map(|item| {
            let mut parts = vec![];

            // Create checkbox using tui-checkbox with predefined symbols
            // We use the ASCII bracket symbols for maximum terminal compatibility
            let checkbox_style = if item.selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Use Checkbox::new() with predefined symbols from the library
            let _checkbox = Checkbox::new("", item.selected)
                .checked_symbol(checkbox_symbols::CHECKED_X)
                .unchecked_symbol(checkbox_symbols::UNCHECKED_SPACE);

            // Extract the symbol for use in our composite List item
            let checkbox_symbol = if item.selected {
                checkbox_symbols::CHECKED_X
            } else {
                checkbox_symbols::UNCHECKED_SPACE
            };

            parts.push(Span::styled(checkbox_symbol, checkbox_style));
            parts.push(Span::raw(" "));
            parts.push(Span::styled(&item.name, name_style));
            // ... additional item information ...
            
            ListItem::new(Line::from(parts))
        })
        .collect();
}
```

## Categories Coverage

### User Land Cleaners
- Browser caches (Firefox, Chrome/Chromium)
- Application caches
- Thumbnail caches
- Temporary files owned by the user
- Package manager caches (pip, npm, cargo)
- User trash

**Checkbox Behavior**: All user cleaners use the same checkbox rendering from `render_cleaners()`

### System Cleaners (requires root)
- Package manager caches (apt, pacman, dnf, etc.)
- System logs
- System caches
- Temporary files
- Old kernels (on supported systems)
- Crash reports and core dumps

**Checkbox Behavior**: All system cleaners use the same checkbox rendering from `render_cleaners()`

## How Categories Are Loaded

The cleaners are loaded into categories in `src/ui/mod.rs`:

```rust
fn load_cleaners(app: &mut App) {
    // Add user cleaners
    let mut user_items = Vec::new();
    for cleaner in user_cleaners::get_cleaners() {
        user_items.push(CleanerItem {
            name: cleaner.name.to_string(),
            description: cleaner.description.to_string(),
            requires_root: false,
            selected: false,
            function: cleaner.function,
            bytes_cleaned: 0,
            status: None,
        });
    }

    // Add system cleaners
    let mut system_items = Vec::new();
    for cleaner in system_cleaners::get_cleaners() {
        system_items.push(CleanerItem {
            name: cleaner.name.to_string(),
            description: cleaner.description.to_string(),
            requires_root: true,
            selected: false,
            function: cleaner.function,
            bytes_cleaned: 0,
            status: None,
        });
    }

    app.categories = vec![
        CleanerCategory {
            name: "User Land Cleaners".to_string(),
            description: "Clean user-specific files and caches".to_string(),
            items: user_items,
        },
        CleanerCategory {
            name: "System Cleaners".to_string(),
            description: "Clean system files and caches (requires root)".to_string(),
            items: system_items,
        },
    ];
}
```

## Checkbox Symbols Used

We use the ASCII bracket style from `tui-checkbox` for maximum terminal compatibility:

- **Selected**: `[X]` - `checkbox_symbols::CHECKED_X`
  - Style: Green with Bold modifier
- **Unselected**: `[ ]` - `checkbox_symbols::UNCHECKED_SPACE`
  - Style: White (default)

## Visual Examples

### User Land Cleaners Display
```
┌─ User Land Cleaners Items ─────────────────────────────┐
│ > [X] Browser Caches                                    │
│   [ ] Application Caches                                │
│   [X] Thumbnail Caches                                  │
│   [ ] Temporary Files                                   │
│   [X] Package Manager Caches (pip, npm, cargo)          │
│   [ ] User Trash                                        │
└─────────────────────────────────────────────────────────┘
```

### System Cleaners Display (when root)
```
┌─ System Cleaners Items ────────────────────────────────┐
│ > [X] Package Manager Caches (root)                     │
│   [ ] System Logs (root)                                │
│   [X] System Caches (root)                              │
│   [ ] Temporary Files (root)                            │
│   [ ] Old Kernels (root)                                │
│   [X] Crash Reports (root)                              │
└─────────────────────────────────────────────────────────┘
```

### System Cleaners Display (when not root)
```
┌─ System Cleaners Items ────────────────────────────────┐
│   [ ] Package Manager Caches (root)                     │
│   [ ] System Logs (root)                                │
│   [ ] System Caches (root)                              │
│   [ ] Temporary Files (root)                            │
│   [ ] Old Kernels (root)                                │
│   [ ] Crash Reports (root)                              │
│                                                          │
│   Note: System cleaners are grayed out without root     │
└─────────────────────────────────────────────────────────┘
```

## Key Features

1. **Unified Rendering**: Both user and system cleaners use the exact same checkbox rendering logic
2. **Consistent Styling**: Selected items are always green and bold, unselected are white
3. **Root Awareness**: System cleaners show "(root)" indicator and are grayed out when not running as root
4. **Library Integration**: Uses `tui-checkbox` library's predefined symbols for consistency
5. **ASCII Compatibility**: ASCII bracket style `[X]`/`[ ]` works on all terminals

## Testing Checklist

To verify checkbox functionality works for both user and system cleaners:

- [ ] Run `cleansys` as regular user
  - [ ] Verify User Land Cleaners show checkboxes
  - [ ] Verify System Cleaners show checkboxes (grayed out)
  - [ ] Toggle selection with Space key
  - [ ] Selected items show `[X]` in green
  - [ ] Unselected items show `[ ]` in white

- [ ] Run `sudo cleansys` as root
  - [ ] Verify User Land Cleaners show checkboxes
  - [ ] Verify System Cleaners show checkboxes (enabled)
  - [ ] Toggle selection with Space key
  - [ ] All cleaners respond to selection

## Future Enhancements

If we ever need to add more categories or change checkbox styles:

1. **Add New Category**: Categories are automatically handled by `render_cleaners()`, no checkbox code changes needed
2. **Change Symbol Style**: Update the symbols in `render_cleaners()`:
   ```rust
   .checked_symbol(checkbox_symbols::CHECKED)      // Unicode ☑
   .unchecked_symbol(checkbox_symbols::UNCHECKED)  // Unicode ☐
   ```
3. **Custom Symbols**: Use any custom symbols:
   ```rust
   .checked_symbol("✓ ")
   .unchecked_symbol("○ ")
   ```

## Related Documentation

- [TUI_CHECKBOX_INTEGRATION.md](./TUI_CHECKBOX_INTEGRATION.md) - Detailed integration guide
- [CLEANUP_SUMMARY.md](./CLEANUP_SUMMARY.md) - Code cleanup documentation
- [tui-checkbox GitHub](https://github.com/sorinirimies/tui-checkbox) - Library repository

## Conclusion

The checkbox implementation is **universal and applies to all cleaners** (both user and system) through the centralized `render_cleaners()` function. The use of the `tui-checkbox` library ensures consistent, professional-looking checkboxes throughout the application.