use cleansys_gui::{update, view, CleanSysGui, Message};

fn boot() -> (CleanSysGui, iced::Task<Message>) {
    // Measure what every cleaner can free as soon as the window opens.
    let mut tasks = vec![iced::Task::done(Message::ScanAll)];
    // `CLEANSYS_GUI_OPEN=schedule` opens the schedule dialog at start-up
    // (used to produce documentation screenshots).
    if std::env::var("CLEANSYS_GUI_OPEN").as_deref() == Ok("schedule") {
        tasks.push(iced::Task::done(Message::OpenSchedule));
    }
    (CleanSysGui::new(), iced::Task::batch(tasks))
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
            // `CLEANSYS_GUI_TOP=1` keeps the window above others (documentation screenshots).
            level: if std::env::var_os("CLEANSYS_GUI_TOP").is_some() {
                iced::window::Level::AlwaysOnTop
            } else {
                iced::window::Level::Normal
            },
            min_size: Some(iced::Size::new(420.0, 520.0)),
            ..Default::default()
        })
        .run()
}
