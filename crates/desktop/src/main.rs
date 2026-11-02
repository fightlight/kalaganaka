#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod platform;
mod settings;
mod storage;
mod ui;

fn main() -> iced::Result {
    iced::daemon(app::App::new, app::App::update, app::App::view)
        .title(app::App::title)
        .subscription(app::App::subscription)
        .theme(app::App::theme)
        .run()
}
