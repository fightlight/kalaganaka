use iced::{
    Element, Fill, Padding,
    widget::{button, column, container, pick_list, row, stack, text, text_input},
};

use crate::{
    app::{App, Message},
    settings::{MAX_TASK_LIMIT, MIN_TASK_LIMIT, ThemePreference},
};

pub fn view(app: &App) -> Element<'_, Message> {
    let selector = pick_list(
        ThemePreference::ALL,
        Some(app.settings.theme),
        Message::SetTheme,
    )
    .width(180);
    let limit_input = text_input("1–30", &app.task_limit_input)
        .padding(Padding {
            top: 8.0,
            right: 26.0,
            bottom: 8.0,
            left: 8.0,
        })
        .on_input_maybe(app.ready().then_some(Message::TaskLimitInput))
        .on_submit_maybe(app.ready().then_some(Message::SaveTaskLimit));
    let current_limit = app
        .task_limit_input
        .parse::<u32>()
        .ok()
        .filter(|value| (MIN_TASK_LIMIT..=MAX_TASK_LIMIT).contains(value))
        .unwrap_or(app.settings.task_limit);
    let down = button(text("▼").size(10))
        .padding(0)
        .width(20)
        .height(16)
        .style(button::text)
        .on_press_maybe(
            (app.ready() && current_limit > MIN_TASK_LIMIT).then_some(Message::AdjustTaskLimit(-1)),
        );
    let up = button(text("▲").size(10))
        .padding(0)
        .width(20)
        .height(16)
        .style(button::text)
        .on_press_maybe(
            (app.ready() && current_limit < MAX_TASK_LIMIT).then_some(Message::AdjustTaskLimit(1)),
        );
    let limit = stack![
        limit_input,
        container(column![up, down])
            .width(Fill)
            .height(Fill)
            .align_x(iced::Right)
            .align_y(iced::Center)
            .padding(2),
    ]
    .width(74);
    let mut content = column![
        text("Settings").size(26),
        row![text("Appearance").width(Fill), selector]
            .spacing(16)
            .align_y(iced::Center),
        row![text("Task results limit").width(Fill), limit]
            .spacing(8)
            .align_y(iced::Center),
    ]
    .spacing(18);
    if app.task_limit_error {
        content = content.push(text("Task results limit must be between 1 and 30."));
    }
    if app.busy {
        content = content.push(text(if app.loaded {
            "Saving…"
        } else {
            "Loading…"
        }));
    }
    if let Some(error) = &app.error {
        content = content.push(text(error));
        if !app.busy {
            content = content.push(button("Reload").on_press(Message::Retry));
        }
    }
    container(content)
        .padding(24)
        .width(Fill)
        .height(Fill)
        .into()
}
