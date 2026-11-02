use iced::{
    Element, Fill,
    widget::{button, column, container, pick_list, row, text},
};

use crate::{
    app::{App, Message},
    settings::ThemePreference,
};

pub fn view(app: &App) -> Element<'_, Message> {
    let selector = pick_list(
        ThemePreference::ALL,
        Some(app.settings.theme),
        Message::SetTheme,
    )
    .width(180);
    let mut content = column![
        text("Settings").size(26),
        row![text("Appearance").width(Fill), selector]
            .spacing(16)
            .align_y(iced::Center),
        text("System follows your device’s appearance."),
    ]
    .spacing(18);
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
