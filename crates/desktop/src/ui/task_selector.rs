use crate::app::{App, Message};
use iced::{
    Element,
    widget::{button, column, pick_list, row, text, text_input},
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
    column![
        text("Current task"),
        row![
            selector,
            text_input("New task name", &app.new_task_name)
                .on_input(Message::TaskName)
                .on_submit(Message::AddTask)
                .width(220),
            add
        ]
        .spacing(8)
    ]
    .spacing(6)
    .into()
}
