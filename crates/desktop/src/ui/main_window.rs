use super::{
    calendar,
    icons::{self, Icon},
    task_selector,
};
use crate::app::{App, Message, ViewMode};
use iced::{
    Element, Fill,
    widget::{button, column, container, row, space, text, tooltip},
};

pub fn view(app: &App) -> Element<'_, Message> {
    let toolbar = row![
        button("‹").on_press(Message::Navigate(-1)),
        button("Today").on_press(Message::Today),
        button("›").on_press(Message::Navigate(1)),
        text(app.date.format("%d %B %Y").to_string()),
        button("Day").on_press(Message::ChangeView(ViewMode::Day)),
        button("Week").on_press(Message::ChangeView(ViewMode::Week)),
        space().width(Fill),
        tooltip(
            button(icons::view(Icon::Statistics, 24.0))
                .padding(10)
                .style(button::text)
                .on_press(Message::OpenStatistics),
            "Statistics",
            tooltip::Position::Bottom,
        ),
        tooltip(
            button(icons::view(Icon::Settings, 24.0))
                .padding(10)
                .style(button::text)
                .on_press(Message::OpenSettings),
            "Settings",
            tooltip::Position::Bottom,
        ),
    ]
    .spacing(10)
    .align_y(iced::Center);
    // Keep status messages inside the header so they cannot replace the
    // calendar's widget state when they appear or disappear.
    let mut header = column![toolbar].spacing(12);
    if app.busy {
        header = header.push(text("Saving / loading…"));
    }
    if let Some(error) = &app.error {
        header =
            header.push(row![text(error), button("Reload").on_press(Message::Retry)].spacing(8));
    }
    if app.recovered_session {
        header = header.push(text("Recovered a running session. Review it and pause when ready; time since its start is included."));
    }
    if app.confirm_quit {
        header = header.push(
            row![
                text("A task is running."),
                button("Pause and quit").on_press(Message::ConfirmQuit),
                button("Cancel").on_press(Message::CancelQuit)
            ]
            .spacing(12),
        );
    }
    let content = column![header]
        .spacing(12)
        .push(calendar::view(app))
        .push(task_selector::view(app));
    container(content)
        .padding(18)
        .width(Fill)
        .height(Fill)
        .into()
}
