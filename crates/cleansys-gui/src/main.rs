use cleansys_gui::{update, view, CleanSysGui, Message};

fn boot() -> (CleanSysGui, iced::Task<Message>) {
    (CleanSysGui::new(), iced::Task::none())
}

fn main() -> iced::Result {
    env_logger::init();

    iced::application(boot, update, view)
        .title("CleanSys")
        .settings(iced::Settings {
            fonts: vec![iced_fonts::BOOTSTRAP_FONT_BYTES.into()],
            ..Default::default()
        })
        .window(iced::window::Settings {
            size: iced::Size::new(1000.0, 760.0),
            min_size: Some(iced::Size::new(720.0, 480.0)),
            ..Default::default()
        })
        .run()
}
