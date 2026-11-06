use super::{
    calendar,
    icons::{self, Icon},
    task_selector, time_frame,
};
use crate::app::{App, Message, ViewMode};
use chrono::Datelike;
use iced::{
    Element, Fill,
    widget::{button, column, container, mouse_area, row, space, stack, text, tooltip},
};

pub fn view(app: &App) -> Element<'_, Message> {
    let (next_view, switch_label) = match app.mode {
        ViewMode::Day => (ViewMode::Week, "Switch to week view"),
        ViewMode::Week => (ViewMode::Day, "Switch to day view"),
    };
    let actions = row![
        tooltip(
            button(
                container(text(app.now.day().to_string()).size(18))
                    .center_x(24)
                    .center_y(24)
                    .style(|theme: &iced::Theme| container::Style {
                        border: iced::Border {
                            color: theme.extended_palette().background.base.text,
                            width: 1.8,
                            radius: 2.0.into(),
                        },
                        ..container::Style::default()
                    })
            )
            .padding(10)
            .style(button::text)
            .on_press(Message::Today),
            "Go to today",
            tooltip::Position::Bottom,
        ),
        tooltip(
            button(icons::view(Icon::Calendar, 24.0))
                .padding(10)
                .style(button::text)
                .on_press(Message::ChangeView(next_view)),
            switch_label,
            tooltip::Position::Bottom,
        ),
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
    let arrows = row![
        button(icons::view(Icon::Previous, 24.0))
            .padding(5)
            .style(button::text)
            .on_press(Message::Navigate(-1)),
        button(icons::view(Icon::Next, 24.0))
            .padding(5)
            .style(button::text)
            .on_press(Message::Navigate(1)),
    ];
    let navigation = row![arrows, space().width(Fill), actions]
        .spacing(10)
        .width(Fill)
        .align_y(iced::Center);
    let toolbar = row![
        space().width(Fill),
        text(app.date.format("%d %B %Y").to_string()),
        navigation
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
        .push(row![time_frame::view(app), task_selector::view(app)].align_y(iced::Top));
    let content = container(content).padding(18).width(Fill).height(Fill);
    let mut layers = stack![content];
    if !app.new_task_name.trim().is_empty() && !app.task_menu_open {
        let dismiss =
            || mouse_area(space().width(Fill).height(Fill)).on_press(Message::DismissSearch);
        layers = layers.push(
            column![
                dismiss(),
                row![
                    dismiss(),
                    space().width(260),
                    container(dismiss()).width(106)
                ]
                .height(36),
                container(dismiss()).height(62),
            ]
            .width(Fill)
            .height(Fill),
        );
        layers = layers.push(
            container(task_selector::search_results(app))
                .width(Fill)
                .height(Fill)
                .align_x(iced::Right)
                .align_y(iced::Bottom)
                .padding(iced::Padding {
                    top: 18.0,
                    right: 106.0,
                    bottom: 106.0,
                    left: 18.0,
                }),
        );
    }
    if app.task_menu_open {
        let dismiss =
            mouse_area(space().width(Fill).height(Fill)).on_press(Message::DismissTaskMenu);
        layers = layers.push(dismiss);
        layers = layers.push(
            container(task_selector::menu(app))
                .width(Fill)
                .height(Fill)
                .align_x(iced::Right)
                .align_y(iced::Bottom)
                .padding(iced::Padding {
                    top: 18.0,
                    right: 106.0,
                    bottom: 62.0,
                    left: 18.0,
                }),
        );
    }
    layers.into()
}
