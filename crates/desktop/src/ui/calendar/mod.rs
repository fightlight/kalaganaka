mod canvas;
pub mod layout;

use crate::app::{App, Message};
use iced::{
    Element, Fill, Task,
    widget::{canvas as canvas_widget, operation, scrollable},
};

const SCROLL_ID: &str = "calendar";

pub fn scroll_to_now(app: &App) -> Task<Message> {
    if !layout::dates(app.date, app.mode).contains(&app.now.date_naive()) {
        return Task::none();
    }

    scroll_to_time(app.now.with_timezone(&chrono::Utc))
}

pub fn scroll_to_time(time: chrono::DateTime<chrono::Utc>) -> Task<Message> {
    operation::scroll_to(
        SCROLL_ID,
        scrollable::AbsoluteOffset {
            x: 0.0,
            // Keep the current-time marker below the top edge of the viewport.
            y: layout::y(layout::minute(time)) - layout::HEADER,
        },
    )
}

pub fn view(app: &App) -> Element<'_, Message> {
    scrollable(
        canvas_widget(canvas::Calendar { app })
            .width(Fill)
            .height(layout::HEIGHT),
    )
    .id(SCROLL_ID)
    .height(Fill)
    .into()
}
