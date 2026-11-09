use super::icons::{self, Icon};
use crate::app::{App, Message};
use iced::{
    Color, Element, Fill,
    widget::{button, column, container, row, scrollable, text, text_input},
};

const FIELD_HEIGHT: f32 = 36.0;
const SPACING: f32 = 8.0;
const TOGGLE_SIZE: f32 = FIELD_HEIGHT * 2.0 + SPACING;

pub fn view(app: &App) -> Element<'_, Message> {
    let label = app
        .selected
        .as_ref()
        .map_or("Select a task", |task| task.name.as_str());
    let selector = button(row![text(label).size(16).width(Fill), text("▾")])
        .padding([10, 10])
        .width(260)
        .on_press_maybe(app.ready().then_some(Message::ToggleTaskMenu));
    let new_task = text_input("Enter task name", &app.new_task_name)
        .on_input_maybe(app.ready().then_some(Message::TaskName))
        .on_submit_maybe(app.ready().then_some(Message::SearchSubmit))
        .size(16)
        .line_height(1.0)
        .padding(10)
        .width(Fill);
    let running = app.snapshot.active.is_some();
    let (icon, color) = if running {
        (Icon::Pause(Color::BLACK), Color::from_rgb8(67, 181, 111))
    } else {
        (Icon::Play(Color::BLACK), Color::from_rgb8(245, 200, 66))
    };
    let mut toggle = button(
        container(icons::view(icon, 38.0))
            .center_x(Fill)
            .center_y(Fill),
    )
    .width(TOGGLE_SIZE)
    .height(TOGGLE_SIZE)
    .style(move |theme, status| {
        let opacity = match status {
            button::Status::Active => 1.0,
            button::Status::Hovered | button::Status::Pressed => 0.85,
            button::Status::Disabled => 0.5,
        };
        let base = button::primary(theme, status);
        button::Style {
            background: Some(color.scale_alpha(opacity).into()),
            text_color: Color::BLACK,
            border: iced::Border {
                radius: (TOGGLE_SIZE / 2.0).into(),
                ..base.border
            },
            ..base
        }
    });
    if app.ready() && app.selected.is_some() {
        toggle = toggle.on_press(Message::Toggle);
    }
    let controls = row![
        column![new_task, selector].spacing(SPACING).width(260),
        toggle
    ]
    .spacing(SPACING)
    .height(TOGGLE_SIZE);
    row![
        column![
            text("Task").font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::default()
            }),
            controls
        ]
        .spacing(6)
    ]
    .align_y(iced::Bottom)
    .into()
}

pub fn menu(app: &App) -> Element<'_, Message> {
    let mut items = column![];
    for (index, task) in recent_tasks(app).enumerate() {
        let item = editable_task_item(app, task, index == app.search_index);
        items = items.push(item);
    }
    container(column![
        scrollable(items)
            .id("task-results")
            .height(iced::Length::Shrink)
    ])
    .width(260)
    .max_height(300)
    .style(container::rounded_box)
    .into()
}

pub fn recent_tasks(app: &App) -> impl Iterator<Item = &tracking_core::Task> {
    let mut tasks: Vec<_> = app.snapshot.tasks.iter().collect();
    tasks.sort_by_key(|task| last_used(&app.snapshot, task.id));
    let skip = tasks.len().saturating_sub(app.settings.task_limit as usize);
    tasks.into_iter().skip(skip)
}

pub fn last_used(
    snapshot: &tracking_core::Snapshot,
    task_id: uuid::Uuid,
) -> Option<chrono::DateTime<chrono::Utc>> {
    snapshot
        .frames
        .iter()
        .filter(|frame| frame.task_id == task_id)
        .map(|frame| frame.started_at)
        .chain(
            snapshot
                .active
                .iter()
                .filter(|active| active.task_id == task_id)
                .map(|active| active.started_at),
        )
        .max()
}

pub fn matching_tasks(app: &App) -> impl Iterator<Item = &tracking_core::Task> {
    let query = app.new_task_name.trim().to_lowercase();
    app.snapshot
        .tasks
        .iter()
        .filter(move |task| task.name.to_lowercase().contains(&query))
        .rev()
        .take(app.settings.task_limit as usize)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
}

pub fn search_results(app: &App) -> Element<'_, Message> {
    let matches: Vec<_> = matching_tasks(app).collect();
    let mut items = column![];
    if matches.is_empty() {
        items = items.push(
            button(row![text(app.new_task_name.trim()).width(Fill), text("+")])
                .width(Fill)
                .padding(10)
                .on_press_maybe(app.ready().then_some(Message::AddTask)),
        );
    } else {
        for (index, task) in matches.into_iter().enumerate() {
            items = items.push(editable_task_item(app, task, index == app.search_index));
        }
    }
    container(scrollable(items).height(iced::Length::Shrink))
        .width(260)
        .max_height(300)
        .style(container::rounded_box)
        .into()
}

fn editable_task_item<'a>(
    app: &'a App,
    task: &'a tracking_core::Task,
    highlighted: bool,
) -> Element<'a, Message> {
    if let Some((id, name)) = &app.editing_task {
        if *id == task.id {
            return text_input("Task name", name)
                .id("task-name-edit")
                .on_input_maybe(app.ready().then_some(Message::EditTaskName))
                .on_submit_maybe(app.ready().then_some(Message::SaveTaskName))
                .padding(10)
                .into();
        }
    }
    task_item(app, task, highlighted)
}

fn task_item<'a>(
    app: &'a App,
    task: &'a tracking_core::Task,
    highlighted: bool,
) -> Element<'a, Message> {
    let mut select = button(text(&task.name).width(Fill))
        .width(Fill)
        .padding(10)
        .on_press_maybe(app.ready().then_some(Message::SelectTask(task.clone())));
    if !highlighted {
        select = select.style(button::text);
    }
    let pencil = button(icons::view(Icon::Pencil, 18.0))
        .padding(8)
        .style(button::text)
        .on_press_maybe(app.ready().then_some(Message::EditTask(task.clone())));
    row![select, pencil].align_y(iced::Center).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracking_core::{Snapshot, Tracker};

    #[test]
    fn newest_matching_tasks_keep_creation_order() {
        let (mut app, _) = App::new();
        let mut tracker = Tracker::new(Snapshot::default());
        for name in [
            "Alpha", "Beta", "Another", "Gamma", "Again", "Delta", "Last",
        ] {
            tracker.add_task(name).unwrap();
        }
        app.snapshot = tracker.into_snapshot();
        app.settings.task_limit = 3;
        let names = |app: &App| {
            matching_tasks(app)
                .map(|task| task.name.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&app), ["Again", "Delta", "Last"]);
        app.new_task_name = "A".into();
        assert_eq!(names(&app), ["Again", "Delta", "Last"]);
        app.new_task_name = "Ga".into();
        assert_eq!(names(&app), ["Gamma", "Again"]);
        app.new_task_name = "missing".into();
        assert!(names(&app).is_empty());
    }
}
