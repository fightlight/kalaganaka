use super::icons::{self, Icon};
use crate::app::{App, FrameField, Message};
use chrono::{DateTime, Local, Utc};
use iced::{
    Color, Element,
    widget::{button, column, container, row, space, text, text_input},
};

fn elapsed(minutes: i64) -> String {
    let mut remaining = minutes.max(0);
    let mut parts = Vec::new();
    for (size, suffix) in [(10080, "w"), (1440, "d"), (60, "h"), (1, "m")] {
        let value = remaining / size;
        remaining %= size;
        if value > 0 {
            parts.push(format!("{value}{suffix}"));
        }
    }
    if parts.is_empty() {
        "0m".into()
    } else {
        parts.join(" ")
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let heading = text("Time Frame").font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::default()
    });
    let add = button(icons::view(Icon::Add, 18.0))
        .padding(2)
        .style(button::text)
        .on_press_maybe(app.ready().then_some(Message::OpenAddFrame));
    let mut content = column![row![heading, add].spacing(6).align_y(iced::Center)].spacing(3);
    if let Some((task_id, start, end)) = app.frame_details() {
        if let Some(task) = app.snapshot.tasks.iter().find(|task| task.id == task_id) {
            let colour = Color::from_rgb8(
                (task.colour >> 16) as u8,
                (task.colour >> 8) as u8,
                task.colour as u8,
            );
            let swatch = container(space().width(12).height(12)).style(move |_| container::Style {
                background: Some(colour.into()),
                ..Default::default()
            });
            content = content.push(
                row![text("Task:"), swatch, text(&task.name)]
                    .spacing(6)
                    .align_y(iced::Center),
            );
            let until = end.unwrap_or_else(|| app.now.with_timezone(&Utc));
            content = content.push(text(format!(
                "Spent: {}",
                elapsed((until - start).num_minutes())
            )));
            content = content.push(text(format!(
                "Date: {}",
                start.with_timezone(&Local).format("%d %B %Y")
            )));
            content = content.push(time_row(app, "Start:", FrameField::Start, Some(start)));
            content = content.push(time_row(app, "End:", FrameField::End, end));
        }
    }
    content.into()
}

fn time_row<'a>(
    app: &'a App,
    label: &'static str,
    field: FrameField,
    time: Option<DateTime<Utc>>,
) -> Element<'a, Message> {
    let mut line = row![text(label)].spacing(6).align_y(iced::Center);
    if app.editing_frame_field == Some(field) {
        line = line.push(
            text_input("HH", &app.frame_hour_input)
                .id("frame-time")
                .on_input_maybe(app.ready().then_some(Message::FrameHourInput))
                .on_submit_maybe(app.ready().then_some(Message::SaveFrameTime))
                .width(45),
        );
        line = line.push(text(":"));
        line = line.push(
            text_input("MM", &app.frame_minute_input)
                .id("frame-minute")
                .on_input_maybe(app.ready().then_some(Message::FrameMinuteInput))
                .on_submit_maybe(app.ready().then_some(Message::SaveFrameTime))
                .width(45),
        );
        line = line.push(
            button(icons::view(Icon::Check, 18.0))
                .padding(2)
                .style(button::text)
                .on_press_maybe(app.ready().then_some(Message::SaveFrameTime)),
        );
    } else if let Some(time) = time {
        line = line.push(text(time.with_timezone(&Local).format("%H:%M").to_string()));
        line = line.push(
            button(icons::view(Icon::Pencil, 18.0))
                .padding(2)
                .style(button::text)
                .on_press_maybe(app.ready().then_some(Message::EditFrameTime(field))),
        );
    } else {
        line = line.push(text("..."));
    }
    line.into()
}
