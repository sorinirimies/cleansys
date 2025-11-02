//! Tests for password prompt component
//!
//! These tests verify the password prompt functionality in src/components/password_prompt.rs

use cleansys::PasswordPrompt;

#[test]
fn test_password_prompt_creation() {
    let prompt = PasswordPrompt::new();
    assert!(!prompt.is_visible());
    assert!(!prompt.is_authenticated());
}

#[test]
fn test_show_hide() {
    let mut prompt = PasswordPrompt::new();
    assert!(!prompt.is_visible());

    prompt.show();
    assert!(prompt.is_visible());

    prompt.hide();
    assert!(!prompt.is_visible());
}

#[test]
fn test_password_input() {
    let mut prompt = PasswordPrompt::new();

    prompt.add_char('a');
    prompt.add_char('b');
    prompt.add_char('c');

    prompt.remove_char();
}

#[test]
fn test_cancel() {
    let mut prompt = PasswordPrompt::new();
    prompt.show();
    prompt.add_char('a');

    prompt.cancel();
    assert!(!prompt.is_visible());
}
