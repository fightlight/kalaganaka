use super::icons::{self, Icon};
use crate::app::{App, FrameField, Message};
use chrono::{DateTime, Local, Utc};
use iced::{
    Color, Element, Fill,
    widget::{
        button, column, container, mouse_area, row, scrollable, space, text, text_editor,
        text_input,
    },
};

const FRAME_DETAILS_WIDTH: f32 = 270.0;

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

pub fn view(app: &App, stacked: bool) -> Element<'_, Message> {
    let heading = text("Time Frame").font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::default()
    });
    let add = button(icons::view(Icon::Add, 18.0))
        .padding(2)
        .style(button::text)
        .on_press_maybe(app.ready().then_some(Message::OpenAddFrame));
    let mut header = row![heading, add].spacing(6).align_y(iced::Center);
    let mut content = column![].spacing(3);
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
            header = header.push(swatch).push(text(&task.name));
            let until = end.unwrap_or_else(|| app.now.with_timezone(&Utc));
            content = content.push(text(
                start
                    .with_timezone(&Local)
                    .format("%A %d %B %Y")
                    .to_string(),
            ));
            content = content.push(
                row![
                    time_row(app, FrameField::Start, Some(start)),
                    text("–"),
                    time_row(app, FrameField::End, end),
                    text(format!("({})", elapsed((until - start).num_minutes())))
                ]
                .spacing(6)
                .align_y(iced::Center),
            );
        }
    }
    if app.frame_metadata().is_none() {
        return column![header, content].spacing(3).width(Fill).into();
    }
    let left = column![content, tags(app)]
        .spacing(3)
        .width(FRAME_DETAILS_WIDTH);
    if stacked {
        return column![header, left, description(app)]
            .spacing(3)
            .width(Fill)
            .into();
    }
    column![
        header,
        row![left, description(app)]
            .spacing(16)
            .align_y(iced::Top)
            .width(Fill)
    ]
    .spacing(3)
    .width(Fill)
    .into()
}

fn metadata_heading(
    name: &'static str,
    editing: bool,
    tags: bool,
    ready: bool,
) -> Element<'static, Message> {
    row![
        text(name),
        button(icons::view(
            if editing { Icon::Check } else { Icon::Pencil },
            18.0
        ))
        .padding(2)
        .style(button::text)
        .on_press_maybe(ready.then_some(if editing {
            Message::SaveMetadata(tags)
        } else {
            Message::EditMetadata(tags)
        }))
    ]
    .spacing(6)
    .align_y(iced::Center)
    .into()
}

fn tags(app: &App) -> Element<'_, Message> {
    let metadata = app.frame_metadata().expect("selected frame exists");
    let tags = if app.editing_tags {
        &app.tag_draft
    } else {
        &metadata.tags
    };
    let mut chips = row![text("Tags:")].spacing(4).align_y(iced::Center);
    for (index, tag) in tags.iter().enumerate() {
        let mut chip = row![text(tag)].spacing(4).align_y(iced::Center);
        {
            chip = chip.push(
                button("×")
                    .padding(2)
                    .style(button::text)
                    .on_press_maybe(app.ready().then_some(Message::TagRemove(index))),
            );
        }
        chips = chips.push(
            container(chip)
                .padding([2, 4])
                .style(container::rounded_box),
        );
    }
    if app.editing_tags {
        let input = text_input("Add tag", &app.tag_input)
            .id("frame-tags")
            .on_input_maybe(app.ready().then_some(Message::TagInput))
            .on_submit_maybe(app.ready().then_some(Message::TagSubmit))
            .width(100);
        let suggestions = (!app.tag_input.trim().is_empty()).then(|| tag_suggestions(app));
        chips = chips.push(super::popover::above(input, suggestions));
    } else {
        chips = chips.push(
            button("+")
                .padding([2, 6])
                .on_press_maybe(app.ready().then_some(Message::EditMetadata(true))),
        );
    }
    mouse_area(chips.width(FRAME_DETAILS_WIDTH).wrap())
        .on_enter(Message::TagAreaHovered(true))
        .on_exit(Message::TagAreaHovered(false))
        .on_press(Message::MetadataAreaPressed)
        .into()
}

fn tag_suggestions(app: &App) -> Element<'_, Message> {
    let suggestions = app.tag_suggestions();
    let mut results = column![];
    if suggestions.is_empty() {
        results = results.push(
            button(row![text(app.tag_input.trim()).width(Fill), text("+")])
                .width(Fill)
                .on_press(Message::TagSubmit),
        );
    } else {
        for (index, tag) in suggestions.into_iter().enumerate() {
            let mut item = button(text(tag.clone()).width(Fill))
                .width(Fill)
                .on_press(Message::TagChoose(tag));
            if index != app.tag_index {
                item = item.style(button::text);
            }
            results = results.push(item);
        }
    }
    mouse_area(
        container(scrollable(results).height(iced::Length::Shrink))
            .max_height(160)
            .width(200)
            .style(container::rounded_box),
    )
    .on_enter(Message::TagSuggestionsHovered(true))
    .on_exit(Message::TagSuggestionsHovered(false))
    .into()
}

fn description(app: &App) -> Element<'_, Message> {
    let metadata = app.frame_metadata().expect("selected frame exists");
    let field: Element<'_, Message> = if app.editing_description {
        text_editor(&app.description_draft)
            .id("frame-description")
            .height(56)
            .on_action(Message::DescriptionInput)
            .into()
    } else {
        container(text(&metadata.description).width(Fill))
            .padding(8)
            .height(56)
            .width(Fill)
            .style(container::rounded_box)
            .into()
    };
    mouse_area(
        column![
            space().width(280).height(0),
            metadata_heading("Description", app.editing_description, false, app.ready()),
            field
        ]
        .spacing(3)
        .width(Fill),
    )
    .on_enter(Message::DescriptionAreaHovered(true))
    .on_exit(Message::DescriptionAreaHovered(false))
    .on_press(Message::MetadataAreaPressed)
    .into()
}

fn time_row<'a>(
    app: &'a App,
    field: FrameField,
    time: Option<DateTime<Utc>>,
) -> Element<'a, Message> {
    let mut line = row![].spacing(6).align_y(iced::Center);
    if app.editing_frame_field == Some(field) {
        line = line.spacing(3);
        line = line.push(
            text_input("HH", &app.frame_hour_input)
                .id("frame-time")
                .on_input_maybe(app.ready().then_some(Message::FrameHourInput))
                .on_submit_maybe(app.ready().then_some(Message::SaveFrameTime))
                .padding([2, 3])
                .width(32),
        );
        line = line.push(text(":"));
        line = line.push(
            text_input("MM", &app.frame_minute_input)
                .id("frame-minute")
                .on_input_maybe(app.ready().then_some(Message::FrameMinuteInput))
                .on_submit_maybe(app.ready().then_some(Message::SaveFrameTime))
                .padding([2, 3])
                .width(32),
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
    if app.editing_frame_field == Some(field) {
        mouse_area(line)
            .on_enter(Message::FrameTimeHovered(true))
            .on_exit(Message::FrameTimeHovered(false))
            .into()
    } else {
        line.into()
    }
}
