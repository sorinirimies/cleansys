//! Theme selector widget for the CleanSys GUI.
//!
//! Provides a `pick_list()` drop-down that lets the user switch between
//! all themes defined in `cleansys_core` at runtime.

use iced::widget::pick_list;
use iced::{Element, Length};

use crate::message::Message;

/// A wrapper around a theme index that implements `Display` for the pick-list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeChoice {
    pub index: usize,
    pub name: &'static str,
}

impl std::fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

/// All available themes as [`ThemeChoice`] values, derived from the canonical
/// core definitions.
pub fn all_themes() -> Vec<ThemeChoice> {
    cleansys_core::THEME_NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| ThemeChoice { index: i, name })
        .collect()
}

/// [`all_themes`] preceded by a "System" entry (index [`cleansys_core::THEME_COUNT`]).
pub fn theme_choices() -> Vec<ThemeChoice> {
    let mut v = vec![ThemeChoice {
        index: cleansys_core::THEME_COUNT,
        name: cleansys_core::SYSTEM_THEME_NAME,
    }];
    v.extend(all_themes());
    v
}

/// Create a theme selector `pick_list()` widget.
///
/// The widget displays the name of the current theme and, when opened, lists
/// every theme returned by [`all_themes`]. Selecting a new entry emits
/// [`Message::ThemeChanged`] with the chosen theme index.
pub fn theme_selector(
    current_theme_index: usize,
    follow_system: bool,
) -> Element<'static, Message> {
    let choices = theme_choices();
    let selected = if follow_system {
        choices.first().cloned()
    } else {
        // +1: the "System" entry is listed first.
        choices.get(current_theme_index + 1).cloned()
    };

    pick_list(choices, selected, |choice| {
        Message::ThemeChanged(choice.index)
    })
    .placeholder("Select theme")
    .text_size(13.0)
    .width(Length::Fixed(200.0))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_themes_matches_core_count() {
        assert_eq!(all_themes().len(), cleansys_core::THEME_COUNT);
    }

    #[test]
    fn theme_choices_lead_with_system() {
        let c = theme_choices();
        assert_eq!(c.len(), cleansys_core::THEME_COUNT + 1);
        assert_eq!(c[0].index, cleansys_core::THEME_COUNT);
        assert_eq!(c[0].name, "System");
        assert_eq!(c[1].index, 0);
    }

    #[test]
    fn all_themes_indices_are_sequential() {
        for (i, choice) in all_themes().iter().enumerate() {
            assert_eq!(choice.index, i);
        }
    }

    #[test]
    fn theme_choice_display_matches_name() {
        let choice = ThemeChoice {
            index: 9,
            name: "Dracula",
        };
        assert_eq!(format!("{choice}"), "Dracula");
    }
}
