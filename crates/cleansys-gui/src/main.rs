use cleansys_gui::{update, view, CleanSysGui, Message};

fn boot() -> (CleanSysGui, iced::Task<Message>) {
    (CleanSysGui::new(), iced::Task::none())
}

fn main() -> iced::Result {
    env_logger::init();

    iced::application(boot, update, view)
        .title("CleanSys")
        .window(iced::window::Settings {
            size: iced::Size::new(900.0, 700.0),
            ..Default::default()
        })
        .run()
}
