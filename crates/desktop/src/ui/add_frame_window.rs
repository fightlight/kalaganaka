use crate::app::{App, Message};
use chrono::{Datelike, Days};
use iced::{
    Element, Fill,
    widget::{
        button, column, container, mouse_area, row, scrollable, space, stack, text, text_input,
    },
};

pub fn view(app: &App) -> Element<'_, Message> {
    let date = button(text(app.add_frame_date.format("%d %B %Y").to_string()))
        .width(Fill)
        .on_press(Message::ToggleAddFrameDate);
    let label = app
        .add_frame_task
        .as_ref()
        .map_or("Select a task", |task| task.name.as_str());
    let mut selector = button(row![text(label).width(Fill), text("▾")])
        .width(Fill)
        .on_press(Message::ToggleAddFrameTasks);
    if app.add_frame_focus == 0 {
        selector = selector.style(button::secondary);
    }
    let time = |label: &'static str, index: usize| {
        row![
            text(label).width(55),
            text_input("HH", &app.add_frame_times[index])
                .id(format!("add-frame-time-{index}"))
                .on_input(move |value| Message::AddFrameTime(index, value))
                .width(50),
            text(":"),
            text_input("MM", &app.add_frame_times[index + 1])
                .id(format!("add-frame-time-{}", index + 1))
                .on_input(move |value| Message::AddFrameTime(index + 1, value))
                .width(50)
        ]
        .spacing(6)
        .align_y(iced::Center)
    };
    let mut date_content = column![text("Date"), date].spacing(10);
    if app.add_frame_date_open {
        date_content = date_content.push(date_picker(app));
    }
    let mut content = column![
        dismiss_outside(date_content.into(), app.add_frame_tasks_open),
        text("Task"),
        selector
    ]
    .spacing(10);
    if app.add_frame_tasks_open {
        let mut results = column![].spacing(2);
        let tasks: Vec<_> = matching_tasks(app).collect();
        if tasks.is_empty() && !app.add_frame_search.trim().is_empty() {
            results = results.push(
                button(row![
                    text(app.add_frame_search.trim()).width(Fill),
                    text("+")
                ])
                .width(Fill)
                .on_press_maybe(app.ready().then_some(Message::CreateAddFrameTask)),
            );
        }
        for (index, task) in tasks.into_iter().enumerate() {
            let mut item = button(text(&task.name).width(Fill))
                .width(Fill)
                .on_press(Message::AddFrameTask(task.clone()));
            if index != app.add_frame_search_index {
                item = item.style(button::text);
            }
            results = results.push(item);
        }
        content = content.push(
            text_input("Search tasks", &app.add_frame_search)
                .id("add-frame-search")
                .on_input(Message::AddFrameSearch),
        );
        content = content.push(scrollable(results).id("add-frame-results").height(150));
    }
    let mut bottom = column![].spacing(10);
    bottom = bottom.push(time("Start:", 0)).push(time("End:", 2));
    if let Some(error) = &app.add_frame_error {
        bottom = bottom.push(text(error));
    }
    let mut save = button("Add frame").on_press_maybe(app.ready().then_some(Message::SaveNewFrame));
    let mut cancel = button("Cancel").on_press_maybe(
        (!app.busy).then(|| Message::CloseRequested(app.add_frame_window.expect("window is open"))),
    );
    if app.add_frame_focus == 5 {
        save = save.style(button::secondary);
    }
    if app.add_frame_focus == 6 {
        cancel = cancel.style(button::secondary);
    }
    bottom = bottom.push(row![save, cancel].spacing(10));
    content = content.push(dismiss_outside(bottom.into(), app.add_frame_tasks_open));
    let main = container(scrollable(content))
        .padding(20)
        .width(Fill)
        .height(Fill);
    if app.add_frame_tasks_open {
        mouse_area(main)
            .on_press(Message::DismissAddFrameTasks)
            .into()
    } else {
        main.into()
    }
}

pub fn matching_tasks(app: &App) -> impl Iterator<Item = &tracking_core::Task> {
    let query = app.add_frame_search.trim().to_lowercase();
    app.snapshot
        .tasks
        .iter()
        .filter(move |task| task.name.to_lowercase().contains(&query))
}

fn dismiss_outside<'a>(content: Element<'a, Message>, open: bool) -> Element<'a, Message> {
    if open {
        stack![
            content,
            mouse_area(space().width(Fill).height(Fill)).on_press(Message::DismissAddFrameTasks)
        ]
        .into()
    } else {
        content
    }
}

fn date_picker(app: &App) -> Element<'_, Message> {
    let month = app.add_frame_month.with_day(1).expect("first day exists");
    let header = row![
        button("‹").on_press(Message::AddFrameMonth(-1)),
        text(month.format("%B %Y").to_string())
            .width(Fill)
            .align_x(iced::Center),
        button("›").on_press(Message::AddFrameMonth(1))
    ]
    .align_y(iced::Center);
    let mut calendar = column![header].spacing(4);
    let mut weekdays = row![].spacing(2);
    for day in ["M", "T", "W", "T", "F", "S", "S"] {
        weekdays = weekdays.push(text(day).width(Fill).align_x(iced::Center));
    }
    calendar = calendar.push(weekdays);
    let offset = month.weekday().num_days_from_monday();
    for week in 0..6u32 {
        let mut days = row![].spacing(2);
        for weekday in 0..7 {
            let index = week * 7 + weekday;
            let date = index
                .checked_sub(offset)
                .and_then(|n| month.checked_add_days(Days::new(n as u64)))
                .filter(|date| date.month() == month.month());
            if let Some(date) = date {
                let mut day = button(
                    text(date.day().to_string())
                        .width(Fill)
                        .align_x(iced::Center),
                )
                .width(Fill)
                .padding(4)
                .on_press(Message::AddFrameDate(date));
                if date != app.add_frame_date {
                    day = day.style(button::text);
                }
                days = days.push(day);
            } else {
                days = days.push(space().width(Fill));
            }
        }
        calendar = calendar.push(days);
    }
    container(calendar).width(Fill).into()
}
