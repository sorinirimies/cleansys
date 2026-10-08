use cleansys_gui::{update, view, CleanSysGui, Message};

/// One-time, platform-specific setup that needs the window to exist.
///
/// macOS: add the standard Window menu (Minimize, Zoom, Move & Resize / tiling), which winit's
/// default menu bar lacks — without it the system's tiling shortcuts (⌃⌘←, Window → Move &
/// Resize, …) do nothing for this window. See [`cleansys_gui::macos`].
#[cfg(target_os = "macos")]
fn platform_setup() -> iced::Task<Message> {
    iced::window::oldest()
        .and_then(|id| {
            iced::window::run(id, |_window| {
                cleansys_gui::macos::install_window_menu();
            })
        })
        .discard()
}

#[cfg(not(target_os = "macos"))]
fn platform_setup() -> iced::Task<Message> {
    iced::Task::none()
}

fn boot() -> (CleanSysGui, iced::Task<Message>) {
    // Measure what every cleaner can free as soon as the window opens.
    let mut tasks = vec![iced::Task::done(Message::ScanAll), platform_setup()];
    // `CLEANSYS_GUI_OPEN=schedule` opens the schedule dialog at start-up
    // (used to produce documentation screenshots).
    if std::env::var("CLEANSYS_GUI_OPEN").as_deref() == Ok("schedule") {
        tasks.push(iced::Task::done(Message::OpenSchedule));
    }
    (CleanSysGui::new(), iced::Task::batch(tasks))
}

/// Initial window size: `CLEANSYS_GUI_SIZE=WxH` (handy for testing the responsive layouts),
/// else the size remembered from last time, else the default.
fn initial_size() -> iced::Size {
    let saved = cleansys_core::load_settings()
        .ok()
        .and_then(|s| s.saved_window_size());
    let (w, h) = cleansys_gui::window::initial_size(saved);
    iced::Size::new(w, h)
}

fn main() -> iced::Result {
    env_logger::init();

    iced::application(boot, update, view)
        .title("CleanSys")
        .theme(|state: &CleanSysGui| state.iced_theme())
        .subscription(cleansys_gui::subscription)
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
            min_size: Some(iced::Size::new(
                cleansys_gui::window::MIN_SIZE.0,
                cleansys_gui::window::MIN_SIZE.1,
            )),
            ..Default::default()
        })
        .run()
}
