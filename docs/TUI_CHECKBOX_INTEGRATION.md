# TUI-Checkbox Integration

This document describes how the `tui-checkbox` library is integrated into CleanSys.

## Overview

CleanSys uses the [`tui-checkbox`](https://crates.io/crates/tui-checkbox) library to provide consistent and customizable checkbox symbols in the terminal user interface. This library, created by the same author as CleanSys, provides both a `Checkbox` widget and predefined symbol constants.

## Why Symbols Instead of the Checkbox Widget?

The `tui-checkbox` library provides **two complementary APIs**:

1. **`Checkbox` Widget**: For rendering standalone checkboxes as independent widgets
2. **Symbol Constants**: For building composite UI elements like lists

### Our Use Case: List-Based UI

CleanSys uses a `List` widget where each `ListItem` combines multiple pieces of information on a single line:
- Checkbox indicator
- Cleaner name
- Root requirement indicator
- Status (Running/Success/Error)
- Freed space amount

```
[X] Package cache cleaner (root) [Success] (Freed: 245 MB)
[ ] Browser cache cleaner [Pending]
[X] System logs (root) [Running...]
```

### Why Symbols Are The Right Choice

According to the `tui-checkbox` examples, the `Checkbox` widget is designed to be rendered in **its own dedicated area**. Each checkbox is a complete widget that includes both the symbol and label, and gets rendered to a `Rect`.

However, in our List-based UI:
- Each `ListItem` is composed of multiple `Span`s (text segments with individual styling)
- We need to mix the checkbox symbol with other text elements in a single line
- The `Checkbox` widget cannot be embedded inside a `Span` - it needs its own render area

**This is exactly what the symbol constants are designed for!** The library provides symbols like `CHECKED_X`, `UNCHECKED_SPACE`, etc., specifically for use cases where you need checkbox symbols as part of a larger composite element.

## Integration Details

### Location

The checkbox symbols are used in `src/ui/render.rs` in the `render_cleaners` function, which displays the list of available cleaners with checkbox selection.

### Implementation

```rust
// Using tui-checkbox library for consistent checkbox symbols across the application
use tui_checkbox::symbols as checkbox_symbols;

// In render_cleaners function:
let checkbox_style = if item.selected {
    Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD)
} else {
    Style::default().fg(Color::White)
};

let checkbox_symbol = if item.selected {
    checkbox_symbols::CHECKED_X
} else {
    checkbox_symbols::UNCHECKED_SPACE
};

parts.push(Span::styled(checkbox_symbol, checkbox_style));
parts.push(Span::raw(" "));
```

### Current Symbols

- **Checked**: `[X]` (ASCII bracket with X) - `checkbox_symbols::CHECKED_X`
- **Unchecked**: `[ ]` (ASCII bracket with space) - `checkbox_symbols::UNCHECKED_SPACE`

## When to Use Each Approach

Based on the `tui-checkbox` examples and API design:

### Use `Checkbox` Widget When:
- Rendering standalone checkboxes in dedicated areas
- Each checkbox is an independent UI element
- You want the widget to handle both symbol and label rendering
- You're using Layout to create separate rectangles for each checkbox

**Example from tui-checkbox:**
```rust
let checkbox = Checkbox::new("Enable notifications", true)
    .checked_symbol(symbols::CHECKED_X)
    .unchecked_symbol(symbols::UNCHECKED_SPACE);
frame.render_widget(checkbox, area);
```

### Use Symbol Constants When:
- Building composite UI elements (like our List items)
- Mixing checkbox symbols with other text in a single line
- Constructing `Span`s or `Line`s for `List`, `Table`, or `Paragraph` widgets
- Need fine-grained control over layout and styling

**Our approach (correct for our use case):**
```rust
// Building a ListItem with multiple Spans
parts.push(Span::styled(checkbox_symbols::CHECKED_X, checkbox_style));
parts.push(Span::raw(" "));
parts.push(Span::styled(&item.name, name_style));
parts.push(Span::styled(" (root)", root_style));
```

### Available Alternatives

The `tui-checkbox` library provides several symbol sets that can be easily swapped:

1. **ASCII Brackets** (current):
   - `CHECKED_X` → `[X]`
   - `UNCHECKED_SPACE` → `[ ]`

2. **Unicode Box Symbols**:
   - `CHECKED` → `☑`
   - `UNCHECKED` → `☐`

3. **ASCII Plus/Minus**:
   - `CHECKED_PLUS` → `[+]`
   - `UNCHECKED_MINUS` → `[-]`

4. **ASCII Asterisk**:
   - `CHECKED_ASTERISK` → `[*]`
   - `UNCHECKED_SPACE` → `[ ]`

5. **Parenthesis Style**:
   - `CHECKED_PARENTHESIS_X` → `(X)`
   - `UNCHECKED_PARENTHESIS_O` → `(O)`

## Benefits

1. **Consistency**: Using the `tui-checkbox` library ensures consistent checkbox rendering across the application
2. **Maintainability**: Symbol definitions are centralized in the library
3. **Flexibility**: Easy to switch between different symbol styles for different terminal capabilities
4. **Best Practices**: Follows Ratatui ecosystem conventions

## Comparison: Widget vs Symbols

### Checkbox Widget Example (Standalone)
```rust
use tui_checkbox::{symbols, Checkbox};

// Each checkbox rendered in its own area
let checkbox1 = Checkbox::new("Enable notifications", true)
    .checked_symbol(symbols::CHECKED_X)
    .unchecked_symbol(symbols::UNCHECKED_SPACE);

let checkbox2 = Checkbox::new("Dark mode", false)
    .checked_symbol(symbols::CHECKED_X)
    .unchecked_symbol(symbols::UNCHECKED_SPACE);

// Each needs its own Rect
frame.render_widget(checkbox1, areas[0]);
frame.render_widget(checkbox2, areas[1]);
```

### Symbol Constants Example (Composite - Our Approach)
```rust
use tui_checkbox::symbols as checkbox_symbols;

// Building list items with multiple spans
let items: Vec<ListItem> = data.iter().map(|item| {
    let mut parts = vec![];
    
    // Checkbox symbol as part of a larger line
    parts.push(Span::styled(
        if item.selected { checkbox_symbols::CHECKED_X } 
        else { checkbox_symbols::UNCHECKED_SPACE },
        checkbox_style
    ));
    
    // Additional information on the same line
    parts.push(Span::raw(" "));
    parts.push(Span::styled(&item.name, name_style));
    parts.push(Span::styled(" (status)", status_style));
    
    ListItem::new(Line::from(parts))
}).collect();

// Render as a single List widget
frame.render_widget(List::new(items), area);
```

## Customization

### Changing Symbol Style in Our List

To change the checkbox style in our list implementation, simply update the symbol constants in `render_cleaners`:

```rust
// Example: Switch to Unicode style for better aesthetics
let checkbox_symbol = if item.selected {
    checkbox_symbols::CHECKED  // Unicode ☑
} else {
    checkbox_symbols::UNCHECKED  // Unicode ☐
};
parts.push(Span::styled(checkbox_symbol, checkbox_style));

// Or use Plus/Minus style
let checkbox_symbol = if item.selected {
    checkbox_symbols::CHECKED_PLUS  // [+]
} else {
    checkbox_symbols::UNCHECKED_MINUS  // [-]
};
parts.push(Span::styled(checkbox_symbol, checkbox_style));
```

### If We Ever Need Standalone Checkboxes

If in the future we want to add a settings screen with standalone checkboxes, we should use the `Checkbox` widget:

```rust
use tui_checkbox::{symbols, Checkbox};

// Proper use of the Checkbox widget
let checkbox = Checkbox::new("Enable verbose logging", config.verbose)
    .checked_symbol(symbols::CHECKED_X)
    .unchecked_symbol(symbols::UNCHECKED_SPACE)
    .checkbox_style(Style::default().fg(Color::Green))
    .label_style(Style::default().fg(Color::White));

frame.render_widget(checkbox, checkbox_area);
```

## Conclusion

Our use of `tui_checkbox::symbols` is the **correct and intended approach** for our List-based UI. The library provides these symbol constants specifically for use cases like ours where checkboxes need to be part of composite UI elements.

The `Checkbox` widget and symbol constants are complementary features:
- **Widget**: Complete checkbox with label, for standalone use
- **Symbols**: Individual checkbox characters, for composite use

We're using the right tool for our specific job!

## Testing

The checkbox symbols work correctly across various terminal emulators:
- GNOME Terminal
- Alacritty
- Kitty
- iTerm2
- Windows Terminal
- tmux/screen

## Related

- TUI-Checkbox Repository: https://github.com/sorinirimies/tui-checkbox
- TUI-Checkbox Examples: See the `checkbox.rs` example in the repository
- Ratatui: https://github.com/ratatui-org/ratatui