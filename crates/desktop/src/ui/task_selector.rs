use super::icons::{self, Icon};
use crate::app::{App, Message};
use iced::{
    Color, Element, Fill, Padding,
    widget::{button, column, container, row, scrollable, space, stack, text, text_input},
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
    let add_message =
        (app.ready() && !app.new_task_name.trim().is_empty()).then_some(Message::AddTask);
    let input = text_input("New task name", &app.new_task_name)
        .on_input_maybe(app.ready().then_some(Message::TaskName))
        .on_submit_maybe(add_message.clone())
        .size(16)
        .line_height(1.0)
        .padding(Padding {
            top: 10.0,
            right: 40.0,
            bottom: 10.0,
            left: 10.0,
        });
    let add = button(container(text("+").size(24)).center_x(Fill).center_y(Fill))
        .width(32)
        .height(32)
        .padding(0)
        .style(button::text)
        .on_press_maybe(add_message);
    let new_task = stack![
        input,
        container(add)
            .width(Fill)
            .height(Fill)
            .align_x(iced::Right)
            .align_y(iced::Center)
            .padding(2)
    ]
    .width(Fill)
    .height(FIELD_HEIGHT);
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
        space().width(Fill),
        column![text("Task"), controls].spacing(6)
    ]
    .align_y(iced::Bottom)
    .into()
}

pub fn menu(app: &App) -> Element<'_, Message> {
    let mut items = column![];
    let matches = matching_tasks(app);
    let mut found = false;
    for task in matches {
        found = true;
        let item: Element<'_, Message> = if let Some((id, name)) = &app.editing_task {
            if *id == task.id {
                text_input("Task name", name)
                    .id("task-name-edit")
                    .on_input_maybe(app.ready().then_some(Message::EditTaskName))
                    .on_submit_maybe(app.ready().then_some(Message::SaveTaskName))
                    .padding(10)
                    .into()
            } else {
                task_item(app, task)
            }
        } else {
            task_item(app, task)
        };
        items = items.push(item);
    }
    if !found {
        items = items.push(container(text("No matching tasks")).padding(10));
    }
    let search_input = text_input("Search tasks", &app.task_search)
        .id("task-search")
        .on_input_maybe(app.ready().then_some(Message::SearchTasks))
        .padding(10);
    let search: Element<'_, Message> = if app.editing_task.is_some() && app.ready() {
        button(if app.task_search.is_empty() {
            "Search tasks"
        } else {
            &app.task_search
        })
        .width(Fill)
        .padding(10)
        .style(button::text)
        .on_press(Message::FocusTaskSearch)
        .into()
    } else {
        search_input.into()
    };
    container(column![
        search,
        scrollable(items)
            .id("task-results")
            .height(iced::Length::Shrink)
    ])
    .width(260)
    .max_height(300)
    .style(container::rounded_box)
    .into()
}

fn matching_tasks(app: &App) -> impl Iterator<Item = &tracking_core::Task> {
    let query = app.task_search.to_lowercase();
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

fn task_item<'a>(app: &'a App, task: &'a tracking_core::Task) -> Element<'a, Message> {
    let select = button(text(&task.name).width(Fill))
        .width(Fill)
        .padding(10)
        .style(button::text)
        .on_press_maybe(app.ready().then_some(Message::SelectTask(task.clone())));
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
        app.task_search = "A".into();
        assert_eq!(names(&app), ["Again", "Delta", "Last"]);
        app.task_search = "Ga".into();
        assert_eq!(names(&app), ["Gamma", "Again"]);
        app.task_search = "missing".into();
        assert!(names(&app).is_empty());
    }
}
