use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::io::Write;
use std::process::{Command, Stdio};

/// Password prompt component for sudo authentication
pub struct PasswordPrompt {
    /// The password input (stored temporarily during entry)
    password_input: String,
    /// Error message to display if authentication fails
    error_message: Option<String>,
    /// Whether the prompt is currently visible
    visible: bool,
    /// Whether authentication was successful
    authenticated: bool,
}

impl Default for PasswordPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordPrompt {
    /// Create a new password prompt
    pub fn new() -> Self {
        Self {
            password_input: String::new(),
            error_message: None,
            visible: false,
            authenticated: false,
        }
    }

    /// Show the password prompt
    pub fn show(&mut self) {
        self.visible = true;
        self.password_input.clear();
        self.error_message = None;
    }

    /// Hide the password prompt
    pub fn hide(&mut self) {
        self.visible = false;
        self.password_input.clear();
        self.error_message = None;
    }

    /// Check if the prompt is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if authentication was successful
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Add a character to the password input
    pub fn add_char(&mut self, c: char) {
        self.password_input.push(c);
    }

    /// Remove the last character from the password input
    pub fn remove_char(&mut self) {
        self.password_input.pop();
    }

    /// Verify the password using sudo
    pub fn verify_password(&mut self) -> Result<bool> {
        // Try to authenticate with sudo using the provided password
        let mut child = Command::new("sudo")
            .arg("-S")
            .arg("-v")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "{}", self.password_input)?;
        }

        let status = child.wait()?;

        if status.success() {
            self.authenticated = true;
            self.visible = false;
            self.password_input.clear();
            self.error_message = None;
            Ok(true)
        } else {
            self.error_message = Some("Incorrect password. Please try again.".to_string());
            self.password_input.clear();
            Ok(false)
        }
    }

    /// Handle the submit action (Enter key)
    pub fn submit(&mut self) -> Result<bool> {
        self.verify_password()
    }

    /// Handle the cancel action (ESC key)
    pub fn cancel(&mut self) {
        self.hide();
    }

    /// Render the password prompt as an overlay
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        if !self.visible {
            return;
        }

        // Create a centered popup
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 12.min(area.height.saturating_sub(4));

        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Create the popup content
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "🔒 System Cleaner Authentication",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw(
                "System cleaners require root privileges to clean system files.",
            )]),
            Line::from(vec![Span::raw("Please enter your password to continue:")]),
            Line::from(vec![Span::raw("")]),
        ];

        // Add password input line with masked characters
        let password_display = "•".repeat(self.password_input.len());
        lines.push(Line::from(vec![
            Span::styled("Password: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                password_display,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("_", Style::default().fg(Color::Yellow)),
        ]));

        lines.push(Line::from(vec![Span::raw("")]));

        // Add error message if present
        if let Some(error) = &self.error_message {
            lines.push(Line::from(vec![Span::styled(
                format!("❌ {}", error),
                Style::default().fg(Color::Red),
            )]));
            lines.push(Line::from(vec![Span::raw("")]));
        }

        // Add instructions
        lines.push(Line::from(vec![Span::styled(
            "Press Enter to authenticate | ESC to cancel",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));

        let popup = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Authentication Required")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: true });

        // Clear the area behind the popup
        let clear_block = Block::default().style(Style::default().bg(Color::Black));
        f.render_widget(clear_block, popup_area);

        // Render the popup
        f.render_widget(popup, popup_area);
    }
}
