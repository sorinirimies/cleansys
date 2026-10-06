use cleansys_gui::{update, view, CleanSysGui, Message};

fn boot() -> (CleanSysGui, iced::Task<Message>) {
    // Measure what every cleaner can free as soon as the window opens.
    (CleanSysGui::new(), iced::Task::done(Message::ScanAll))
}

/// Initial window size; `CLEANSYS_GUI_SIZE=WxH` overrides (handy for testing
/// the responsive layouts).
fn initial_size() -> iced::Size {
    std::env::var("CLEANSYS_GUI_SIZE")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some(iced::Size::new(w.parse().ok()?, h.parse().ok()?))
        })
        .unwrap_or(iced::Size::new(1180.0, 780.0))
}

fn main() -> iced::Result {
    env_logger::init();

    iced::application(boot, update, view)
        .title("CleanSys")
        .theme(|state: &CleanSysGui| state.iced_theme())
        .settings(iced::Settings {
            fonts: vec![iced_fonts::BOOTSTRAP_FONT_BYTES.into()],
            ..Default::default()
        })
        .window(iced::window::Settings {
            size: initial_size(),
            min_size: Some(iced::Size::new(420.0, 520.0)),
            ..Default::default()
        })
        .run()
}
