use super::icons::{self, Icon};
use crate::app::{App, Message};
use iced::{
    Color, Element, Fill,
    widget::{button, container, pick_list, row, space, text, text_input},
};

pub fn view(app: &App) -> Element<'_, Message> {
    let selector = pick_list(
        app.snapshot.tasks.clone(),
        app.selected.clone(),
        Message::SelectTask,
    )
    .placeholder("Select a task")
    .width(260);
    let mut add = button("Add task…");
    if app.ready() && !app.new_task_name.trim().is_empty() {
        add = add.on_press(Message::AddTask);
    }
    let running = app.snapshot.active.is_some();
    let (icon, color) = if running {
        (Icon::Pause(Color::BLACK), Color::from_rgb8(67, 181, 111))
    } else {
        (Icon::Play(Color::BLACK), Color::from_rgb8(245, 200, 66))
    };
    let mut toggle = button(
        container(icons::view(icon, 26.0))
            .center_x(Fill)
            .center_y(Fill),
    )
    .width(44)
    .height(44)
    .style(move |theme, status| {
        let opacity = match status {
            button::Status::Active => 1.0,
            button::Status::Hovered | button::Status::Pressed => 0.85,
            button::Status::Disabled => 0.5,
        };
        button::Style {
            background: Some(color.scale_alpha(opacity).into()),
            text_color: Color::BLACK,
            ..button::primary(theme, status)
        }
    });
    if app.ready() && app.selected.is_some() {
        toggle = toggle.on_press(Message::Toggle);
    }
    row![
        text_input("New task name", &app.new_task_name)
            .on_input(Message::TaskName)
            .on_submit(Message::AddTask)
            .width(220),
        add,
        space().width(Fill),
        text("Current task"),
        selector,
        toggle
    ]
    .spacing(8)
    .align_y(iced::Center)
    .into()
}
